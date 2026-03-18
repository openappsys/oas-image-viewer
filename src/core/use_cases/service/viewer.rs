use super::{CurrentImageInfo, OASImageViewerService};
use crate::core::domain::{Language, ViewMode, ViewerSettings};
use crate::core::use_cases::ViewState;
use crate::core::Result;
use std::path::{Path, PathBuf};

impl OASImageViewerService {
    pub fn get_view_mode(&self) -> Result<ViewMode> {
        self.read_state(|s| s.view.view_mode)
    }

    pub fn get_view_state(&self) -> Result<ViewState> {
        self.read_state(|s| s.view.clone())
    }

    pub fn get_view_state_and_settings(&self) -> Result<(ViewState, ViewerSettings)> {
        self.read_state(|s| (s.view.clone(), s.config.viewer))
    }

    pub fn set_view_state(&self, view: ViewState) -> Result<()> {
        self.update_state(|state| {
            state.view = view;
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

    pub fn fit_to_width(&self, window_width: f32) -> Result<()> {
        self.update_state(|state| {
            self.view_use_case.fit_to_width(&mut state.view, window_width);
        })
    }

    pub fn fit_to_height(&self, window_height: f32) -> Result<()> {
        self.update_state(|state| {
            self.view_use_case.fit_to_height(&mut state.view, window_height);
        })
    }
}
