//! Image-Viewer - 一个现代化的图片查看器
//!
//! 采用 Clean Architecture 架构：
//! - core: 纯业务逻辑，零外部依赖
//! - infrastructure: 技术实现
//! - adapters: UI 适配器

pub mod adapters;
pub mod core;
pub mod infrastructure;
pub mod utils;

// 保持向后兼容的重新导出
pub use core::domain;
pub use core::ports;
pub use core::use_cases;

/// 版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
