//! Domain 层 - 核心业务实体和值对象

pub mod errors;
pub mod editing;
pub mod image;
pub mod language;
pub mod types;

// 重新导出常用类型
pub use errors::{Boundary, ConfigError, GalleryError, UnavailableReason, ViewError};
pub use editing::{
    BatchDestination, BatchExecutionReport, BatchFailure, BatchPreviewItem, BatchRenamePlan,
    ExportFormat, ExportOptions, ImageTransform, OutputLocation,
};
pub use image::{is_image_file, Gallery, Image, ImageFormat, ImageMetadata};
pub use language::Language;
pub use types::{
    AppConfig, Color, Dimensions, DisplayMode, GalleryLayout, NavigationDirection, Position, Scale,
    Theme, ViewMode, ViewerSettings, WindowState,
};
