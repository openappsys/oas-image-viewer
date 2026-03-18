use crate::core::domain::{Image, ImageMetadata};
use crate::core::ports::{
    AsyncImageSource, ImageLoadedCallback, ImageSource, ThumbnailLoadedCallback,
};
use crate::core::Result;
use crate::utils::threading::ThreadPoolManager;
use std::path::{Path, PathBuf};

use super::FsImageSource;

/// 异步文件系统图像源
#[allow(dead_code)]
pub struct AsyncFsImageSource {
    inner: FsImageSource,
    thread_pool: ThreadPoolManager,
}

impl AsyncFsImageSource {
    /// 创建新的异步图像源
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            inner: FsImageSource::new(),
            thread_pool: ThreadPoolManager::default(),
        }
    }

    /// 使用指定线程数的异步图像源
    #[allow(dead_code)]
    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            inner: FsImageSource::new(),
            thread_pool: ThreadPoolManager::new(num_threads),
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

        self.thread_pool.spawn(move || {
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
        index: usize,
        callback: ThumbnailLoadedCallback,
    ) {
        let path = path.to_path_buf();
        let inner = FsImageSource::new();

        self.thread_pool.spawn(move || {
            let result = inner
                .generate_thumbnail(&path, max_size)
                .map(|(_, _, data)| data);
            callback(index, result);
        });
    }
}
