use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImageTransform {
    Rotate90Cw,
    Rotate90Ccw,
    FlipHorizontal,
    FlipVertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExportFormat {
    KeepOriginal,
    Png,
    Jpeg,
    WebP,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OutputLocation {
    OverwriteOriginal,
    NewFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub quality: u8,
    pub output: OutputLocation,
}

impl ExportOptions {
    pub fn validated(&self) -> Self {
        Self {
            format: self.format,
            quality: self.quality.clamp(1, 100),
            output: self.output.clone(),
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::KeepOriginal,
            quality: 90,
            output: OutputLocation::OverwriteOriginal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BatchDestination {
    CurrentDirectory,
    Directory(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BatchRenamePlan {
    pub pattern: String,
    pub start_index: u32,
    pub padding: u8,
    pub destination: BatchDestination,
}

impl Default for BatchRenamePlan {
    fn default() -> Self {
        Self {
            pattern: "{index}".to_string(),
            start_index: 1,
            padding: 3,
            destination: BatchDestination::CurrentDirectory,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BatchPreviewItem {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BatchFailure {
    pub source: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct BatchExecutionReport {
    pub total: usize,
    pub succeeded: usize,
    pub failed: Vec<BatchFailure>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_options_quality_is_clamped() {
        let options = ExportOptions {
            quality: 180,
            ..ExportOptions::default()
        };
        assert_eq!(options.validated().quality, 100);
    }

    #[test]
    fn batch_rename_plan_default_values() {
        let plan = BatchRenamePlan::default();
        assert_eq!(plan.pattern, "{index}");
        assert_eq!(plan.start_index, 1);
        assert_eq!(plan.padding, 3);
        assert_eq!(plan.destination, BatchDestination::CurrentDirectory);
    }
}
