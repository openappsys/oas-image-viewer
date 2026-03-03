//! Infrastructure 层 - 技术实现
//!
//! 实现 Core 层定义的端口接口

use crate::core::domain::ImageMetadata;
use crate::core::ports::{
    AppConfig, AsyncImageSource, FileDialogPort, ImageLoadedCallback, ImageSource, Storage,
    ThumbnailLoadedCallback,
};
use crate::core::{CoreError, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// 文件系统图像源实现
pub struct FsImageSource;

/// JSON 配置存储实现
pub struct JsonStorage {
    config_path: PathBuf,
    save_tx: Option<Sender<AppConfig>>,
}

/// 异步文件系统图像源
pub struct AsyncFsImageSource {
    inner: FsImageSource,
}

impl FsImageSource {
    /// 创建新的文件系统图像源
    pub fn new() -> Self {
        Self
    }

    /// 支持的图像扩展名
    const SUPPORTED_EXTENSIONS: &[&str] =
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
            return Err(CoreError::ImageNotFound(path.to_string_lossy().to_string()));
        }

        // 获取文件信息
        let metadata = std::fs::metadata(path)
            .map_err(|e| CoreError::StorageError(format!("Failed to read metadata: {}", e)))?;

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

        // 尝试加载图像获取尺寸
        let format = Image::detect_format(path);
        let (width, height) = if format.is_supported() {
            match self.load_image_data(path) {
                Ok((w, h, _)) => (w, h),
                Err(_) => (0, 0),
            }
        } else {
            (0, 0)
        };

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
        // 首先尝试使用 image::open 加载
        let img_result = image::open(path);

        let img = match img_result {
            Ok(img) => img,
            Err(_e) => {
                // 备用方法：从内存加载
                let data = std::fs::read(path)
                    .map_err(|e| CoreError::StorageError(format!("Failed to read file: {}", e)))?;

                image::load_from_memory(&data).map_err(|e| {
                    CoreError::InvalidImageFormat(format!("Failed to decode image: {}", e))
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
            return Err(CoreError::StorageError(format!(
                "Not a directory: {}",
                path.display()
            )));
        }

        let mut images: Vec<PathBuf> = std::fs::read_dir(path)
            .map_err(|e| CoreError::StorageError(format!("Failed to read directory: {}", e)))?
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

        // 如果图像已经小于最大尺寸，直接返回
        if width <= max_size && height <= max_size {
            return Ok((width, height, data));
        }

        // 需要缩放，使用 image crate
        let img = image::open(path)
            .map_err(|e| CoreError::InvalidImageFormat(format!("Failed to open: {}", e)))?;

        let resized = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);

        let rgba = resized.to_rgba8();
        let new_width = rgba.width();
        let new_height = rgba.height();
        let new_data = rgba.as_raw().clone();

        Ok((new_width, new_height, new_data))
    }
}

// 引入 Image 用于 detect_format
use crate::core::domain::Image;

impl JsonStorage {
    /// 创建新的 JSON 存储
    pub fn new() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| CoreError::StorageError(format!("Failed to create config dir: {}", e)))?;

        let config_path = config_dir.join("config.json");

        Ok(Self {
            config_path,
            save_tx: None,
        })
    }

    /// 获取配置目录
    fn config_dir() -> Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "imageviewer", "image-viewer")
            .ok_or_else(|| CoreError::StorageError("Failed to get project dirs".to_string()))?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }

    /// 启动防抖保存线程
    pub fn with_debounce(mut self) -> Self {
        let (tx, rx): (Sender<AppConfig>, Receiver<AppConfig>) = channel();
        let config_path = self.config_path.clone();

        thread::spawn(move || {
            use std::time::{Duration, Instant};

            const DEBOUNCE_MS: u64 = 500;
            let mut last_save = Instant::now();
            let mut pending: Option<AppConfig> = None;

            loop {
                match rx.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
                    Ok(config) => {
                        pending = Some(config);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(config) = pending.take() {
                            if last_save.elapsed().as_millis() >= 100 {
                                let _ = Self::save_to_file(&config_path, &config);
                                last_save = Instant::now();
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        self.save_tx = Some(tx);
        self
    }

    /// 保存配置到文件
    fn save_to_file(path: &Path, config: &AppConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| CoreError::StorageError(format!("Failed to serialize: {}", e)))?;

        // 原子写入
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, json)
            .map_err(|e| CoreError::StorageError(format!("Failed to write: {}", e)))?;

        std::fs::rename(&temp_path, path)
            .map_err(|e| CoreError::StorageError(format!("Failed to rename: {}", e)))?;

        Ok(())
    }

    /// 从文件加载配置
    fn load_from_file(path: &Path) -> Result<AppConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CoreError::StorageError(format!("Failed to read: {}", e)))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| CoreError::StorageError(format!("Failed to parse: {}", e)))?;

        Ok(config)
    }
}

