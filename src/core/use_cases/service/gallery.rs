use super::OASImageViewerService;
use crate::core::domain::{Image, NavigationDirection, ViewMode};
use crate::core::ports::ImageSource;
use crate::core::Result;
use std::path::{Path, PathBuf};

impl OASImageViewerService {
    pub fn get_gallery_state_for_render(&self) -> Result<super::GalleryState> {
        self.read_state(|s| {
            let mut gallery = s.gallery.clone();
            gallery.layout.thumbnail_size = s.config.gallery.thumbnail_size;
            gallery
        })
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

    pub fn get_gallery_thumbnail_size_if_gallery_mode(&self) -> Result<Option<u32>> {
        self.read_state(|s| {
            if s.view.view_mode == ViewMode::Gallery {
                Some(s.config.gallery.thumbnail_size)
            } else {
                None
            }
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

    pub fn navigate_gallery(&self, direction: NavigationDirection) -> Result<Option<usize>> {
        let mut selected = None;
        self.update_state(|state| {
            selected = self
                .navigate_use_case
                .navigate(&mut state.gallery, direction);
        })?;
        Ok(selected)
    }
}
