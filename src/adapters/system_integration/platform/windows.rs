//! Windows 平台系统集成实现

use super::IntegrationStatus;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// 支持的图片扩展名列表
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp", "ico", "heic", "heif", "avif",
];

/// 注册表路径前缀
const REGISTRY_BASE_PATH: &str = r"Software\Classes\SystemFileAssociations";

/// 获取当前可执行文件路径
fn get_exe_path() -> Result<PathBuf> {
    std::env::current_exe().context("无法获取可执行文件路径")
}

/// 获取系统集成状态
pub fn get_integration_status() -> IntegrationStatus {
    IntegrationStatus {
        context_menu_registered: is_context_menu_registered(),
        default_app_registered: is_default_app_registered(),
    }
}

/// 检查右键菜单是否已注册
fn is_context_menu_registered() -> bool {
    // 检查第一个扩展名的注册表项是否存在
    let first_ext = IMAGE_EXTENSIONS[0];
    let reg_path = format!(
        r"{}\.{}\shell\OpenWithOASImageViewer",
        REGISTRY_BASE_PATH, first_ext
    );

    unsafe {
        match winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).open_subkey(&reg_path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// 检查是否已设为默认图片查看器
fn is_default_app_registered() -> bool {
    // 检查 .png 文件的默认打开方式
    let reg_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.png\UserChoice";

    unsafe {
        match winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).open_subkey(reg_path) {
            Ok(key) => match key.get_value::<String>("ProgId") {
                Ok(prog_id) => prog_id.contains("OASImageViewer"),
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

/// 注册右键菜单
pub fn register_context_menu() -> Result<()> {
    let exe_path = get_exe_path()?;
    let exe_path_str = exe_path.to_string_lossy();

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

    for ext in IMAGE_EXTENSIONS {
        let shell_path = format!(
            r"{}\.{}\shell\OpenWithOASImageViewer",
            REGISTRY_BASE_PATH, ext
        );

        // 创建 shell 项
        let (key, _) = hkcu
            .create_subkey(&shell_path)
            .with_context(|| format!("创建注册表项失败: {}", shell_path))?;

        // 设置显示名称
        key.set_value("", &"Open with OAS Image Viewer")
            .with_context(|| format!("设置显示名称失败: {}", shell_path))?;

        // 设置图标
        key.set_value("Icon", &exe_path_str.as_ref())
            .with_context(|| format!("设置图标失败: {}", shell_path))?;

        // 创建 command 子项
        let (cmd_key, _) = hkcu
            .create_subkey(format!(r"{}\command", shell_path))
            .with_context(|| format!("创建 command 子项失败: {}", shell_path))?;

        // 设置命令
        let command = format!(r#""{}" "%1""#, exe_path_str);
        cmd_key
            .set_value("", &command)
            .with_context(|| format!("设置命令失败: {}", shell_path))?;
    }

    Ok(())
}

/// 注销右键菜单
pub fn unregister_context_menu() -> Result<()> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

    for ext in IMAGE_EXTENSIONS {
        let shell_path = format!(
            r"{}\.{}\shell\OpenWithOASImageViewer",
            REGISTRY_BASE_PATH, ext
        );

        // 删除整个 shell 项树
        let _ = hkcu.delete_subkey_all(&shell_path);
    }

    Ok(())
}

/// 设为默认图片查看器
pub fn set_as_default_app() -> Result<()> {
    let exe_path = get_exe_path()?;
    let exe_path_str = exe_path.to_string_lossy();

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

    // 创建应用程序注册表项
    let app_path = r"Software\Classes\OASImageViewer.Application";
    let (app_key, _) = hkcu
        .create_subkey(app_path)
        .context("创建应用程序注册表项失败")?;

    app_key
        .set_value("", &"OAS Image Viewer Application")
        .context("设置应用程序名称失败")?;

    // 创建 DefaultIcon
    let (icon_key, _) = hkcu
        .create_subkey(format!(r"{}\DefaultIcon", app_path))
        .context("创建 DefaultIcon 项失败")?;
    icon_key
        .set_value("", &exe_path_str.as_ref())
        .context("设置图标路径失败")?;

    // 创建 shell\open\command
    let (cmd_key, _) = hkcu
        .create_subkey(format!(r"{}\shell\open\command", app_path))
        .context("创建 command 项失败")?;
    let command = format!(r#""{}" "%1""#, exe_path_str);
    cmd_key.set_value("", &command).context("设置命令失败")?;

    // 关联文件扩展名
    for ext in IMAGE_EXTENSIONS {
        let ext_path = format!(r"Software\Classes\.{}", ext);
        let (ext_key, _) = hkcu
            .create_subkey(&ext_path)
            .with_context(|| format!("创建扩展名注册表项失败: {}", ext))?;

        // 设置打开方式
        ext_key
            .set_value("", &"OASImageViewer.Application")
            .with_context(|| format!("设置打开方式失败: {}", ext))?;
    }

    Ok(())
}
