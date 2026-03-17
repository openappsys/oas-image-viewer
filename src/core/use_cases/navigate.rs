//! 画廊导航用例与目录装载逻辑

use crate::core::domain::{GalleryLayout, Image, NavigationDirection};
use crate::core::ports::ImageSource;
use crate::core::{CoreError, Result};
use std::path::Path;

use super::GalleryState;

pub struct NavigateGalleryUseCase;

impl NavigateGalleryUseCase {
    pub fn navigate_to(&self, state: &mut GalleryState, index: usize) -> Result<bool> {
        if state.gallery.select_image(index) {
            Ok(true)
        } else {
            Err(CoreError::technical(
                "NAVIGATION_ERROR",
                format!("Invalid index: {}", index),
            ))
        }
    }

    pub fn navigate(
        &self,
        state: &mut GalleryState,
        direction: NavigationDirection,
    ) -> Option<usize> {
        let success = match direction {
            NavigationDirection::Next => state.gallery.select_next(),
            NavigationDirection::Previous => state.gallery.select_prev(),
            NavigationDirection::First => {
                if !state.gallery.is_empty() {
                    state.gallery.select_image(0)
                } else {
                    false
                }
            }
            NavigationDirection::Last => {
                let len = state.gallery.len();
                if len > 0 {
                    state.gallery.select_image(len - 1)
                } else {
                    false
                }
            }
        };

        if success {
            state.gallery.selected_index()
        } else {
            None
        }
    }

    pub fn navigate_grid(
        &self,
        state: &mut GalleryState,
        direction: NavigationDirection,
    ) -> Option<usize> {
        let success = match direction {
            NavigationDirection::Next => state.gallery.select_next(),
            NavigationDirection::Previous => state.gallery.select_prev(),
            _ => return self.navigate(state, direction),
        };

        if success {
            state.gallery.selected_index()
        } else {
            None
        }
    }

    pub fn load_directory(
        &self,
        state: &mut GalleryState,
        image_source: &dyn ImageSource,
        path: &Path,
    ) -> Result<usize> {
        let paths = image_source.scan_directory(path)?;
        let count = paths.len();

        state.gallery.clear();
        for (idx, path) in paths.into_iter().enumerate() {
            let image = Image::new(format!("gallery_img_{}", idx), path);
            state.gallery.add_image(image);
        }

        Ok(count)
    }

    pub fn add_image(&self, state: &mut GalleryState, image: Image) {
        state.gallery.add_image(image);
    }

    pub fn remove_image(&self, state: &mut GalleryState, index: usize) -> Option<Image> {
        state.gallery.remove_image(index)
    }

    pub fn find_by_path(&self, state: &GalleryState, path: &Path) -> Option<usize> {
        state.gallery.index_by_path(path)
    }

    pub fn update_layout(&self, state: &mut GalleryState, layout: GalleryLayout) {
        state.layout = layout.validated();
    }

    pub fn calculate_items_per_row(&self, state: &GalleryState, available_width: f32) -> usize {
        state.layout.calculate_items_per_row(available_width)
    }
}
