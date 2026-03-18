use super::OASImageViewerService;
use crate::core::domain::{BatchExecutionReport, BatchPreviewItem, BatchRenamePlan};
use crate::core::Result;
use std::path::PathBuf;

impl OASImageViewerService {
    pub fn preview_batch_rename(
        &self,
        sources: &[PathBuf],
        plan: &BatchRenamePlan,
    ) -> Result<Vec<BatchPreviewItem>> {
        self.batch_use_case.preview_rename(sources, plan)
    }

    pub fn execute_batch_rename(
        &self,
        sources: &[PathBuf],
        plan: &BatchRenamePlan,
    ) -> Result<BatchExecutionReport> {
        self.batch_use_case.execute_rename(sources, plan)
    }
}
