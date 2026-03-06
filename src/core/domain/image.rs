//! Domain 实体 - 核心业务对象
//!
//! 纯 Rust 结构体，不依赖任何外部框架

use std::path::{Path, PathBuf};

/// 图像实体
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    id: String,
    path: PathBuf,
    metadata: ImageMetadata,
}

/// 图像元数据
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub file_size: u64,
    pub created_at: Option<u64>, // Unix timestamp
    pub modified_at: Option<u64>,
}

/// 支持的图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Tiff,
    Bmp,
    #[default]
    Unknown,
}

/// 画廊实体 - 管理图像集合
#[derive(Debug, Clone, PartialEq)]
pub struct Gallery {
    images: Vec<Image>,
    selected_index: Option<usize>,
    name: String,
}

impl Image {
    /// 创建新的图像实体
    pub fn new(id: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            metadata: ImageMetadata::default(),
        }
    }

    /// 获取图像 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取图像路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 获取文件名
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// 获取元数据
    pub fn metadata(&self) -> &ImageMetadata {
        &self.metadata
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, metadata: ImageMetadata) {
        self.metadata = metadata;
    }

    /// 检测图像格式
    pub fn detect_format(path: &Path) -> ImageFormat {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| match e.to_lowercase().as_str() {
                "png" => ImageFormat::Png,
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "gif" => ImageFormat::Gif,
                "webp" => ImageFormat::Webp,
                "tiff" | "tif" => ImageFormat::Tiff,
                "bmp" => ImageFormat::Bmp,
                _ => ImageFormat::Unknown,
            })
            .unwrap_or(ImageFormat::Unknown)
    }

    /// 计算图像的百万像素数
    pub fn megapixels(&self) -> f64 {
        (self.metadata.width as f64 * self.metadata.height as f64) / 1_000_000.0
    }

    /// 获取宽高比
    pub fn aspect_ratio(&self) -> f32 {
        if self.metadata.height == 0 {
            return 0.0;
        }
        self.metadata.width as f32 / self.metadata.height as f32
    }
}

impl ImageFormat {
    /// 获取格式的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::Gif => "GIF",
            ImageFormat::Webp => "WebP",
            ImageFormat::Tiff => "TIFF",
            ImageFormat::Bmp => "BMP",
            ImageFormat::Unknown => "Unknown",
        }
    }

    /// 检查是否为支持的格式
    pub fn is_supported(&self) -> bool {
        !matches!(self, ImageFormat::Unknown)
    }
}