impl Default for JsonStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create JsonStorage")
    }
}

impl Storage for JsonStorage {
    fn load_config(&self) -> Result<AppConfig> {
        if self.config_path.exists() {
            Self::load_from_file(&self.config_path)
        } else {
            Ok(AppConfig::default())
        }
    }

    fn save_config(&self, config: &AppConfig) -> Result<()> {
        Self::save_to_file(&self.config_path, config)
    }

    fn request_save(&self, config: &AppConfig) -> Result<()> {
        if let Some(ref tx) = self.save_tx {
            tx.send(config.clone())
                .map_err(|_| CoreError::StorageError("Save channel closed".to_string()))?;
            Ok(())
        } else {
            self.save_config(config)
        }
    }
}

impl AsyncFsImageSource {
    /// 创建新的异步图像源
    pub fn new() -> Self {
        Self {
            inner: FsImageSource::new(),
        }
    }
}

impl Default for AsyncFsImageSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageSource for AsyncFsImageSource {
    fn load_metadata(&self, path: &Path) -> Result<ImageMetadata> {
        self.inner.load_metadata(path)
    }

    fn load_image_data(&self, path: &Path) -> Result<(u32, u32, Vec<u8>)> {
        self.inner.load_image_data(path)
    }

    fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        self.inner.scan_directory(path)
    }

    fn is_supported(&self, path: &Path) -> bool {
        self.inner.is_supported(path)
    }

    fn generate_thumbnail(&self, path: &Path, max_size: u32) -> Result<(u32, u32, Vec<u8>)> {
        self.inner.generate_thumbnail(path, max_size)
    }
}

impl AsyncImageSource for AsyncFsImageSource {
    fn load_image_async(&self, path: &Path, callback: ImageLoadedCallback) {
        let path = path.to_path_buf();
        let inner = FsImageSource::new();

        thread::spawn(move || {
            let metadata = inner.load_metadata(&path);
            let result = metadata.map(|m| {
                let mut image = Image::new(
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown"),
                    &path,
                );
                image.set_metadata(m);
                image
            });
            callback(result);
        });
    }

    fn generate_thumbnail_async(
        &self,
        path: &Path,
        max_size: u32,
        _index: usize,
        callback: ThumbnailLoadedCallback,
    ) {
        let path = path.to_path_buf();
        let inner = FsImageSource::new();

        thread::spawn(move || {
            let result = inner
                .generate_thumbnail(&path, max_size)
                .map(|(_, _, data)| data);
            callback(_index, result);
        });
    }
}

/// RFD 文件对话框实现
pub struct RfdFileDialog;

impl RfdFileDialog {
    /// 创建新的文件对话框
    pub fn new() -> Self {
        Self
    }
}

impl Default for RfdFileDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDialogPort for RfdFileDialog {
    fn open_files(&self) -> Option<Vec<PathBuf>> {
        rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"],
            )
            .add_filter("All Files", &["*"])
            .pick_files()
    }

    fn open_directory(&self) -> Option<PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }
}

