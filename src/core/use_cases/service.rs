//! 应用服务：聚合用例并提供语义化状态接口

mod batch;
mod config;
mod edit;
mod gallery;
mod lifecycle;
mod ui_state;
mod viewer;

use std::path::PathBuf;
use std::sync::Mutex;

use super::{
    AppState, BatchUseCase, EditImageUseCase, GalleryState, ManageConfigUseCase,
    NavigateGalleryUseCase, ViewImageUseCase,
};

pub type CurrentImageInfo = (PathBuf, (u32, u32), String);

pub struct OASImageViewerService {
    view_use_case: ViewImageUseCase,
    navigate_use_case: NavigateGalleryUseCase,
    config_use_case: ManageConfigUseCase,
    edit_use_case: EditImageUseCase,
    batch_use_case: BatchUseCase,
    state: Mutex<AppState>,
}
