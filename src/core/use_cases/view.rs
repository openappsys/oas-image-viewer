//! 图像查看用例与缩放/视图模式逻辑

use crate::core::domain::{Image, Scale, ViewMode};
use crate::core::ports::{ImageSource, Storage};
use crate::core::{CoreError, Result};
use std::path::Path;
use std::sync::Arc;

use super::ViewState;

pub struct ViewImageUseCase {
    image_source: Arc<dyn ImageSource>,
}

impl ViewImageUseCase {
    pub fn new(image_source: Arc<dyn ImageSource>, _storage: Arc<dyn Storage>) -> Self {
        Self { image_source }
    }

    pub fn open_image(
        &self,
        path: &Path,
        state: &mut ViewState,
        window_width: Option<f32>,
        window_height: Option<f32>,
        fit_to_window: bool,
    ) -> Result<()> {
        if !self.image_source.is_supported(path) {
            return Err(CoreError::technical(
                "INVALID_IMAGE_FORMAT",
                path.to_string_lossy().to_string(),
            ));
        }

        let metadata = self.image_source.load_metadata(path)?;
        let mut image = Image::new(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown"),
            path,
        );
        image.set_metadata(metadata.clone());

        state.current_image = Some(image);

        if fit_to_window {
            if let (Some(win_w), Some(win_h)) = (window_width, window_height) {
                let img_w = metadata.width as f32;
                let img_h = metadata.height as f32;
                let scale_x = win_w / img_w;
                let scale_y = win_h / img_h;
                let fit_scale = scale_x.min(scale_y).min(1.0);
                state.scale = Scale::new(fit_scale, 0.1, 20.0);
            } else {
                state.scale.reset();
            }
        } else {
            state.scale.reset();
        }

        state.offset.reset();
        state.user_zoomed = false;
        state.view_mode = ViewMode::Viewer;

        Ok(())
    }

    pub fn open_image_with_data(
        &self,
        path: &Path,
        state: &mut ViewState,
        window_width: Option<f32>,
        window_height: Option<f32>,
        fit_to_window: bool,
    ) -> Result<(u32, u32, Vec<u8>)> {
        self.open_image(path, state, window_width, window_height, fit_to_window)?;
        self.image_source.load_image_data(path)
    }

    pub fn close_image(&self, state: &mut ViewState) {
        state.current_image = None;
        state.scale.reset();
        state.offset.reset();
        state.user_zoomed = false;
    }

    pub fn zoom(&self, state: &mut ViewState, factor: f32, min: f32, max: f32) {
        if factor > 1.0 {
            state.scale.zoom_in(factor, max);
        } else {
            state.scale.zoom_out(1.0 / factor, min);
        }
        state.user_zoomed = true;
    }

    pub fn zoom_in(&self, state: &mut ViewState, step: f32, max: f32) {
        state.scale.zoom_in(step, max);
        state.user_zoomed = true;
    }

    pub fn zoom_out(&self, state: &mut ViewState, step: f32, min: f32) {
        state.scale.zoom_out(step, min);
        state.user_zoomed = true;
    }

    pub fn reset_zoom(&self, state: &mut ViewState) {
        state.scale.reset();
        state.offset.reset();
        state.user_zoomed = true;
    }

    pub fn fit_to_window(&self, state: &mut ViewState, window_width: f32, window_height: f32) {
        self.apply_fit_scale(state, window_width, window_height, FitStrategy::Window);
    }

    pub fn fit_to_width(&self, state: &mut ViewState, window_width: f32) {
        self.apply_fit_scale(state, window_width, 0.0, FitStrategy::Width);
    }

    pub fn fit_to_height(&self, state: &mut ViewState, window_height: f32) {
        self.apply_fit_scale(state, 0.0, window_height, FitStrategy::Height);
    }

    fn apply_fit_scale(
        &self,
        state: &mut ViewState,
        window_width: f32,
        window_height: f32,
        strategy: FitStrategy,
    ) {
        if let Some(ref image) = state.current_image {
            let fit_scale = Self::calculate_fit_scale_by_strategy(
                image.metadata().width,
                image.metadata().height,
                window_width,
                window_height,
                strategy,
            );
            state.scale = Scale::new(fit_scale, 0.1, 20.0);
        }
        state.offset.reset();
        state.user_zoomed = false;
    }

    pub fn calculate_fit_scale(
        image_width: u32,
        image_height: u32,
        window_width: f32,
        window_height: f32,
    ) -> f32 {
        Self::calculate_fit_scale_by_strategy(
            image_width,
            image_height,
            window_width,
            window_height,
            FitStrategy::Window,
        )
    }

    pub fn calculate_fit_width_scale(image_width: u32, window_width: f32) -> f32 {
        Self::calculate_fit_scale_by_strategy(image_width, 1, window_width, 0.0, FitStrategy::Width)
    }

    pub fn calculate_fit_height_scale(image_height: u32, window_height: f32) -> f32 {
        Self::calculate_fit_scale_by_strategy(1, image_height, 0.0, window_height, FitStrategy::Height)
    }

    fn calculate_fit_scale_by_strategy(
        image_width: u32,
        image_height: u32,
        window_width: f32,
        window_height: f32,
        strategy: FitStrategy,
    ) -> f32 {
        let img_w = image_width.max(1) as f32;
        let img_h = image_height.max(1) as f32;
        match strategy {
            FitStrategy::Window => {
                let scale_x = window_width / img_w;
                let scale_y = window_height / img_h;
                scale_x.min(scale_y).min(1.0)
            }
            FitStrategy::Width => (window_width / img_w).min(1.0),
            FitStrategy::Height => (window_height / img_h).min(1.0),
        }
    }

    pub fn pan(&self, state: &mut ViewState, delta_x: f32, delta_y: f32) {
        state.offset.translate(delta_x, delta_y);
    }

    pub fn toggle_view_mode(&self, state: &mut ViewState) {
        state.view_mode = match state.view_mode {
            ViewMode::Gallery => ViewMode::Viewer,
            ViewMode::Viewer => ViewMode::Gallery,
        };
    }

    pub fn set_view_mode(&self, state: &mut ViewState, mode: ViewMode) {
        state.view_mode = mode;
    }
}

#[derive(Clone, Copy)]
enum FitStrategy {
    Window,
    Width,
    Height,
}
