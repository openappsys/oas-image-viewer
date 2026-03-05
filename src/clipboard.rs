//! 剪贴板操作模块 - 提供图片和文本的复制功能

use arboard::{Clipboard, ImageData};
use std::path::Path;
use tracing::{debug, error, info};

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

/// 剪贴板管理器
pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    /// 创建新的剪贴板管理器
    pub fn new() -> Self {
        match Clipboard::new() {
            Ok(clipboard) => {
                debug!("剪贴板初始化成功");
                Self {
                    clipboard: Some(clipboard),
                }
            }
            Err(e) => {
                error!("无法初始化剪贴板: {}", e);
                Self { clipboard: None }
            }
        }
    }

    /// 检查剪贴板是否可用
    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }

    /// 复制文本到剪贴板
    pub fn copy_text(&mut self, text: &str) -> Result<()> {
        let clipboard = self
            .clipboard
            .as_mut()
            .ok_or_else(|| ClipboardError::FailedToAccess("剪贴板不可用".to_string()))?;

        clipboard
            .set_text(text)
            .map_err(|e| ClipboardError::FailedToCopy(e.to_string()))?;

        info!("文本已复制到剪贴板");
        Ok(())
    }

    /// 复制图片路径到剪贴板
    pub fn copy_image_path(&mut self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        self.copy_text(&path_str)?;
        info!("图片路径已复制: {:?}", path);
        Ok(())
    }

    /// 复制图片数据到剪贴板
    pub fn copy_image(&mut self, image_data: &[u8], width: usize, height: usize) -> Result<()> {
        let clipboard = self
            .clipboard
            .as_mut()
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
            .set_image(image_data)
            .map_err(|e| ClipboardError::FailedToCopy(e.to_string()))?;

        info!("图片已复制到剪贴板 ({}x{})", width, height);
        Ok(())
    }

    /// 从文件路径复制图片到剪贴板
    pub fn copy_image_from_file(&mut self, path: &Path) -> Result<()> {
        // 从文件读取
        let img = image::open(path)
            .map_err(|e| ClipboardError::InvalidImage(format!("无法打开图片: {}", e)))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        self.copy_image(&data, width as usize, height as usize)
    }

    /// 在文件管理器中显示文件
    pub fn show_in_folder(path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open")
                .args(["-R", &path.to_string_lossy().to_string()])
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
                .args(["/select,", &path.to_string_lossy().to_string()])
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
