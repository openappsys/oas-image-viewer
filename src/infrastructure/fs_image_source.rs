use crate::core::domain::{Image, ImageMetadata};
use crate::core::ports::ImageSource;
use crate::core::{CoreError, Result};
use std::path::{Path, PathBuf};

/// 文件系统图像源实现
pub struct FsImageSource;

impl FsImageSource {
    /// 创建新的文件系统图像源
    pub fn new() -> Self {
        Self
    }

    /// 支持的图像扩展名
    pub(crate) const SUPPORTED_EXTENSIONS: &[&str] =
        &["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"];
}

impl Default for FsImageSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageSource for FsImageSource {
    fn load_metadata(&self, path: &Path) -> Result<ImageMetadata> {
        if !path.exists() {
            return Err(CoreError::technical(
                "IMAGE_NOT_FOUND",
                path.to_string_lossy().to_string(),
            ));
        }

        let metadata = std::fs::metadata(path).map_err(|e| {
            CoreError::technical("STORAGE_ERROR", format!("Failed to read metadata: {}", e))
        })?;

        let file_size = metadata.len();
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let created_at = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let (width, height) = match self.load_image_data(path) {
            Ok((w, h, _)) => (w, h),
            Err(_) => (0, 0),
        };

        let format = Image::detect_format(path);

        Ok(ImageMetadata {
            width,
            height,
            format,
            file_size,
            created_at,
            modified_at,
        })
    }

    fn load_image_data(&self, path: &Path) -> Result<(u32, u32, Vec<u8>)> {
        let img_result = image::open(path);

        let img = match img_result {
            Ok(img) => img,
            Err(_) => {
                let data = std::fs::read(path).map_err(|e| {
                    CoreError::technical("STORAGE_ERROR", format!("Failed to read file: {}", e))
                })?;

                image::load_from_memory(&data).map_err(|e| {
                    CoreError::technical(
                        "INVALID_IMAGE_FORMAT",
                        format!("Failed to decode image: {}", e),
                    )
                })?
            }
        };

        let width = img.width();
        let height = img.height();
        let rgba = img.to_rgba8();
        let data = rgba.as_raw().clone();

        Ok((width, height, data))
    }

    fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        if !path.is_dir() {
            return Err(CoreError::technical(
                "STORAGE_ERROR",
                format!("Not a directory: {}", path.display()),
            ));
        }

        let mut images: Vec<PathBuf> = std::fs::read_dir(path)
            .map_err(|e| {
                CoreError::technical("STORAGE_ERROR", format!("Failed to read directory: {}", e))
            })?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| self.is_supported(p))
            .collect();

        images.sort();
        Ok(images)
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                let ext = e.to_lowercase();
                Self::SUPPORTED_EXTENSIONS.contains(&ext.as_str())
            })
            .unwrap_or(false)
    }

    fn generate_thumbnail(&self, path: &Path, max_size: u32) -> Result<(u32, u32, Vec<u8>)> {
        let (width, height, data) = self.load_image_data(path)?;

        if width <= max_size && height <= max_size {
            return Ok((width, height, data));
        }

        let img = image::open(path).map_err(|e| {
            CoreError::technical("INVALID_IMAGE_FORMAT", format!("Failed to open: {}", e))
        })?;

        let resized = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let new_width = rgba.width();
        let new_height = rgba.height();
        let new_data = rgba.as_raw().clone();

        Ok((new_width, new_height, new_data))
    }
}
