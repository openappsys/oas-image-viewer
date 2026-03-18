use super::OASImageViewerService;
use crate::core::domain::{Language, Theme};
use crate::core::ports::AppConfig;
use crate::core::Result;
use std::path::PathBuf;

impl OASImageViewerService {
    pub fn get_config(&self) -> Result<AppConfig> {
        self.read_state(|s| s.config.clone())
    }

    pub fn update_config(&self, updater: impl FnOnce(&mut AppConfig)) -> Result<()> {
        self.update_state(|state| updater(&mut state.config))
    }

    pub fn request_save_config(&self) -> Result<()> {
        let config = self.get_config()?;
        self.config_use_case.request_save(&config)
    }

    pub fn save_config_now(&self) -> Result<()> {
        let config = self.get_config()?;
        self.config_use_case.save_config(&config)
    }

    pub fn set_theme(&self, theme: Theme) -> Result<()> {
        self.update_config(|config| {
            config.theme = theme;
        })
    }

    pub fn set_language(&self, language: Language) -> Result<()> {
        self.update_config(|config| {
            config.language = language;
        })
    }

    pub fn set_thumbnail_size(&self, thumbnail_size: u32) -> Result<()> {
        self.update_config(|config| {
            let mut layout = config.gallery;
            layout.thumbnail_size = thumbnail_size;
            self.config_use_case.update_gallery_layout(config, layout);
        })
    }

    pub fn set_last_opened_directory(&self, path: PathBuf) -> Result<()> {
        self.update_config(|config| {
            self.config_use_case.set_last_directory(config, path);
        })
    }
}
