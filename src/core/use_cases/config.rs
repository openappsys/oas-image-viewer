//! 配置管理用例与配置更新规则

use crate::core::domain::{GalleryLayout, ViewerSettings, WindowState};
use crate::core::ports::{AppConfig, Storage};
use crate::core::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ManageConfigUseCase {
    storage: Arc<dyn Storage>,
}

impl ManageConfigUseCase {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    pub fn load_config(&self) -> Result<AppConfig> {
        self.storage.load_config()
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        self.storage.save_config(config)
    }

    pub fn request_save(&self, config: &AppConfig) -> Result<()> {
        self.storage.request_save(config)
    }

    pub fn update_window_state(&self, config: &mut AppConfig, state: WindowState) {
        config.window = state;
    }

    pub fn update_gallery_layout(&self, config: &mut AppConfig, layout: GalleryLayout) {
        config.gallery = layout.validated();
    }

    pub fn update_viewer_settings(&self, config: &mut AppConfig, settings: ViewerSettings) {
        config.viewer = settings.validated();
    }

    pub fn set_last_directory(&self, config: &mut AppConfig, path: PathBuf) {
        config.last_opened_directory = Some(path);
    }

    pub fn validate_config(&self, config: &AppConfig) -> AppConfig {
        AppConfig {
            window: config.window,
            gallery: config.gallery.validated(),
            viewer: config.viewer.validated(),
            last_opened_directory: config.last_opened_directory.clone(),
            language: config.language,
            theme: config.theme,
        }
    }
}
