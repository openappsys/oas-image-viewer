//! Linux 平台系统集成实现
//!
//! 使用 xdg-mime 和 .desktop 文件实现：
//! - 设置为默认图片查看器
//! - 添加到右键菜单
//! - 从右键菜单移除

use super::SystemIntegration;
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::Language;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Linux 系统集成实现
pub struct LinuxIntegration;

/// 支持的图片 MIME 类型列表
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

/// 桌面文件名
const DESKTOP_FILENAME: &str = "oas-image-viewer.desktop";

impl LinuxIntegration {
    /// 创建新的 Linux 集成实例
    pub fn new() -> Self {
        Self
    }

    /// 获取 .desktop 文件完整路径
    fn get_desktop_file_path(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        home.join(".local/share/applications")
            .join(DESKTOP_FILENAME)
    }

    /// 获取当前可执行文件路径
    fn get_exe_path(&self) -> Result<PathBuf> {
        std::env::current_exe().context("Failed to get executable path")
    }

    /// 生成基础 .desktop 文件内容（无 Actions）
    fn generate_base_desktop_content(&self, exe_path: &str) -> String {
        format!(
            r#"[Desktop Entry]
Name=OAS Image Viewer
Exec={} %F
Icon=oas-image-viewer
Type=Application
MimeType=image/png;image/jpeg;image/gif;image/webp;image/tiff;image/bmp;
Categories=Graphics;Viewer;
Terminal=false
"#,
            exe_path
        )
    }

    /// 生成带 Actions 的 .desktop 文件内容
    fn generate_desktop_content_with_actions(&self, exe_path: &str) -> String {
        format!(
            r#"[Desktop Entry]
Name=OAS Image Viewer
Exec={} %F
Icon=oas-image-viewer
Type=Application
MimeType=image/png;image/jpeg;image/gif;image/webp;image/tiff;image/bmp;
Categories=Graphics;Viewer;
Terminal=false
Actions=open-with-oas;

[Desktop Action open-with-oas]
Name=Open with OAS Image Viewer
Exec={} %f
"#,
            exe_path, exe_path
        )
    }

    /// 确保 .desktop 文件父目录存在
    fn ensure_desktop_dir_exists(&self, language: Language) -> Result<()> {
        let desktop_path = self.get_desktop_file_path();
        if let Some(parent) = desktop_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| get_text("error_desktop_file_write", language).to_string())?;
        }
        Ok(())
    }

    /// 更新桌面数据库
    fn update_desktop_database(&self) -> Result<()> {
        let apps_dir = dirs::home_dir()
            .map(|h| h.join(".local/share/applications"))
            .unwrap_or_else(|| PathBuf::from("~/.local/share/applications"));

        // 尝试使用 update-desktop-database，失败也不报错
        let _ = Command::new("update-desktop-database")
            .arg(&apps_dir)
            .output();

        Ok(())
    }
}

impl Default for LinuxIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemIntegration for LinuxIntegration {
    /// 设置为默认图片查看器
    ///
    /// 使用 xdg-mime 命令将 oas-image-viewer.desktop 设为所有支持图片格式的默认应用
    fn set_as_default(&self, language: Language) -> Result<()> {
        // 首先确保 .desktop 文件存在（基础版本）
        let exe_path = self.get_exe_path()?;
        let exe_path_str = exe_path.to_string_lossy();

        self.ensure_desktop_dir_exists(language)?;

        let desktop_path = self.get_desktop_file_path();

        // 如果文件不存在，创建基础版本
        if !desktop_path.exists() {
            let content = self.generate_base_desktop_content(&exe_path_str);
            fs::write(&desktop_path, content)
                .with_context(|| get_text("error_desktop_file_write", language).to_string())?;

            // 设置可执行权限
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&desktop_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&desktop_path, perms)
                    .with_context(|| get_text("error_desktop_file_write", language).to_string())?;
            }

            self.update_desktop_database()?;
        }

        // 使用 xdg-mime 设置默认应用
        for mime_type in IMAGE_MIME_TYPES {
            let output = Command::new("xdg-mime")
                .args(["default", DESKTOP_FILENAME, mime_type])
                .output()
                .with_context(|| format!("xdg-mime failed for {}", mime_type))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!(
                    "{}",
                    get_text("error_xdg_mime_failed", language)
                        .replace("{}", &format!("{}: {}", mime_type, stderr))
                );
            }
        }

        Ok(())
    }

    /// 取消默认图片查看器
    fn unset_default(&self, language: Language) -> Result<()> {
        anyhow::bail!("{}", get_text("unset_default_not_supported", language))
    }

    /// 添加到右键菜单
    ///
    /// 通过添加 Desktop Action 到 .desktop 文件实现右键菜单集成
    fn add_context_menu(&self, language: Language) -> Result<()> {
        let exe_path = self.get_exe_path()?;
        let exe_path_str = exe_path.to_string_lossy();

        self.ensure_desktop_dir_exists(language)?;

        let desktop_path = self.get_desktop_file_path();

        // 生成带 Actions 的内容
        let content = self.generate_desktop_content_with_actions(&exe_path_str);

        // 写入文件
        fs::write(&desktop_path, content)
            .with_context(|| get_text("error_desktop_file_write", language).to_string())?;

        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&desktop_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&desktop_path, perms)
                .with_context(|| get_text("error_desktop_file_write", language).to_string())?;
        }

        self.update_desktop_database()?;

        Ok(())
    }

    /// 从右键菜单移除
    ///
    /// 通过移除 Desktop Actions 部分实现
    fn remove_context_menu(&self, language: Language) -> Result<()> {
        let desktop_path = self.get_desktop_file_path();

        // 如果文件不存在，直接返回成功
        if !desktop_path.exists() {
            return Ok(());
        }

        // 读取当前内容
        let content = fs::read_to_string(&desktop_path)
            .with_context(|| get_text("error_desktop_file_read", language).to_string())?;

        // 查找 Actions 部分的起始位置
        if let Some(actions_pos) = content.find("\n[Desktop Action") {
            // 截取基础部分（不含 Actions）
            let base_content = &content[..actions_pos];
            fs::write(&desktop_path, base_content)
                .with_context(|| get_text("error_desktop_file_write", language).to_string())?;
        } else {
            // 如果没有 Actions，检查是否有 Actions= 行
            let lines: Vec<&str> = content.lines().collect();
            let filtered: Vec<&str> = lines
                .into_iter()
                .filter(|line| !line.starts_with("Actions="))
                .collect();
            let new_content = filtered.join("\n");
            fs::write(&desktop_path, new_content)
                .with_context(|| get_text("error_desktop_file_write", language).to_string())?;
        }

        self.update_desktop_database()?;

        Ok(())
    }

    fn refresh_open_with_registration(&self, language: Language) -> Result<()> {
        anyhow::bail!("{}", get_text("refresh_open_with_not_supported", language))
    }

    /// 检查是否已是默认查看器
    ///
    /// 通过 xdg-mime query default 检查 image/png 的默认应用
    fn is_default(&self) -> bool {
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
}

/// 平台目录工具：获取用户主目录
mod dirs {
    use std::path::PathBuf;

    /// 获取用户主目录
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
    }
}
