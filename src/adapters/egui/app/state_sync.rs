use super::types::EguiApp;
use crate::core::domain::{Language, Theme, ViewMode};
use crate::core::ports::AppConfig;
use std::path::PathBuf;

impl EguiApp {
    pub(super) fn update_view_mode(&self, mode: ViewMode) -> crate::core::Result<()> {
        self.service.update_state(|state| {
            self.service
                .view_use_case
                .set_view_mode(&mut state.view, mode);
        })
    }

    fn update_config<F>(&self, updater: F) -> crate::core::Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        self.service
            .update_state(|state| updater(&mut state.config))
    }

    fn update_config_and_request_save<F>(&self, updater: F) -> crate::core::Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        self.update_config(updater)?;
        self.request_save_config();
        Ok(())
    }

    fn request_save_config(&self) {
        if let Ok(state) = self.service.get_state() {
            if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                tracing::error!(error = %e, "请求保存配置失败");
            }
        }
    }

    pub(super) fn save_config_now(&self) {
        if let Ok(state) = self.service.get_state() {
            if let Err(e) = self.service.config_use_case.save_config(&state.config) {
                tracing::error!(error = %e, "保存配置失败");
            } else {
                tracing::info!("配置已保存");
            }
        }
    }

    pub(super) fn set_info_panel_visible(&self, visible: bool) -> crate::core::Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.show_info_panel = visible;
            self.service
                .config_use_case
                .update_viewer_settings(config, viewer);
        })
    }

    pub(super) fn toggle_info_panel_visible(&self) -> crate::core::Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.show_info_panel = !viewer.show_info_panel;
            self.service
                .config_use_case
                .update_viewer_settings(config, viewer);
        })
    }

    pub(super) fn set_theme_and_save(&self, theme: Theme) -> crate::core::Result<()> {
        self.update_config_and_request_save(|config| {
            config.theme = theme;
        })
    }

    pub(super) fn set_language_and_save(&self, language: Language) -> crate::core::Result<()> {
        self.update_config_and_request_save(|config| {
            config.language = language;
        })
    }

    pub(super) fn set_thumbnail_size_and_save(
        &self,
        thumbnail_size: u32,
    ) -> crate::core::Result<()> {
        self.update_config_and_request_save(|config| {
            let mut layout = config.gallery;
            layout.thumbnail_size = thumbnail_size;
            self.service
                .config_use_case
                .update_gallery_layout(config, layout);
        })
    }

    pub(super) fn set_last_opened_directory_and_save(
        &self,
        path: PathBuf,
    ) -> crate::core::Result<()> {
        self.update_config_and_request_save(|config| {
            self.service
                .config_use_case
                .set_last_directory(config, path);
        })
    }

    pub(super) fn set_window_position_and_save(&self, x: f32, y: f32) -> crate::core::Result<()> {
        self.update_config_and_request_save(|config| {
            let mut window = config.window;
            window.x = Some(x);
            window.y = Some(y);
            self.service
                .config_use_case
                .update_window_state(config, window);
        })
    }

    pub(super) fn set_about_window_position(
        &self,
        pos: Option<egui::Pos2>,
    ) -> crate::core::Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.about_window_pos = pos.map(|p| crate::core::domain::Position::new(p.x, p.y));
            self.service
                .config_use_case
                .update_viewer_settings(config, viewer);
        })
    }
}
