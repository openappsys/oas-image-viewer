//! 应用服务：聚合用例并提供语义化状态接口

use crate::core::domain::{
    Image, Language, NavigationDirection, Position, Theme, ViewMode, ViewerSettings,
};
use crate::core::ports::{AppConfig, ImageSource};
use crate::core::{CoreError, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::{
    AppState, GalleryState, ManageConfigUseCase, NavigateGalleryUseCase, ViewImageUseCase, ViewState,
};

pub type CurrentImageInfo = (PathBuf, (u32, u32), String);

pub struct OASImageViewerService {
    view_use_case: ViewImageUseCase,
    navigate_use_case: NavigateGalleryUseCase,
    config_use_case: ManageConfigUseCase,
    state: Mutex<AppState>,
}

impl OASImageViewerService {
    fn read_state<T>(&self, f: impl FnOnce(&AppState) -> T) -> Result<T> {
        let state = self
            .state
            .lock()
            .map_err(|_| CoreError::technical("CONFIG_ERROR", "Lock poisoned".to_string()))?;
        Ok(f(&state))
    }

    fn write_state<T>(&self, f: impl FnOnce(&mut AppState) -> T) -> Result<T> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CoreError::technical("CONFIG_ERROR", "Lock poisoned".to_string()))?;
        Ok(f(&mut state))
    }

    pub fn new(
        view_use_case: ViewImageUseCase,
        navigate_use_case: NavigateGalleryUseCase,
        config_use_case: ManageConfigUseCase,
    ) -> Self {
        let config = AppConfig::default();
        let state = AppState {
            view: ViewState::default(),
            gallery: GalleryState::default(),
            config: config.clone(),
        };

        Self {
            view_use_case,
            navigate_use_case,
            config_use_case,
            state: Mutex::new(state),
        }
    }

    pub fn initialize(&self, config: Option<AppConfig>) -> Result<()> {
        let config = if let Some(cfg) = config {
            cfg
        } else {
            self.config_use_case.load_config()?
        };

        let mut state = self
            .state
            .lock()
            .map_err(|_| CoreError::technical("CONFIG_ERROR", "Lock poisoned".to_string()))?;
        state.config = config;
        state.gallery.layout = state.config.gallery;
        Ok(())
    }

    pub fn get_state(&self) -> Result<AppState> {
        self.read_state(|s| s.clone())
    }

    pub fn get_view_mode(&self) -> Result<ViewMode> {
        self.read_state(|s| s.view.view_mode)
    }

    pub fn get_gallery_state_for_render(&self) -> Result<GalleryState> {
        self.read_state(|s| {
            let mut gallery = s.gallery.clone();
            gallery.layout.thumbnail_size = s.config.gallery.thumbnail_size;
            gallery
        })
    }

    pub fn get_view_state(&self) -> Result<ViewState> {
        self.read_state(|s| s.view.clone())
    }

    pub fn set_view_state(&self, view: ViewState) -> Result<()> {
        self.update_state(|state| {
            state.view = view;
        })
    }

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

    pub fn get_selected_gallery_image_for_open(&self) -> Result<Option<(PathBuf, bool)>> {
        self.read_state(|s| {
            if s.view.view_mode != ViewMode::Gallery {
                return None;
            }
            s.gallery.gallery.selected_index().and_then(|index| {
                s.gallery
                    .gallery
                    .get_image(index)
                    .map(|img| (img.path().to_path_buf(), s.config.viewer.fit_to_window))
            })
        })
    }

    pub fn get_current_view_image_path_and_language(&self) -> Result<Option<(PathBuf, Language)>> {
        self.read_state(|s| {
            s.view
                .current_image
                .as_ref()
                .map(|image| (image.path().to_path_buf(), s.config.language))
        })
    }

    pub fn get_current_view_image_path(&self) -> Result<Option<PathBuf>> {
        self.read_state(|s| s.view.current_image.as_ref().map(|image| image.path().to_path_buf()))
    }

    pub fn get_current_view_image_path_if_viewer(&self) -> Result<Option<PathBuf>> {
        self.read_state(|s| {
            if s.view.view_mode != ViewMode::Viewer {
                return None;
            }
            s.view
                .current_image
                .as_ref()
                .map(|image| image.path().to_path_buf())
        })
    }

    pub fn should_show_info_panel(&self) -> Result<bool> {
        self.read_state(|s| {
            s.config.viewer.show_info_panel
                && s.view.current_image.is_some()
                && s.view.view_mode == ViewMode::Viewer
        })
    }

    pub fn get_current_view_image_info(&self) -> Result<Option<CurrentImageInfo>> {
        self.read_state(|s| {
            s.view.current_image.as_ref().map(|image| {
                (
                    image.path().to_path_buf(),
                    (image.metadata().width, image.metadata().height),
                    format!("{:?}", image.metadata().format),
                )
            })
        })
    }

    pub fn get_gallery_thumbnail_size_if_gallery_mode(&self) -> Result<Option<u32>> {
        self.read_state(|s| {
            if s.view.view_mode == ViewMode::Gallery {
                Some(s.config.gallery.thumbnail_size)
            } else {
                None
            }
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

    pub fn get_gallery_image_path_and_fit_if_viewer(
        &self,
        index: usize,
    ) -> Result<Option<(PathBuf, bool)>> {
        self.read_state(|s| {
            if s.view.view_mode != ViewMode::Viewer {
                return None;
            }
            s.gallery
                .gallery
                .get_image(index)
                .map(|image| (image.path().to_path_buf(), s.config.viewer.fit_to_window))
        })
    }

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

    pub fn update_state(&self, f: impl FnOnce(&mut AppState)) -> Result<()> {
        self.write_state(|state| {
            f(state);
        })
        .map(|_| ())
    }

    pub fn toggle_view_mode(&self) -> Result<()> {
        self.update_state(|state| {
            self.view_use_case.toggle_view_mode(&mut state.view);
        })
    }

    pub fn set_view_mode(&self, mode: ViewMode) -> Result<()> {
        self.update_state(|state| {
            self.view_use_case.set_view_mode(&mut state.view, mode);
        })
    }

    pub fn add_image_to_gallery(&self, path: &Path) -> Result<()> {
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        self.update_state(|state| {
            let image = Image::new(file_name, path.to_path_buf());
            state.gallery.gallery.add_image(image);
        })
    }

    pub fn load_directory(&self, image_source: &dyn ImageSource, path: &Path) -> Result<usize> {
        let mut loaded_result: Result<usize> = Ok(0);
        self.update_state(|state| {
            loaded_result = self
                .navigate_use_case
                .load_directory(&mut state.gallery, image_source, path);
            if let Ok(count) = loaded_result {
                if count > 0 {
                    state.gallery.gallery.select_image(0);
                }
            }
            self.view_use_case
                .set_view_mode(&mut state.view, ViewMode::Gallery);
        })?;
        loaded_result
    }

    pub fn open_image(
        &self,
        path: &Path,
        window_width: Option<f32>,
        window_height: Option<f32>,
        fit_to_window: bool,
    ) -> Result<()> {
        let mut open_result: Result<()> = Ok(());
        self.update_state(|state| {
            open_result = self.view_use_case.open_image(
                path,
                &mut state.view,
                window_width,
                window_height,
                fit_to_window,
            );
        })?;
        open_result
    }

    pub fn navigate_gallery(&self, direction: NavigationDirection) -> Result<Option<usize>> {
        let mut selected = None;
        self.update_state(|state| {
            selected = self
                .navigate_use_case
                .navigate(&mut state.gallery, direction);
        })?;
        Ok(selected)
    }

    pub fn zoom_in(&self, step: f32) -> Result<()> {
        self.update_state(|state| {
            let max = state.config.viewer.max_scale;
            self.view_use_case.zoom_in(&mut state.view, step, max);
        })
    }

    pub fn zoom_out(&self, step: f32) -> Result<()> {
        self.update_state(|state| {
            let min = state.config.viewer.min_scale;
            self.view_use_case.zoom_out(&mut state.view, step, min);
        })
    }

    pub fn reset_zoom(&self) -> Result<()> {
        self.update_state(|state| {
            self.view_use_case.reset_zoom(&mut state.view);
        })
    }

    pub fn fit_to_window(&self, window_width: f32, window_height: f32) -> Result<()> {
        self.update_state(|state| {
            self.view_use_case
                .fit_to_window(&mut state.view, window_width, window_height);
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
}
