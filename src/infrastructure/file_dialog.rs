use crate::core::ports::FileDialogPort;
use std::path::PathBuf;

/// rfd 文件对话框实现
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
