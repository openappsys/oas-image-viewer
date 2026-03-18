use super::OASImageViewerService;
use crate::core::domain::{Color, Language, Position, Theme, ViewMode, ViewerSettings};
use crate::core::Result;

impl OASImageViewerService {
    pub fn get_viewer_settings(&self) -> Result<ViewerSettings> {
        self.read_state(|s| s.config.viewer)
    }

    pub fn is_fit_to_window_enabled(&self) -> Result<bool> {
        self.read_state(|s| s.config.viewer.fit_to_window)
    }

    pub fn get_language(&self) -> Result<Language> {
        self.read_state(|s| s.config.language)
    }

    pub fn get_theme(&self) -> Result<Theme> {
        self.read_state(|s| s.config.theme)
    }

    pub fn should_show_info_panel(&self) -> Result<bool> {
        self.read_state(|s| {
            s.config.viewer.show_info_panel
                && s.view.current_image.is_some()
                && s.view.view_mode == ViewMode::Viewer
        })
    }

    pub fn get_about_window_position(&self) -> Result<Option<Position>> {
        self.read_state(|s| s.config.viewer.about_window_pos)
    }

    pub fn get_window_position(&self) -> Result<Option<(f32, f32)>> {
        self.read_state(|s| match (s.config.window.x, s.config.window.y) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        })
    }

    pub fn set_info_panel_visible(&self, visible: bool) -> Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.show_info_panel = visible;
            self.config_use_case.update_viewer_settings(config, viewer);
        })
    }

    pub fn toggle_info_panel_visible(&self) -> Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.show_info_panel = !viewer.show_info_panel;
            self.config_use_case.update_viewer_settings(config, viewer);
        })
    }

    pub fn set_window_position(&self, x: f32, y: f32) -> Result<()> {
        self.update_config(|config| {
            let mut window = config.window;
            window.x = Some(x);
            window.y = Some(y);
            self.config_use_case.update_window_state(config, window);
        })
    }

    pub fn set_about_window_position(&self, pos: Option<Position>) -> Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.about_window_pos = pos;
            self.config_use_case.update_viewer_settings(config, viewer);
        })
    }

    pub fn set_viewer_background_color(&self, color: Color) -> Result<()> {
        self.update_config(|config| {
            let mut viewer = config.viewer;
            viewer.background_color = color;
            self.config_use_case.update_viewer_settings(config, viewer);
        })
    }
}
