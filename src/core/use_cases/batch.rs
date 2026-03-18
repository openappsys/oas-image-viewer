use crate::core::domain::{BatchExecutionReport, BatchPreviewItem, BatchRenamePlan};
use crate::core::ports::BatchPort;
use crate::core::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub struct BatchUseCase {
    batch_port: Arc<dyn BatchPort>,
}

impl BatchUseCase {
    pub fn new(batch_port: Arc<dyn BatchPort>) -> Self {
        Self { batch_port }
    }

    pub fn preview_rename(
        &self,
        sources: &[PathBuf],
        plan: &BatchRenamePlan,
    ) -> Result<Vec<BatchPreviewItem>> {
        self.batch_port.preview_rename(sources, plan)
    }

    pub fn execute_rename(
        &self,
        sources: &[PathBuf],
        plan: &BatchRenamePlan,
    ) -> Result<BatchExecutionReport> {
        self.batch_port.execute_rename(sources, plan)
    }
}