impl Gallery {
    /// 创建新的空画廊
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            images: Vec::new(),
            selected_index: None,
            name: name.into(),
        }
    }

    /// 从路径列表创建画廊
    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut gallery = Self::new("Gallery");
        for (idx, path) in paths.into_iter().enumerate() {
            let image = Image::new(format!("img_{}", idx), path);
            gallery.add_image(image);
        }
        gallery
    }

    /// 添加图像
    pub fn add_image(&mut self, image: Image) {
        self.images.push(image);
    }

    /// 移除图像
    pub fn remove_image(&mut self, index: usize) -> Option<Image> {
        if index < self.images.len() {
            // 更新选中索引
            if let Some(selected) = self.selected_index {
                if selected == index {
                    self.selected_index = None;
                } else if selected > index {
                    self.selected_index = Some(selected - 1);
                }
            }
            Some(self.images.remove(index))
        } else {
            None
        }
    }

    /// 获取图像数量
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// 获取所有图像
    pub fn images(&self) -> &[Image] {
        &self.images
    }

    /// 获取指定索引的图像
    pub fn get_image(&self, index: usize) -> Option<&Image> {
        self.images.get(index)
    }

    /// 获取选中图像
    pub fn selected_image(&self) -> Option<&Image> {
        self.selected_index.and_then(|idx| self.images.get(idx))
    }

    /// 获取选中索引
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// 选择图像
    pub fn select_image(&mut self, index: usize) -> bool {
        if index < self.images.len() {
            self.selected_index = Some(index);
            true
        } else {
            false
        }
    }

    /// 选择下一个图像
    pub fn select_next(&mut self) -> bool {
        match self.selected_index {
            Some(idx) if idx + 1 < self.images.len() => {
                self.selected_index = Some(idx + 1);
                true
            }
            None if !self.images.is_empty() => {
                self.selected_index = Some(0);
                true
            }
            _ => false,
        }
    }

    /// 选择上一个图像
    pub fn select_prev(&mut self) -> bool {
        match self.selected_index {
            Some(idx) if idx > 0 => {
                self.selected_index = Some(idx - 1);
                true
            }
            None if !self.images.is_empty() => {
                self.selected_index = Some(self.images.len() - 1);
                true
            }
            _ => false,
        }
    }

    /// 按行选择（用于网格导航）
    pub fn select_by_offset(&mut self, offset: isize, items_per_row: usize) -> bool {
        if items_per_row == 0 || self.images.is_empty() {
            return false;
        }

        let current = self.selected_index.unwrap_or(0);
        let new_index = current as isize + offset * items_per_row as isize;

        if new_index >= 0 && (new_index as usize) < self.images.len() {
            self.selected_index = Some(new_index as usize);
            true
        } else {
            false
        }
    }

    /// 向上选择
    pub fn select_up(&mut self, items_per_row: usize) -> bool {
        self.select_by_offset(-1, items_per_row)
    }

    /// 向下选择
    pub fn select_down(&mut self, items_per_row: usize) -> bool {
        self.select_by_offset(1, items_per_row)
    }

    /// 清除所有图像
    pub fn clear(&mut self) {
        self.images.clear();
        self.selected_index = None;
    }

    /// 获取图像索引
    pub fn index_of(&self, image: &Image) -> Option<usize> {
        self.images.iter().position(|i| i.id == image.id)
    }

    /// 按路径查找图像索引
    pub fn index_by_path(&self, path: &Path) -> Option<usize> {
        self.images.iter().position(|i| i.path == path)
    }

    /// 获取画廊名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 判断路径是否为支持的图像文件
