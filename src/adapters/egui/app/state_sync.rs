//! 应用状态写回与配置持久化辅助逻辑

use super::types::EguiApp;
use crate::core::domain::{Color, Language, Position, Theme, ViewMode};
use std::path::PathBuf;

impl EguiApp {
    pub(super) fn update_view_mode(&self, mode: ViewMode) -> crate::core::Result<()> {
        self.service.set_view_mode(mode)
    }

    fn request_save_config(&self) {
        if let Err(e) = self.service.request_save_config() {
            tracing::error!(error = %e, "请求保存配置失败");
        }
    }

    pub(super) fn save_config_now(&self) {
        if let Err(e) = self.service.save_config_now() {
            tracing::error!(error = %e, "保存配置失败");
        } else {
            tracing::info!("配置已保存");
        }
    }

    pub(super) fn set_info_panel_visible(&self, visible: bool) -> crate::core::Result<()> {
        self.service.set_info_panel_visible(visible)
    }

    pub(super) fn toggle_info_panel_visible(&self) -> crate::core::Result<()> {
        self.service.toggle_info_panel_visible()
    }

    pub(super) fn set_theme_and_save(&self, theme: Theme) -> crate::core::Result<()> {
        self.service.set_theme(theme)?;
        self.request_save_config();
        Ok(())
    }

    pub(super) fn set_language_and_save(&self, language: Language) -> crate::core::Result<()> {
        self.service.set_language(language)?;
        self.request_save_config();
        Ok(())
    }

    pub(super) fn set_thumbnail_size_and_save(
        &self,
        thumbnail_size: u32,
    ) -> crate::core::Result<()> {
        self.service.set_thumbnail_size(thumbnail_size)?;
        self.request_save_config();
        Ok(())
    }

    pub(super) fn set_viewer_background_color_and_save(
        &self,
        color: Color,
    ) -> crate::core::Result<()> {
        self.service.set_viewer_background_color(color)?;
        self.request_save_config();
        Ok(())
    }

    pub(super) fn set_last_opened_directory_and_save(
        &self,
        path: PathBuf,
    ) -> crate::core::Result<()> {
        self.service.set_last_opened_directory(path)?;
        self.request_save_config();
        Ok(())
    }

    pub(super) fn set_window_position_and_save(&self, x: f32, y: f32) -> crate::core::Result<()> {
        self.service.set_window_position(x, y)?;
        self.request_save_config();
        Ok(())
    }

    pub(super) fn set_about_window_position(
        &self,
        pos: Option<egui::Pos2>,
    ) -> crate::core::Result<()> {
        self.service
            .set_about_window_position(pos.map(|p| Position::new(p.x, p.y)))
    }
}
