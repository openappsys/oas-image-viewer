//! Windows 平台系统集成实现

use super::SystemIntegration;
use anyhow::{Context, Result};
use std::path::PathBuf;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

/// 支持的图片扩展名列表
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"];

/// 应用程序 ProgID
const PROG_ID: &str = "oas-image-viewer";

/// 应用程序显示名称
const APP_DISPLAY_NAME: &str = "OAS Image Viewer";

/// Windows 系统集成实现
pub struct WindowsIntegration;

impl WindowsIntegration {
    /// 创建新的 Windows 集成实例
    pub fn new() -> Self {
        Self
    }

    /// 获取当前可执行文件路径
    fn get_exe_path(&self) -> Result<PathBuf> {
        std::env::current_exe().context("无法获取可执行文件路径")
    }
}

impl Default for WindowsIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemIntegration for WindowsIntegration {
    /// 设置为默认图片查看器
    fn set_as_default(&self) -> anyhow::Result<()> {
        let exe_path = self.get_exe_path()?;
        let exe_path_str = exe_path.to_string_lossy();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 创建 ProgID 注册表项
        let prog_id_path = format!(r"Software\Classes\{}", PROG_ID);
        let (prog_id_key, _) = hkcu
            .create_subkey(&prog_id_path)
            .with_context(|| format!("创建 ProgID 注册表项失败: {}", prog_id_path))?;

        // 设置 ProgID 显示名称
        prog_id_key
            .set_value("", &APP_DISPLAY_NAME)
            .context("设置 ProgID 显示名称失败")?;

        // 创建 DefaultIcon 子项
        let default_icon_path = format!(r"{}\DefaultIcon", prog_id_path);
        let (icon_key, _) = hkcu
            .create_subkey(&default_icon_path)
            .context("创建 DefaultIcon 子项失败")?;
        icon_key
            .set_value("", &exe_path_str.as_ref())
            .context("设置图标路径失败")?;

        // 创建 shell\open\command 子项
        let command_path = format!(r"{}\shell\open\command", prog_id_path);
        let (cmd_key, _) = hkcu
            .create_subkey(&command_path)
            .context("创建 command 子项失败")?;

        // 设置打开命令
        let command = format!(r#""{}" "%1""#, exe_path_str);
        cmd_key
            .set_value("", &command)
            .context("设置打开命令失败")?;

        // 关联图片格式扩展名
        for ext in IMAGE_EXTENSIONS {
            let ext_path = format!(r"Software\Classes\.{}", ext);
            let (ext_key, _) = hkcu
                .create_subkey(&ext_path)
                .with_context(|| format!("创建扩展名注册表项失败: {}", ext))?;

            // 设置默认值为 ProgID
            ext_key
                .set_value("", &PROG_ID)
                .with_context(|| format!("设置扩展名关联失败: {}", ext))?;
        }

        Ok(())
    }

    /// 添加到右键菜单
    fn add_context_menu(&self) -> anyhow::Result<()> {
        let exe_path = self.get_exe_path()?;
        let exe_path_str = exe_path.to_string_lossy();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 创建右键菜单项（对所有文件类型）
        let shell_path = r"Software\Classes\*\shell\Open with OAS Image Viewer";
        let (shell_key, _) = hkcu
            .create_subkey(shell_path)
            .context("创建右键菜单注册表项失败")?;

        // 设置菜单显示名称
        shell_key
            .set_value("", &"Open with OAS Image Viewer")
            .context("设置右键菜单显示名称失败")?;

        // 设置菜单图标
        shell_key
            .set_value("Icon", &exe_path_str.as_ref())
            .context("设置右键菜单图标失败")?;

        // 创建 command 子项
        let command_path = format!(r"{}\command", shell_path);
        let (cmd_key, _) = hkcu
            .create_subkey(&command_path)
            .context("创建右键菜单命令子项失败")?;

        // 设置命令
        let command = format!(r#""{}" "%1""#, exe_path_str);
        cmd_key
            .set_value("", &command)
            .context("设置右键菜单命令失败")?;

        // 同时针对图片格式添加特定的右键菜单
        for ext in IMAGE_EXTENSIONS {
            let ext_shell_path = format!(
                r"Software\Classes\SystemFileAssociations\.{ext}\shell\Open with OAS Image Viewer",
            );
            let (ext_shell_key, _) = hkcu
                .create_subkey(&ext_shell_path)
                .with_context(|| format!("创建图片格式右键菜单项失败: {}", ext))?;

            ext_shell_key
                .set_value("", &"Open with OAS Image Viewer")
                .with_context(|| format!("设置图片格式右键菜单名称失败: {}", ext))?;

            ext_shell_key
                .set_value("Icon", &exe_path_str.as_ref())
                .with_context(|| format!("设置图片格式右键菜单图标失败: {}", ext))?;

            let ext_cmd_path = format!(r"{}\command", ext_shell_path);
            let (ext_cmd_key, _) = hkcu
                .create_subkey(&ext_cmd_path)
                .with_context(|| format!("创建图片格式右键菜单命令失败: {}", ext))?;

            ext_cmd_key
                .set_value("", &command)
                .with_context(|| format!("设置图片格式右键菜单命令失败: {}", ext))?;
        }

        Ok(())
    }

    /// 从右键菜单移除
    fn remove_context_menu(&self) -> anyhow::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 删除通用右键菜单项
        let shell_path = r"Software\Classes\*\shell\Open with OAS Image Viewer";
        let _ = hkcu.delete_subkey_all(shell_path);

        // 删除图片格式特定的右键菜单项
        for ext in IMAGE_EXTENSIONS {
            let ext_shell_path = format!(
                r"Software\Classes\SystemFileAssociations\.{ext}\shell\Open with OAS Image Viewer",
            );
            let _ = hkcu.delete_subkey_all(&ext_shell_path);
        }

        Ok(())
    }

    /// 检查是否已是默认查看器
    fn is_default(&self) -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 检查第一个图片扩展名的默认打开方式
        let first_ext = IMAGE_EXTENSIONS[0];
        let user_choice_path = format!(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{first_ext}\UserChoice"
        );

        match hkcu.open_subkey(&user_choice_path) {
            Ok(key) => match key.get_value::<String, _>("ProgId") {
                Ok(prog_id) => prog_id == PROG_ID,
                Err(_) => false,
            },
            Err(_) => {
                // 如果 UserChoice 不存在，检查 Classes 中的默认关联
                let ext_path = format!(r"Software\Classes\.{}", first_ext);
                match hkcu.open_subkey(&ext_path) {
                    Ok(ext_key) => match ext_key.get_value::<String, _>("") {
                        Ok(value) => value == PROG_ID,
                        Err(_) => false,
                    },
                    Err(_) => false,
                }
            }
        }
    }
}
