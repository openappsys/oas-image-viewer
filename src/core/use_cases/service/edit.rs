use super::OASImageViewerService;
use crate::core::domain::{ExportOptions, ImageTransform};
use crate::core::Result;
use std::path::{Path, PathBuf};

impl OASImageViewerService {
    pub fn export_with_transforms(
        &self,
        source: &Path,
        transforms: &[ImageTransform],
        options: &ExportOptions,
    ) -> Result<PathBuf> {
        self.edit_use_case
            .export_with_transforms(source, transforms, options)
    }

    pub fn convert_format(&self, source: &Path, options: &ExportOptions) -> Result<PathBuf> {
        self.edit_use_case.convert_format(source, options)
    }
}
