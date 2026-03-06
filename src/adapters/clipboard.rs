//! 剪贴板操作模块 - 提供图片和文本的复制功能

use arboard::{Clipboard, ImageData};
use parking_lot::Mutex;
use std::path::Path;
use tracing::{debug, error, info};

use crate::core::ports::ClipboardPort;
use crate::core::Result as CoreResult;

/// 剪贴板操作结果
pub type Result<T> = std::result::Result<T, ClipboardError>;

/// 剪贴板错误类型
#[derive(Debug, Clone)]
pub enum ClipboardError {
    FailedToAccess(String),
    FailedToCopy(String),
    InvalidImage(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::FailedToAccess(msg) => write!(f, "无法访问剪贴板: {}", msg),
            ClipboardError::FailedToCopy(msg) => write!(f, "复制失败: {}", msg),
            ClipboardError::InvalidImage(msg) => write!(f, "无效的图片: {}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}

impl From<ClipboardError> for crate::core::CoreError {
    fn from(e: ClipboardError) -> Self {
        crate::core::CoreError::technical("STORAGE_ERROR", e.to_string())
    }
}

/// 剪贴板管理器
pub struct ClipboardManager {
    clipboard: Option<Mutex<Clipboard>>,
}

impl ClipboardManager {
    /// 创建新的剪贴板管理器
    pub fn new() -> Self {
        match Clipboard::new() {
            Ok(clipboard) => {
                debug!("剪贴板初始化成功");
                Self {
                    clipboard: Some(Mutex::new(clipboard)),
                }
            }
            Err(e) => {
                error!("无法初始化剪贴板: {}", e);
                Self { clipboard: None }
            }
        }
    }

    /// 复制文本到剪贴板
    pub fn copy_text(&self, text: &str) -> Result<()> {
        let clipboard = self
            .clipboard
            .as_ref()
            .ok_or_else(|| ClipboardError::FailedToAccess("剪贴板不可用".to_string()))?;

        clipboard
            .lock()
            .set_text(text)
            .map_err(|e| ClipboardError::FailedToCopy(e.to_string()))?;

        info!("文本已复制到剪贴板");
        Ok(())
    }

    /// 复制图片数据到剪贴板
    pub fn copy_image_data(&self, image_data: &[u8], width: usize, height: usize) -> Result<()> {
        let clipboard = self
            .clipboard
            .as_ref()
            .ok_or_else(|| ClipboardError::FailedToAccess("剪贴板不可用".to_string()))?;

        // 确保数据长度正确
        let expected_len = width * height * 4; // RGBA
        if image_data.len() != expected_len {
            return Err(ClipboardError::InvalidImage(format!(
                "图片数据长度不匹配: 期望 {} 字节, 实际 {} 字节",
                expected_len,
                image_data.len()
            )));
        }

        let image_data = ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Borrowed(image_data),
        };

        clipboard
            .lock()
            .set_image(image_data)
            .map_err(|e| ClipboardError::FailedToCopy(e.to_string()))?;

        info!("图片已复制到剪贴板 ({}x{})", width, height);
        Ok(())
    }

    /// 从文件路径复制图片到剪贴板
    pub fn copy_image_from_file(&self, path: &Path) -> Result<()> {
        // 从文件读取
        let img = image::open(path)
            .map_err(|e| ClipboardError::InvalidImage(format!("无法打开图片: {}", e)))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        self.copy_image_data(&data, width as usize, height as usize)
    }
}

impl ClipboardPort for ClipboardManager {
    fn copy_image(&self, width: usize, height: usize, data: &[u8]) -> CoreResult<()> {
        self.copy_image_data(data, width, height)
            .map_err(|e| e.into())
    }

    fn copy_path(&self, path: &Path) -> CoreResult<()> {
        let path_str = path.to_string_lossy().to_string();
        self.copy_text(&path_str)
            .map_err(|e| -> crate::core::CoreError { e.into() })?;
        info!("图片路径已复制: {:?}", path);
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }

    fn show_in_folder(&self, path: &Path) -> CoreResult<()> {
        Self::show_in_folder_impl(path).map_err(|e| e.into())
    }
}

impl ClipboardManager {
    /// 在文件管理器中显示文件（内部实现）
    fn show_in_folder_impl(path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let path_str = path.to_string_lossy();
            Command::new("open")
                .args(["-R", path_str.as_ref()])
                .spawn()
                .map_err(|e| ClipboardError::FailedToCopy(format!("无法打开文件夹: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            let path_str = path.to_string_lossy().to_string();
            let parent = path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone());

            // 尝试 xdg-open
            let result = Command::new("xdg-open").arg(&parent).spawn();
            if result.is_err() {
                // 回退到 dbus-send (Nautilus)
                let _ = Command::new("dbus-send")
                    .args([
                        "--session",
                        "--dest=org.freedesktop.FileManager1",
                        "--type=method_call",
                        "/org/freedesktop/FileManager1",
                        "org.freedesktop.FileManager1.ShowItems",
                        format!("array:string:file://{}", path_str).as_str(),
                        "string:\"\"",
                    ])
                    .spawn();
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("explorer")
                .args(["/select,", &path.to_string_lossy().as_ref()])
                .spawn()
                .map_err(|e| ClipboardError::FailedToCopy(format!("无法打开文件夹: {}", e)))?;
        }

        info!("已在文件夹中显示: {:?}", path);
        Ok(())
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_manager_new() {
        let manager = ClipboardManager::new();
        // 在某些环境（如 CI）中剪贴板可能不可用
        // 但不应该 panic
        let _ = manager.is_available();
    }

    #[test]
    fn test_clipboard_manager_default() {
        let manager: ClipboardManager = Default::default();
        let _ = manager.is_available();
    }
}
