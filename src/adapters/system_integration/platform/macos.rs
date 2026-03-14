//! macOS 平台系统集成实现
//!
//! 通过 CoreServices API (Python + LaunchServices) 设置默认程序
//! 右键菜单通过 Info.plist 配置自动支持"打开方式"

use super::IntegrationStatus;
use anyhow::{bail, Context, Result};
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

/// 支持的图片文件 UTI 列表（Uniform Type Identifier）
const IMAGE_UTIS: &[&str] = &[
    "public.png",         // PNG 图片
    "public.jpeg",        // JPEG 图片
    "public.tiff",        // TIFF 图片
    "com.compuserve.gif", // GIF 图片
    "com.microsoft.bmp",  // BMP 图片
    "com.adobe.webp",     // WebP 图片
    "public.image",       // 通用图片类型
];

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

/// 获取当前可执行文件的 Bundle Identifier
fn get_bundle_id() -> Result<String> {
    // 首先尝试从当前可执行文件路径获取 bundle ID
    if let Ok(exe_path) = std::env::current_exe() {
        // 如果应用被打包成 .app 格式，尝试读取 Info.plist
        if let Some(bundle_id) = read_bundle_id_from_plist(&exe_path) {
            return Ok(bundle_id);
        }
    }

    // 默认使用开发时的 bundle ID
    Ok("com.openappsys.oas-image-viewer".to_string())
}

/// 从可执行文件路径尝试读取 bundle ID
/// 在 macOS 中，应用通常位于 MyApp.app/Contents/MacOS/executable
fn read_bundle_id_from_plist(exe_path: &PathBuf) -> Option<String> {
    // 尝试找到 .app 包目录
    let mut current = exe_path.as_path();

    // 向上遍历查找 .app 目录
    while let Some(parent) = current.parent() {
        if let Some(name) = parent.file_name()?.to_str() {
            if name.ends_with(".app") {
                // 找到 .app 包，读取 Info.plist
                let plist_path = parent.join("Contents/Info.plist");
                if plist_path.exists() {
                    return parse_bundle_id_from_plist(&plist_path);
                }
            }
        }
        current = parent;
    }

    None
}

/// 解析 plist 文件获取 Bundle Identifier
fn parse_bundle_id_from_plist(plist_path: &PathBuf) -> Option<String> {
    // 使用 plutil 命令将 plist 转换为 JSON 格式
    let output = Command::new("plutil")
        .args(["-convert", "json", "-o", "-", plist_path.to_str()?])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8(output.stdout).ok()?;
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    // 获取 CFBundleIdentifier
    json.get("CFBundleIdentifier")?
        .as_str()
        .map(|s| s.to_string())
}

/// 获取指定 UTI 的默认程序 bundle ID
fn get_default_handler_for_uti(uti: &str) -> Result<String> {
    // 使用 Launch Services API 的 Python 脚本来获取默认程序
    let python_script = format!(
        r#"
import sys
try:
    from LaunchServices import LSCopyDefaultRoleHandlerForContentType
    result = LSCopyDefaultRoleHandlerForContentType("{}", 0xFFFFFFFF)
    if result:
        print(result)
        sys.exit(0)
except Exception as e:
    pass
sys.exit(1)
"#,
        uti
    );

    let output = Command::new("python3")
        .args(["-c", &python_script])
        .output()
        .or_else(|_| {
            Command::new("python")
                .args(["-c", &python_script])
                .output()
        })
        .context("无法运行 Python 查询默认程序")?;

    if output.status.success() {
        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !bundle_id.is_empty() {
            return Ok(bundle_id);
        }
    }

    bail!("无法获取 UTI {} 的默认程序", uti)
}

/// 使用 Python 脚本设置默认程序
fn set_default_with_python(uti: &str, bundle_id: &str) -> Result<()> {
    let python_script = format!(
        r#"
import sys
try:
    from LaunchServices import LSSetDefaultRoleHandlerForContentType
    import objc
    
    result = LSSetDefaultRoleHandlerForContentType(
        "{}",
        0xFFFFFFFF,
        "{}"
    )
    if result == 0:
        sys.exit(0)
    else:
        print(f"设置默认程序失败，错误码: {{result}}")
        sys.exit(1)
except ImportError as e:
    print(f"缺少必要的模块: {{e}}")
    sys.exit(1)
except Exception as e:
    print(f"错误: {{e}}")
    sys.exit(1)
"#,
        uti, bundle_id
    );

    let output = Command::new("python3")
        .args(["-c", &python_script])
        .output()
        .or_else(|_| {
            Command::new("python")
                .args(["-c", &python_script])
                .output()
        })
        .context("无法运行 Python 设置默认程序")?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("设置默认程序失败: {}", stderr)
    }
}

/// 使用 lsregister 命令注册应用（备选方案）
fn register_with_lsregister() -> Result<()> {
    if let Some(bundle_path) = get_bundle_path() {
        let output = Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
            .args(["-f", "-r", "-R", "-v", &bundle_path.to_string_lossy()])
            .output()
            .context("无法运行 lsregister 命令")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("lsregister 注册失败: {}", stderr)
        }
    } else {
        bail!("无法找到应用 bundle 路径")
    }
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
    // 使用第一个图片类型进行检查
    let first_uti = IMAGE_UTIS[0];

    match get_default_handler_for_uti(first_uti) {
        Ok(current_bundle) => {
            if let Ok(our_bundle) = get_bundle_id() {
                current_bundle == our_bundle
            } else {
                false
            }
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
    let bundle_id = get_bundle_id()?;

    // 首先尝试使用 Python + LaunchServices API
    let mut _any_success = false;
    let mut python_error = None;

    for uti in IMAGE_UTIS {
        match set_default_with_python(uti, &bundle_id) {
            Ok(()) => {
                _any_success = true;
            }
            Err(e) => {
                python_error = Some(e.to_string());
            }
        }
    }

    // 如果 Python 方式都失败，尝试使用 lsregister 作为备选
    if !_any_success {
        match register_with_lsregister() {
            Ok(()) => {
                _any_success = true;
            }
            Err(e) => {
                // 所有方式都失败，返回友好的错误提示
                bail!(
                    "MacOS 设置默认程序失败。\n\
                    \n\
                    可能的原因：\n\
                    - Python 或 PyObjC 模块未安装: {}\n\
                    - lsregister 命令失败: {}\n\
                    \n\
                    请手动设置：\n\
                    1. 右键图片文件 → 显示简介\n\
                    2. 在「打开方式」中选择「OAS Image Viewer」\n\
                    3. 点击「全部更改」",
                    python_error.unwrap_or_else(|| "未知错误".to_string()),
                    e
                );
            }
        }
    }

    // 通知 Finder 刷新（可选）
    let _ = Command::new("killall")
        .arg("-u")
        .arg("$USER")
        .arg("Finder")
        .output();

    Ok(())
}

/// 创建 Info.plist 文件（用于应用打包）
#[allow(dead_code)]
pub fn create_info_plist() -> Result<String> {
    Ok(INFO_PLIST_TEMPLATE.to_string())
}
