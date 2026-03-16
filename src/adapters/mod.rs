//! Adapters 层 - 适配器实现

pub mod clipboard;
pub mod egui;
pub mod macos_file_open;
pub mod platform;

pub use egui::info_panel;
pub use egui::shortcuts_help;
pub use egui::EguiApp;