pub fn is_image_file(path: &Path) -> bool {
    Image::detect_format(path).is_supported()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_image_new() {
        let image = Image::new("test_1", "/path/to/test.png");
        assert_eq!(image.id(), "test_1");
        assert_eq!(image.path(), Path::new("/path/to/test.png"));
    }

    #[test]
    fn test_image_file_name() {
        let image = Image::new("1", "/path/to/image.png");
        assert_eq!(image.file_name(), Some("image.png"));
    }

    #[test]
    fn test_image_extension() {
        let image = Image::new("1", "/path/to/image.PNG");
        assert_eq!(image.extension(), Some("PNG"));
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(
            Image::detect_format(Path::new("test.png")),
            ImageFormat::Png
        );
        assert_eq!(
            Image::detect_format(Path::new("test.jpg")),
            ImageFormat::Jpeg
        );
        assert_eq!(
            Image::detect_format(Path::new("test.jpeg")),
            ImageFormat::Jpeg
        );
        assert_eq!(
            Image::detect_format(Path::new("test.gif")),
            ImageFormat::Gif
        );
        assert_eq!(
            Image::detect_format(Path::new("test.webp")),
            ImageFormat::Webp
        );
        assert_eq!(
            Image::detect_format(Path::new("test.tiff")),
            ImageFormat::Tiff
        );
        assert_eq!(
            Image::detect_format(Path::new("test.bmp")),
            ImageFormat::Bmp
        );
        assert_eq!(
            Image::detect_format(Path::new("test.txt")),
            ImageFormat::Unknown
        );
        assert_eq!(
            Image::detect_format(Path::new("test")),
            ImageFormat::Unknown
        );
    }

    #[test]
    fn test_image_format_display() {
        assert_eq!(ImageFormat::Png.display_name(), "PNG");
        assert_eq!(ImageFormat::Unknown.display_name(), "Unknown");
    }

    #[test]
    fn test_image_megapixels() {
        let mut image = Image::new("1", "/test.png");
        let metadata = ImageMetadata {
            width: 2000,
            height: 1500,
            ..Default::default()
        };
        image.set_metadata(metadata);
        assert!((image.megapixels() - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_image_aspect_ratio() {
        let mut image = Image::new("1", "/test.png");
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        image.set_metadata(metadata);
        assert!((image.aspect_ratio() - 1.777).abs() < 0.01);
    }

    #[test]
    fn test_gallery_new() {
        let gallery = Gallery::new("Test Gallery");
        assert!(gallery.is_empty());
        assert_eq!(gallery.len(), 0);
        assert_eq!(gallery.name(), "Test Gallery");
    }

    #[test]
    fn test_gallery_add_image() {
        let mut gallery = Gallery::new("Test");
        let image = Image::new("1", "/test.png");
        gallery.add_image(image);
        assert_eq!(gallery.len(), 1);
    }

    #[test]
    fn test_gallery_select_image() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        assert!(gallery.select_image(0));
        assert_eq!(gallery.selected_index(), Some(0));

        assert!(gallery.select_image(1));
        assert_eq!(gallery.selected_index(), Some(1));

        assert!(!gallery.select_image(5)); // 无效索引
    }

    #[test]
    fn test_gallery_select_next() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        assert!(gallery.select_next());
        assert_eq!(gallery.selected_index(), Some(0));

        assert!(gallery.select_next());
        assert_eq!(gallery.selected_index(), Some(1));

        assert!(!gallery.select_next()); // 已经在最后
    }

    #[test]
    fn test_gallery_select_prev() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        gallery.select_image(1);
        assert!(gallery.select_prev());
        assert_eq!(gallery.selected_index(), Some(0));

        assert!(!gallery.select_prev()); // 已经在最前
    }

    #[test]
    fn test_gallery_remove_image() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));
        gallery.select_image(0);

        let removed = gallery.remove_image(0);
        assert!(removed.is_some());
        assert_eq!(gallery.len(), 1);
        assert!(gallery.selected_index().is_none()); // 选中的被移除了
    }

    #[test]
    fn test_gallery_select_up_down() {
        let mut gallery = Gallery::new("Test");
        for i in 0..9 {
            gallery.add_image(Image::new(format!("{}", i), format!("/{}.png", i)));
        }

        gallery.select_image(4); // 第二行中间
        assert!(gallery.select_up(3)); // 3 items per row
        assert_eq!(gallery.selected_index(), Some(1));

        assert!(!gallery.select_up(3)); // 已经在第一行

        assert!(gallery.select_down(3));
        assert_eq!(gallery.selected_index(), Some(4));
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test")));
    }

    #[test]
    fn test_gallery_from_paths() {
        let paths = vec![PathBuf::from("/a.png"), PathBuf::from("/b.jpg")];
        let gallery = Gallery::from_paths(paths);
        assert_eq!(gallery.len(), 2);
    }

    #[test]
    fn test_gallery_index_by_path() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.jpg"));

        assert_eq!(gallery.index_by_path(Path::new("/a.png")), Some(0));
        assert_eq!(gallery.index_by_path(Path::new("/b.jpg")), Some(1));
        assert_eq!(gallery.index_by_path(Path::new("/c.png")), None);
    }

    #[test]
    fn test_gallery_clear() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.select_image(0);

        gallery.clear();
        assert!(gallery.is_empty());
        assert!(gallery.selected_index().is_none());
    }

    // =========================================================================
    // 从旧代码迁移的额外测试
    // =========================================================================

    #[test]
    fn test_image_format_variants() {
        let formats = [
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Gif,
            ImageFormat::Webp,
            ImageFormat::Tiff,
            ImageFormat::Bmp,
            ImageFormat::Unknown,
        ];
        assert_eq!(formats.len(), 7);
    }

    #[test]
    fn test_image_format_equality() {
        assert_eq!(ImageFormat::Png, ImageFormat::Png);
        assert_ne!(ImageFormat::Png, ImageFormat::Jpeg);
        assert_eq!(ImageFormat::Unknown, ImageFormat::Unknown);
    }

    #[test]
    fn test_image_format_clone() {
        let format = ImageFormat::Png;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_image_format_copy() {
        let format = ImageFormat::Jpeg;
        let copied = format;
        assert_eq!(format, ImageFormat::Jpeg);
        assert_eq!(copied, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_format_debug() {
        let format = ImageFormat::Png;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Png"));
    }

    #[test]
    fn test_image_format_default() {
        let format: ImageFormat = Default::default();
        assert_eq!(format, ImageFormat::Unknown);
    }

    #[test]
    fn test_image_format_is_supported() {
        assert!(ImageFormat::Png.is_supported());
        assert!(ImageFormat::Jpeg.is_supported());
        assert!(ImageFormat::Gif.is_supported());
        assert!(ImageFormat::Webp.is_supported());
        assert!(ImageFormat::Tiff.is_supported());
        assert!(ImageFormat::Bmp.is_supported());
        assert!(!ImageFormat::Unknown.is_supported());
    }

    #[test]
    fn test_detect_format_case_insensitive() {
        assert_eq!(
            Image::detect_format(Path::new("test.PNG")),
            ImageFormat::Png
        );
        assert_eq!(
            Image::detect_format(Path::new("test.JPG")),
            ImageFormat::Jpeg
        );
        assert_eq!(
            Image::detect_format(Path::new("test.JPEG")),
            ImageFormat::Jpeg
        );
        assert_eq!(
            Image::detect_format(Path::new("test.GIF")),
            ImageFormat::Gif
        );
        assert_eq!(
            Image::detect_format(Path::new("test.WEBP")),
            ImageFormat::Webp
        );
        assert_eq!(
            Image::detect_format(Path::new("test.TIFF")),
            ImageFormat::Tiff
        );
        assert_eq!(
            Image::detect_format(Path::new("test.TIF")),
            ImageFormat::Tiff
        );
        assert_eq!(
            Image::detect_format(Path::new("test.BMP")),
            ImageFormat::Bmp
        );
    }

    #[test]
    fn test_detect_format_with_paths() {
        assert_eq!(
            Image::detect_format(Path::new("/home/user/image.png")),
            ImageFormat::Png
        );
        assert_eq!(
            Image::detect_format(Path::new("./relative/path/image.jpg")),
            ImageFormat::Jpeg
        );
        assert_eq!(
            Image::detect_format(Path::new("C:\\Users\\image.gif")),
            ImageFormat::Gif
        );
    }

    #[test]
    fn test_image_metadata_default() {
        let metadata = ImageMetadata::default();
        assert_eq!(metadata.width, 0);
        assert_eq!(metadata.height, 0);
        assert_eq!(metadata.format, ImageFormat::Unknown);
        assert_eq!(metadata.file_size, 0);
        assert_eq!(metadata.created_at, None);
        assert_eq!(metadata.modified_at, None);
    }

    #[test]
    fn test_image_metadata_clone() {
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            format: ImageFormat::Png,
            file_size: 1024,
            created_at: Some(1234567890),
            modified_at: Some(1234567891),
        };
        let cloned = metadata.clone();
        assert_eq!(metadata, cloned);
    }

    #[test]
    fn test_image_with_metadata() {
        let mut image = Image::new("1", "/test.png");
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            format: ImageFormat::Png,
            file_size: 1024,
            created_at: Some(1234567890),
            modified_at: Some(1234567891),
        };
        image.set_metadata(metadata);

        assert_eq!(image.metadata().width, 1920);
        assert_eq!(image.metadata().height, 1080);
        assert_eq!(image.metadata().format, ImageFormat::Png);
        assert_eq!(image.metadata().file_size, 1024);
    }

    #[test]
    fn test_image_aspect_ratio_zero_height() {
        let mut image = Image::new("1", "/test.png");
        let metadata = ImageMetadata {
            width: 1920,
            height: 0,
            ..Default::default()
        };
        image.set_metadata(metadata);
        assert_eq!(image.aspect_ratio(), 0.0);
    }

    #[test]
    fn test_image_megapixels_zero() {
        let mut image = Image::new("1", "/test.png");
        let metadata = ImageMetadata {
            width: 0,
            height: 0,
            ..Default::default()
        };
        image.set_metadata(metadata);
        assert_eq!(image.megapixels(), 0.0);
    }

    #[test]
    fn test_gallery_add_multiple_images() {
        let mut gallery = Gallery::new("Test");
        for i in 0..100 {
            gallery.add_image(Image::new(format!("{}", i), format!("/{}.png", i)));
        }
        assert_eq!(gallery.len(), 100);
    }

    #[test]
    fn test_gallery_select_invalid_indices() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));

        assert!(!gallery.select_image(1));
        assert!(!gallery.select_image(100));
        assert!(!gallery.select_image(usize::MAX));
    }

    #[test]
    fn test_gallery_remove_invalid_index() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));

        assert!(gallery.remove_image(1).is_none());
        assert!(gallery.remove_image(100).is_none());
    }

    #[test]
    fn test_gallery_remove_updates_selected_after() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));
        gallery.add_image(Image::new("3", "/c.png"));
        gallery.select_image(2);

        gallery.remove_image(0);
        assert_eq!(gallery.selected_index(), Some(1));
    }

    #[test]
    fn test_gallery_selected_image() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        assert!(gallery.selected_image().is_none());

        gallery.select_image(0);
        assert_eq!(gallery.selected_image().unwrap().id(), "1");

        gallery.select_image(1);
        assert_eq!(gallery.selected_image().unwrap().id(), "2");
    }

    #[test]
    fn test_gallery_get_image() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        assert_eq!(gallery.get_image(0).unwrap().id(), "1");
        assert_eq!(gallery.get_image(1).unwrap().id(), "2");
        assert!(gallery.get_image(2).is_none());
        assert!(gallery.get_image(100).is_none());
    }

    #[test]
    fn test_gallery_images_accessor() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        let images = gallery.images();
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].id(), "1");
        assert_eq!(images[1].id(), "2");
    }

    #[test]
    fn test_gallery_index_of() {
        let mut gallery = Gallery::new("Test");
        let image1 = Image::new("1", "/a.png");
        let image2 = Image::new("2", "/b.png");
        gallery.add_image(image1.clone());
        gallery.add_image(image2.clone());

        assert_eq!(gallery.index_of(&image1), Some(0));
        assert_eq!(gallery.index_of(&image2), Some(1));
    }

    #[test]
    fn test_gallery_select_next_from_none() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        assert!(gallery.select_next());
        assert_eq!(gallery.selected_index(), Some(0));
    }

    #[test]
    fn test_gallery_select_prev_from_none() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.add_image(Image::new("2", "/b.png"));

        assert!(gallery.select_prev());
        assert_eq!(gallery.selected_index(), Some(1));
    }

    #[test]
    fn test_gallery_select_by_offset_zero_items_per_row() {
        let mut gallery = Gallery::new("Test");
        gallery.add_image(Image::new("1", "/a.png"));
        gallery.select_image(0);

        assert!(!gallery.select_by_offset(1, 0));
    }

    #[test]
    fn test_gallery_select_by_offset_empty_gallery() {
        let mut gallery = Gallery::new("Test");

        assert!(!gallery.select_by_offset(1, 3));
    }

    #[test]
    fn test_gallery_select_down_at_bottom() {
        let mut gallery = Gallery::new("Test");
        for i in 0..6 {
            gallery.add_image(Image::new(format!("{}", i), format!("/{}.png", i)));
        }
        gallery.select_image(4);

        assert!(!gallery.select_down(3));
        assert_eq!(gallery.selected_index(), Some(4));
    }

    #[test]
    fn test_gallery_select_up_at_top() {
        let mut gallery = Gallery::new("Test");
        for i in 0..6 {
            gallery.add_image(Image::new(format!("{}", i), format!("/{}.png", i)));
        }
        gallery.select_image(1);

        assert!(!gallery.select_up(3));
        assert_eq!(gallery.selected_index(), Some(1));
    }

    #[test]
    fn test_gallery_is_empty() {
        let mut gallery = Gallery::new("Test");
        assert!(gallery.is_empty());

        gallery.add_image(Image::new("1", "/a.png"));
        assert!(!gallery.is_empty());

        gallery.clear();
        assert!(gallery.is_empty());
    }

    #[test]
    fn test_gallery_len_consistency() {
        let mut gallery = Gallery::new("Test");
        assert_eq!(gallery.len(), gallery.images().len());

        gallery.add_image(Image::new("1", "/a.png"));
        assert_eq!(gallery.len(), 1);
        assert_eq!(gallery.len(), gallery.images().len());

        gallery.remove_image(0);
        assert_eq!(gallery.len(), 0);
        assert_eq!(gallery.len(), gallery.images().len());
    }

    #[test]
    fn test_gallery_from_paths_empty() {
        let paths: Vec<PathBuf> = vec![];
        let gallery = Gallery::from_paths(paths);
        assert!(gallery.is_empty());
    }

    #[test]
    fn test_gallery_from_paths_many() {
        let paths: Vec<PathBuf> = (0..100)
            .map(|i| PathBuf::from(format!("/image{}.png", i)))
            .collect();
        let gallery = Gallery::from_paths(paths);
        assert_eq!(gallery.len(), 100);
    }

    #[test]
    fn test_image_clone() {
        let mut image = Image::new("1", "/test.png");
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            format: ImageFormat::Png,
            file_size: 1024,
            created_at: Some(1234567890),
            modified_at: Some(1234567891),
        };
        image.set_metadata(metadata);

        let cloned = image.clone();
        assert_eq!(image.id(), cloned.id());
        assert_eq!(image.path(), cloned.path());
        assert_eq!(image.metadata(), cloned.metadata());
    }

    #[test]
    fn test_image_equality() {
        let image1 = Image::new("1", "/test.png");
        let image2 = Image::new("1", "/test.png");
        let image3 = Image::new("2", "/other.png");

        assert_eq!(image1, image2);
        assert_ne!(image1, image3);
    }

    #[test]
    fn test_is_image_file_various() {
        // 支持的格式
        assert!(is_image_file(Path::new("image.png")));
        assert!(is_image_file(Path::new("image.jpg")));
        assert!(is_image_file(Path::new("image.jpeg")));
        assert!(is_image_file(Path::new("image.gif")));
        assert!(is_image_file(Path::new("image.webp")));
        assert!(is_image_file(Path::new("image.tiff")));
        assert!(is_image_file(Path::new("image.tif")));
        assert!(is_image_file(Path::new("image.bmp")));

        // 不支持的格式
        assert!(!is_image_file(Path::new("image.txt")));
        assert!(!is_image_file(Path::new("image.rs")));
        assert!(!is_image_file(Path::new("image.pdf")));
        assert!(!is_image_file(Path::new("image.zip")));
        assert!(!is_image_file(Path::new("image.mp4")));
        assert!(!is_image_file(Path::new("image")));
        assert!(!is_image_file(Path::new("")));
    }

    #[test]
    fn test_is_image_file_with_full_paths() {
        assert!(is_image_file(Path::new("/usr/share/images/wallpaper.png")));
        assert!(is_image_file(Path::new(
            "C:\\Users\\User\\Pictures\\photo.jpg"
        )));
        assert!(is_image_file(Path::new("./images/animation.gif")));
        assert!(is_image_file(Path::new("../assets/icon.webp")));
    }

    #[test]
    fn test_gallery_name() {
        let gallery = Gallery::new("My Gallery");
        assert_eq!(gallery.name(), "My Gallery");
    }

    #[test]
    fn test_image_file_name_none() {
        let image = Image::new("1", "/");
        assert!(image.file_name().is_none());
    }

    #[test]
    fn test_image_extension_none() {
        let image = Image::new("1", "/test");
        assert!(image.extension().is_none());
    }
}
