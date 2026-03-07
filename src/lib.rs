//! Image-Viewer - 一个现代化的图片查看器
//!
//! 采用 Clean Architecture 架构：
//! - core: 纯业务逻辑，零外部依赖
//! - infrastructure: 技术实现
//! - adapters: UI 适配器

use std::sync::atomic::{AtomicBool, Ordering};

/// 全局标志：是否支持中文字体显示
static CHINESE_FONT_SUPPORTED: AtomicBool = AtomicBool::new(false);

/// 设置中文字体支持状态（由 main.rs 在初始化时调用）
pub fn set_chinese_supported(supported: bool) {
    CHINESE_FONT_SUPPORTED.store(supported, Ordering::Relaxed);
}

/// 检查是否支持中文字体显示
pub fn is_chinese_supported() -> bool {
    CHINESE_FONT_SUPPORTED.load(Ordering::Relaxed)
}

/// 获取当前界面语言应该使用的文本
pub fn ui_text<'a>(chinese: &'a str, english: &'a str) -> &'a str {
    if is_chinese_supported() {
        chinese
    } else {
        english
    }
}

pub mod adapters;
pub mod core;
pub(crate) mod infrastructure;
pub(crate) mod utils;

// 从 infrastructure 重新导出需要的类型
pub use infrastructure::{FsImageSource, JsonStorage};

// 保持向后兼容的重新导出
pub use adapters::clipboard;
pub use adapters::info_panel;
pub use adapters::shortcuts_help;
pub use core::domain;
pub use core::ports;
pub use core::use_cases;

/// 版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
