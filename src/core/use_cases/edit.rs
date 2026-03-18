use crate::core::domain::{ExportOptions, ImageTransform};
use crate::core::ports::ImageExportPort;
use crate::core::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct EditImageUseCase {
    export_port: Arc<dyn ImageExportPort>,
}

impl EditImageUseCase {
    pub fn new(export_port: Arc<dyn ImageExportPort>) -> Self {
        Self { export_port }
    }

    pub fn export_with_transforms(
        &self,
        source: &Path,
        transforms: &[ImageTransform],
        options: &ExportOptions,
    ) -> Result<PathBuf> {
        self.export_port
            .export_with_transforms(source, transforms, &options.validated())
    }

    pub fn convert_format(&self, source: &Path, options: &ExportOptions) -> Result<PathBuf> {
        self.export_port.convert_format(source, &options.validated())
    }
}
