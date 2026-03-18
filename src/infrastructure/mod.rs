//! Infrastructure 层 - 技术实现
//!
//! 实现 Core 层定义的端口接口

mod async_image_source;
mod batch_port;
mod file_dialog;
mod fs_image_source;
mod image_export;
mod storage;

pub use batch_port::FsBatchPort;
pub use file_dialog::RfdFileDialog;
pub use fs_image_source::FsImageSource;
pub use image_export::FsImageExportPort;
pub use storage::JsonStorage;

#[cfg(test)]
mod tests;