// 引入 serde 依赖
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_image_source_is_supported() {
        let source = FsImageSource::new();
        assert!(source.is_supported(Path::new("test.png")));
        assert!(source.is_supported(Path::new("test.PNG")));
        assert!(source.is_supported(Path::new("test.jpg")));
        assert!(!source.is_supported(Path::new("test.txt")));
        assert!(!source.is_supported(Path::new("test")));
    }

    #[test]
    fn test_fs_image_source_supported_extensions() {
        assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(&"png"));
        assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(&"jpg"));
        assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(&"webp"));
    }

    #[test]
    fn test_json_storage_default() {
        // 只是测试能创建
        let storage = JsonStorage::new();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_rfd_file_dialog() {
        let dialog = RfdFileDialog::new();
        // 只是测试能创建
        drop(dialog);
    }

    // =========================================================================
    // 从旧代码迁移的额外测试
    // =========================================================================

    #[test]
    fn test_fs_image_source_new() {
        let source = FsImageSource::new();
        drop(source);
    }

    #[test]
    fn test_fs_image_source_default() {
        let source: FsImageSource = Default::default();
        drop(source);
    }

    #[test]
    fn test_fs_image_source_is_supported_various() {
        let source = FsImageSource::new();

        // 支持的格式
        assert!(source.is_supported(Path::new("image.png")));
        assert!(source.is_supported(Path::new("image.jpg")));
        assert!(source.is_supported(Path::new("image.jpeg")));
        assert!(source.is_supported(Path::new("image.gif")));
        assert!(source.is_supported(Path::new("image.webp")));
        assert!(source.is_supported(Path::new("image.tiff")));
        assert!(source.is_supported(Path::new("image.tif")));
        assert!(source.is_supported(Path::new("image.bmp")));

        // 不支持的格式
        assert!(!source.is_supported(Path::new("image.txt")));
        assert!(!source.is_supported(Path::new("image.rs")));
        assert!(!source.is_supported(Path::new("image.pdf")));
        assert!(!source.is_supported(Path::new("image.zip")));
        assert!(!source.is_supported(Path::new("image")));
        assert!(!source.is_supported(Path::new("")));
    }

    #[test]
    fn test_fs_image_source_is_supported_case_insensitive() {
        let source = FsImageSource::new();
        // is_supported 使用小写比较
        assert!(source.is_supported(Path::new("image.PNG")));
        assert!(source.is_supported(Path::new("image.JPG")));
        assert!(source.is_supported(Path::new("image.JPEG")));
        assert!(source.is_supported(Path::new("image.GIF")));
        assert!(source.is_supported(Path::new("image.WEBP")));
        assert!(source.is_supported(Path::new("image.TIFF")));
        assert!(source.is_supported(Path::new("image.BMP")));
    }

    #[test]
    fn test_fs_image_source_is_supported_with_paths() {
        let source = FsImageSource::new();
        assert!(source.is_supported(Path::new("/path/to/image.png")));
        assert!(source.is_supported(Path::new("./relative/path/image.jpg")));
        assert!(source.is_supported(Path::new("C:\\Users\\image.gif")));
    }

    #[test]
    fn test_fs_image_source_is_supported_dots_in_name() {
        let source = FsImageSource::new();
        assert!(source.is_supported(Path::new("my.image.file.png")));
        assert!(source.is_supported(Path::new("archive.v2.jpg")));
        // 非图像扩展名
        assert!(!source.is_supported(Path::new("archive.tar.gz")));
    }

    #[test]
    fn test_supported_extensions_count() {
        assert_eq!(FsImageSource::SUPPORTED_EXTENSIONS.len(), 8);
    }

    #[test]
    fn test_supported_extensions_all_present() {
        let expected = vec!["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"];
        for ext in &expected {
            assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(ext));
        }
    }

    #[test]
    fn test_json_storage_new_success() {
        let storage = JsonStorage::new();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_async_fs_image_source_new() {
        let source = AsyncFsImageSource::new();
        drop(source);
    }

    #[test]
    fn test_rfd_file_dialog_new() {
        let dialog = RfdFileDialog::new();
        drop(dialog);
    }

    #[test]
    fn test_rfd_file_dialog_default() {
        let dialog: RfdFileDialog = Default::default();
        drop(dialog);
    }

    #[test]
    fn test_fs_image_source_empty_extension() {
        let source = FsImageSource::new();
        assert!(!source.is_supported(Path::new("file.")));
    }

    #[test]
    fn test_fs_image_source_unicode_paths() {
        let source = FsImageSource::new();
        assert!(source.is_supported(Path::new("图片.png")));
        assert!(source.is_supported(Path::new("画像.jpg")));
        assert!(source.is_supported(Path::new("이미지.gif")));
    }

    #[test]
    fn test_fs_image_source_numeric_names() {
        let source = FsImageSource::new();
        assert!(source.is_supported(Path::new("001.png")));
        assert!(source.is_supported(Path::new("12345.jpg")));
    }

    #[test]
    fn test_fs_image_source_special_chars() {
        let source = FsImageSource::new();
        assert!(source.is_supported(Path::new("my-image_file.png")));
        assert!(source.is_supported(Path::new("image+test.jpg")));
    }
}
