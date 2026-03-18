use crate::core::domain::{BatchExecutionReport, BatchPreviewItem, BatchRenamePlan};
use crate::core::ports::BatchPort;
use crate::core::{CoreError, Result};
use std::path::PathBuf;

pub struct FsBatchPort;

impl FsBatchPort {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsBatchPort {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchPort for FsBatchPort {
    fn preview_rename(
        &self,
        _sources: &[PathBuf],
        _plan: &BatchRenamePlan,
    ) -> Result<Vec<BatchPreviewItem>> {
        Err(CoreError::technical(
            "NOT_IMPLEMENTED",
            "Batch rename preview is not implemented yet",
        ))
    }

    fn execute_rename(
        &self,
        _sources: &[PathBuf],
        _plan: &BatchRenamePlan,
    ) -> Result<BatchExecutionReport> {
        Err(CoreError::technical(
            "NOT_IMPLEMENTED",
            "Batch rename execution is not implemented yet",
        ))
    }
}
