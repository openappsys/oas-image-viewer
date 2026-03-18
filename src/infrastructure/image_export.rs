use crate::core::domain::{ExportOptions, ImageTransform};
use crate::core::ports::ImageExportPort;
use crate::core::{CoreError, Result};
use std::path::{Path, PathBuf};

pub struct FsImageExportPort;

impl FsImageExportPort {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsImageExportPort {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageExportPort for FsImageExportPort {
    fn export_with_transforms(
        &self,
        _source: &Path,
        _transforms: &[ImageTransform],
        _options: &ExportOptions,
    ) -> Result<PathBuf> {
        Err(CoreError::technical(
            "NOT_IMPLEMENTED",
            "Image export is not implemented yet",
        ))
    }

    fn convert_format(&self, _source: &Path, _options: &ExportOptions) -> Result<PathBuf> {
        Err(CoreError::technical(
            "NOT_IMPLEMENTED",
            "Format conversion is not implemented yet",
        ))
    }
}
