//! Domain 层 - 核心业务实体和值对象

pub mod image;
pub mod types;

// 重新导出常用类型
pub use image::{is_image_file, Gallery, Image, ImageFormat, ImageMetadata};
pub use types::{
    Color, Dimensions, DisplayMode, GalleryLayout, NavigationDirection, Position, Scale, ViewMode,
    ViewerSettings, WindowState,
};
