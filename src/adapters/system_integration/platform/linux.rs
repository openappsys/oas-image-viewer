//! Linux 平台系统集成实现

use super::IntegrationStatus;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 支持的图片 MIME 类型
const IMAGE_MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/tiff",
    "image/bmp",
    "image/x-tga",
    "image/x-portable-pixmap",
];

/// 获取当前可执行文件路径
fn get_exe_path() -> Result<PathBuf> {
    std::env::current_exe().context("无法获取可执行文件路径")
}

/// 获取 .desktop 文件路径
fn get_desktop_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    home.join(".local/share/applications/oas-image-viewer.desktop")
}

/// 获取应用图标路径
fn get_icon_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    home.join(".local/share/icons/hicolor/256x256/apps/oas-image-viewer.png")
}

/// 检查 .desktop 文件是否存在
fn desktop_file_exists() -> bool {
    get_desktop_file_path().exists()
}

/// 获取系统集成状态
pub fn get_integration_status() -> IntegrationStatus {
    IntegrationStatus {
        context_menu_registered: desktop_file_exists(),
        default_app_registered: is_default_app_registered(),
    }
}

/// 检查是否已设为默认图片查看器
fn is_default_app_registered() -> bool {
    // 检查 xdg-mime 查询结果
    match Command::new("xdg-mime")
        .args(["query", "default", "image/png"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("oas-image-viewer")
        }
        Err(_) => false,
    }
}

/// 注册右键菜单（Linux 通过创建 .desktop 文件实现）
pub fn register_context_menu() -> Result<()> {
    // Linux 的右键菜单集成是通过 .desktop 文件自动实现的
    // 只需要确保 .desktop 文件存在即可
    create_desktop_file()
}

/// 注销右键菜单
pub fn unregister_context_menu() -> Result<()> {
    let desktop_path = get_desktop_file_path();

    if desktop_path.exists() {
        fs::remove_file(&desktop_path)
            .with_context(|| format!("删除 .desktop 文件失败: {:?}", desktop_path))?;

        // 更新桌面数据库
        update_desktop_database()?;
    }

    Ok(())
}

/// 创建 .desktop 文件
fn create_desktop_file() -> Result<()> {
    let exe_path = get_exe_path()?;
    let exe_path_str = exe_path.to_string_lossy();

    let desktop_path = get_desktop_file_path();

    // 确保父目录存在
    if let Some(parent) = desktop_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("创建目录失败: {:?}", parent))?;
    }

    // 查找或创建图标
    let icon_path = find_or_create_icon()?;
    let icon_path_str = icon_path.to_string_lossy();

    // 生成 .desktop 文件内容
    let desktop_content = format!(
        r#"[Desktop Entry]
Name=OAS Image Viewer
GenericName=Image Viewer
Comment=A modern image viewer built with Rust and egui
Exec={} %F
Icon={}
Terminal=false
Type=Application
Categories=Graphics;Viewer;2DGraphics;RasterGraphics;
MimeType=image/png;image/jpeg;image/gif;image/webp;image/tiff;image/bmp;image/x-tga;image/x-portable-pixmap;
Keywords=image;viewer;picture;photo;gallery;
StartupNotify=true
"#,
        exe_path_str, icon_path_str
    );

    // 写入文件
    fs::write(&desktop_path, desktop_content)
        .with_context(|| format!("写入 .desktop 文件失败: {:?}", desktop_path))?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&desktop_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&desktop_path, perms)
            .with_context(|| format!("设置权限失败: {:?}", desktop_path))?;
    }

    // 更新桌面数据库
    update_desktop_database()?;

    Ok(())
}

/// 查找或创建图标
fn find_or_create_icon() -> Result<PathBuf> {
    let icon_path = get_icon_path();

    // 如果图标已存在，直接返回
    if icon_path.exists() {
        return Ok(icon_path);
    }

    // 尝试从可执行文件所在目录查找图标
    if let Ok(exe_path) = get_exe_path() {
        if let Some(exe_dir) = exe_path.parent() {
            let possible_icon = exe_dir.join("assets/icon.png");
            if possible_icon.exists() {
                // 确保目标目录存在
                if let Some(parent) = icon_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("创建图标目录失败: {:?}", parent))?;
                }

                // 复制图标
                fs::copy(&possible_icon, &icon_path).with_context(|| {
                    format!("复制图标失败: {:?} -> {:?}", possible_icon, icon_path)
                })?;

                return Ok(icon_path);
            }
        }
    }

    // 返回可执行文件路径作为图标（某些桌面环境支持）
    get_exe_path()
}

/// 更新桌面数据库
fn update_desktop_database() -> Result<()> {
    let apps_dir = dirs::home_dir()
        .map(|h| h.join(".local/share/applications"))
        .unwrap_or_else(|| PathBuf::from("~/.local/share/applications"));

    // 尝试使用 update-desktop-database
    let _ = Command::new("update-desktop-database")
        .arg(&apps_dir)
        .output();

    Ok(())
}

/// 设为默认图片查看器
pub fn set_as_default_app() -> Result<()> {
    // 首先确保 .desktop 文件存在
    create_desktop_file()?;

    // 使用 xdg-mime 设置默认应用
    for mime_type in IMAGE_MIME_TYPES {
        let output = Command::new("xdg-mime")
            .args(["default", "oas-image-viewer.desktop", mime_type])
            .output()
            .with_context(|| format!("设置默认应用失败: {}", mime_type))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "xdg-mime failed for {}: {}",
                mime_type,
                stderr
            ));
        }
    }

    Ok(())
}

/// 检查 xdg-mime 是否可用
#[allow(dead_code)]
fn xdg_mime_available() -> bool {
    Command::new("xdg-mime").arg("--version").output().is_ok()
}

/// 获取用户主目录（兼容不同配置）
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
    }
}
