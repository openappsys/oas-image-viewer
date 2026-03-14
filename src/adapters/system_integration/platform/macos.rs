//! macOS 平台系统集成实现

use super::IntegrationStatus;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Info.plist 内容模板
const INFO_PLIST_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.openappsys.oas-image-viewer</string>
    <key>CFBundleName</key>
    <string>OAS Image Viewer</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Image File</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.image</string>
                <string>public.png</string>
                <string>public.jpeg</string>
                <string>public.gif</string>
                <string>public.tiff</string>
                <string>com.compuserve.gif</string>
                <string>com.microsoft.bmp</string>
                <string>com.adobe.webp</string>
            </array>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
        </dict>
    </array>
</dict>
</plist>
"#;

/// 获取应用 Bundle 路径
fn get_bundle_path() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;

    // 向上查找 .app 目录
    let mut path = exe_path.as_path();
    while let Some(parent) = path.parent() {
        if parent.extension()?.to_str()? == "app" {
            return Some(parent.to_path_buf());
        }
        path = parent;
    }

    None
}

/// 获取系统集成状态
pub fn get_integration_status() -> IntegrationStatus {
    IntegrationStatus {
        context_menu_registered: false, // macOS 不支持注册表方式的右键菜单
        default_app_registered: is_default_app_registered(),
    }
}

/// 检查是否已设为默认图片查看器
fn is_default_app_registered() -> bool {
    // 检查 .png 文件的默认打开方式
    match Command::new("duti").args(["-x", ".png"]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("oas-image-viewer")
        }
        Err(_) => false,
    }
}

/// macOS 不支持注册表方式的右键菜单，此方法返回错误
pub fn register_context_menu() -> Result<()> {
    Err(anyhow::anyhow!("macOS does not support registry-based context menu registration. Use the app bundle instead."))
}

/// macOS 不支持注册表方式的右键菜单，此方法返回错误
pub fn unregister_context_menu() -> Result<()> {
    Err(anyhow::anyhow!(
        "macOS does not support registry-based context menu registration."
    ))
}

/// 设为默认图片查看器
pub fn set_as_default_app() -> Result<()> {
    // 尝试使用 duti 工具设置默认应用
    let bundle_id = "com.openappsys.oas-image-viewer";

    // 支持的图片 UTIs
    let image_utis = [
        "public.png",
        "public.jpeg",
        "public.gif",
        "public.tiff",
        "com.compuserve.gif",
        "com.microsoft.bmp",
    ];

    // 检查 duti 是否可用
    match Command::new("duti").arg("-v").output() {
        Ok(_) => {
            // 使用 duti 设置默认应用
            for uti in &image_utis {
                Command::new("duti")
                    .args(["-s", bundle_id, uti, "all"])
                    .output()
                    .with_context(|| format!("Failed to set default app for {}", uti))?;
            }

            // 刷新 Launch Services 数据库
            Command::new("lsregister")
                .arg("-f")
                .arg(
                    get_bundle_path()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                )
                .output()
                .context("Failed to refresh Launch Services")?;

            Ok(())
        }
        Err(_) => {
            // duti 不可用，使用 LSRegister 注册应用
            if let Some(bundle_path) = get_bundle_path() {
                Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
                    .args(["-f", "-r", "-R", "-v"
                    ])
                    .arg(&bundle_path)
                    .output()
                    .context("Failed to register app with Launch Services")?;

                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Could not find app bundle. Please run from an .app bundle."
                ))
            }
        }
    }
}

/// 创建 Info.plist 文件（用于应用打包）
#[allow(dead_code)]
pub fn create_info_plist() -> Result<String> {
    Ok(INFO_PLIST_TEMPLATE.to_string())
}
