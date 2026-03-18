//! 用例层模块导出与聚合入口

mod config;
mod navigate;
mod service;
mod state;
mod view;

pub use config::ManageConfigUseCase;
pub use navigate::NavigateGalleryUseCase;
pub use service::OASImageViewerService;
pub use state::{AppState, GalleryState, ViewState};
pub use view::ViewImageUseCase;

#[cfg(test)]
use crate::core::domain::{
    GalleryLayout, Image, Language, NavigationDirection, Position, Scale, Theme, ViewMode,
    WindowState,
};
#[cfg(test)]
use crate::core::ports::{AppConfig, ImageSource, Storage};
#[cfg(test)]
use crate::core::Result;
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::ImageMetadata;

    #[test]
    fn test_view_state_default() {
        let state = ViewState::default();
        assert!(state.current_image.is_none());
        assert_eq!(state.view_mode, ViewMode::Gallery);
    }

    #[test]
    fn test_gallery_state_default() {
        let state = GalleryState::default();
        assert!(state.gallery.is_empty());
    }

    #[test]
    fn test_navigate_use_case_next() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));

        let result = use_case.navigate(&mut state, NavigationDirection::Next);
        assert!(result.is_some());
    }

    #[test]
    fn test_view_use_case_reset_zoom() {
        // 创建一个 mock image source
        struct MockImageSource;

        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }

            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }

            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }

            fn is_supported(&self, _path: &Path) -> bool {
                true
            }

            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;

        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }

            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }

            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        state.scale.zoom_in(2.0, 10.0);
        assert!(state.scale.value() > 1.0);

        use_case.reset_zoom(&mut state);
        assert!((state.scale.value() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_app_state_clone() {
        let state = AppState {
            view: ViewState::default(),
            gallery: GalleryState::default(),
            config: AppConfig::default(),
        };
        let cloned = state.clone();
        assert_eq!(state.view.view_mode, cloned.view.view_mode);
    }

    // =========================================================================
    // 从旧代码迁移的额外测试
    // =========================================================================

    // ViewImageUseCase 测试
    #[test]
    fn test_view_use_case_zoom_in() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        let initial_scale = state.scale.value();
        use_case.zoom_in(&mut state, 1.25, 20.0);
        assert!(state.scale.value() > initial_scale);
        assert!(state.user_zoomed);
    }

    #[test]
    fn test_view_use_case_zoom_out() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        // 先放大
        use_case.zoom_in(&mut state, 2.0, 20.0);
        let zoomed_scale = state.scale.value();

        // 再缩小
        use_case.zoom_out(&mut state, 2.0, 0.1);
        assert!(state.scale.value() < zoomed_scale);
    }

    #[test]
    fn test_view_use_case_zoom_limits() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        // 放大到最大
        for _ in 0..50 {
            use_case.zoom_in(&mut state, 1.5, 5.0);
        }
        assert!(state.scale.value() <= 5.0);

        // 缩小到最小
        for _ in 0..50 {
            use_case.zoom_out(&mut state, 1.5, 0.5);
        }
        assert!(state.scale.value() >= 0.5);
    }

    #[test]
    fn test_view_use_case_fit_to_window() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();
        // 设置当前图片用于测试 fit_to_window
        let mut image = Image::new("test", "/test.jpg");
        let metadata = ImageMetadata {
            width: 100,
            height: 100,
            ..Default::default()
        };
        image.set_metadata(metadata);
        state.current_image = Some(image);

        // 先缩放和移动
        state.scale.zoom_in(2.0, 10.0);
        state.offset.translate(100.0, 100.0);
        state.user_zoomed = true;

        // 适应窗口后应重置缩放与偏移
        use_case.fit_to_window(&mut state, 800.0, 600.0);
        assert!((state.scale.value() - 1.0).abs() < 0.001);
        assert_eq!(state.offset.x, 0.0);
        assert_eq!(state.offset.y, 0.0);
        assert!(!state.user_zoomed);
    }

    #[test]
    fn test_view_use_case_fit_to_width() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();
        let mut image = Image::new("test", "/test.jpg");
        let metadata = ImageMetadata {
            width: 2000,
            height: 1000,
            ..Default::default()
        };
        image.set_metadata(metadata);
        state.current_image = Some(image);
        state.scale.zoom_in(2.0, 10.0);
        state.offset.translate(50.0, 25.0);
        state.user_zoomed = true;

        use_case.fit_to_width(&mut state, 1000.0);
        assert!((state.scale.value() - 0.5).abs() < 0.001);
        assert_eq!(state.offset.x, 0.0);
        assert_eq!(state.offset.y, 0.0);
        assert!(!state.user_zoomed);
    }

    #[test]
    fn test_view_use_case_fit_to_height() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();
        let mut image = Image::new("test", "/test.jpg");
        let metadata = ImageMetadata {
            width: 1000,
            height: 2000,
            ..Default::default()
        };
        image.set_metadata(metadata);
        state.current_image = Some(image);
        state.scale.zoom_in(2.0, 10.0);
        state.offset.translate(50.0, 25.0);
        state.user_zoomed = true;

        use_case.fit_to_height(&mut state, 1000.0);
        assert!((state.scale.value() - 0.5).abs() < 0.001);
        assert_eq!(state.offset.x, 0.0);
        assert_eq!(state.offset.y, 0.0);
        assert!(!state.user_zoomed);
    }

    #[test]
    fn test_view_use_case_pan() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        use_case.pan(&mut state, 50.0, 100.0);
        assert_eq!(state.offset.x, 50.0);
        assert_eq!(state.offset.y, 100.0);

        use_case.pan(&mut state, -20.0, -30.0);
        assert_eq!(state.offset.x, 30.0);
        assert_eq!(state.offset.y, 70.0);
    }

    #[test]
    fn test_view_use_case_toggle_view_mode() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        assert_eq!(state.view_mode, ViewMode::Gallery);

        use_case.toggle_view_mode(&mut state);
        assert_eq!(state.view_mode, ViewMode::Viewer);

        use_case.toggle_view_mode(&mut state);
        assert_eq!(state.view_mode, ViewMode::Gallery);
    }

    #[test]
    fn test_view_use_case_set_view_mode() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let mut state = ViewState::default();

        use_case.set_view_mode(&mut state, ViewMode::Viewer);
        assert_eq!(state.view_mode, ViewMode::Viewer);

        use_case.set_view_mode(&mut state, ViewMode::Gallery);
        assert_eq!(state.view_mode, ViewMode::Gallery);
    }

    #[test]
    fn test_view_use_case_close_image() {
        struct MockImageSource;
        impl ImageSource for MockImageSource {
            fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
                Ok(ImageMetadata::default())
            }
            fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
            fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
                Ok(vec![])
            }
            fn is_supported(&self, _path: &Path) -> bool {
                true
            }
            fn generate_thumbnail(
                &self,
                _path: &Path,
                _max_size: u32,
            ) -> Result<(u32, u32, Vec<u8>)> {
                Ok((100, 100, vec![0u8; 40000]))
            }
        }

        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        // 模拟有图片状态
        let mut state = ViewState {
            current_image: Some(Image::new("1", "/test.png")),
            scale: Scale::new(2.0, 0.1, 10.0),
            offset: Position::new(100.0, 100.0),
            user_zoomed: true,
            ..ViewState::default()
        };

        use_case.close_image(&mut state);

        assert!(state.current_image.is_none());
        assert!((state.scale.value() - 1.0).abs() < 0.001);
        assert_eq!(state.offset.x, 0.0);
        assert_eq!(state.offset.y, 0.0);
        assert!(!state.user_zoomed);
    }

    // NavigateGalleryUseCase 测试
    #[test]
    fn test_navigate_use_case_navigate_to() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));
        state.gallery.add_image(Image::new("3", "/c.png"));

        assert!(use_case.navigate_to(&mut state, 0).unwrap());
        assert_eq!(state.gallery.selected_index(), Some(0));

        assert!(use_case.navigate_to(&mut state, 2).unwrap());
        assert_eq!(state.gallery.selected_index(), Some(2));

        assert!(use_case.navigate_to(&mut state, 5).is_err());
    }

    #[test]
    fn test_navigate_use_case_navigate_previous() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));
        state.gallery.select_image(1);

        let result = use_case.navigate(&mut state, NavigationDirection::Previous);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_navigate_use_case_navigate_first() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));
        state.gallery.add_image(Image::new("3", "/c.png"));
        state.gallery.select_image(2);

        let result = use_case.navigate(&mut state, NavigationDirection::First);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_navigate_use_case_navigate_last() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));
        state.gallery.add_image(Image::new("3", "/c.png"));

        let result = use_case.navigate(&mut state, NavigationDirection::Last);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_navigate_use_case_add_image() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        use_case.add_image(&mut state, Image::new("1", "/a.png"));
        assert_eq!(state.gallery.len(), 1);

        use_case.add_image(&mut state, Image::new("2", "/b.png"));
        assert_eq!(state.gallery.len(), 2);
    }

    #[test]
    fn test_navigate_use_case_remove_image() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));

        let removed = use_case.remove_image(&mut state, 0);
        assert!(removed.is_some());
        assert_eq!(state.gallery.len(), 1);

        let removed = use_case.remove_image(&mut state, 5);
        assert!(removed.is_none());
    }

    #[test]
    fn test_navigate_use_case_find_by_path() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        state.gallery.add_image(Image::new("1", "/a.png"));
        state.gallery.add_image(Image::new("2", "/b.png"));

        assert_eq!(use_case.find_by_path(&state, Path::new("/a.png")), Some(0));
        assert_eq!(use_case.find_by_path(&state, Path::new("/b.png")), Some(1));
        assert_eq!(use_case.find_by_path(&state, Path::new("/c.png")), None);
    }

    #[test]
    fn test_navigate_use_case_update_layout() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();

        let new_layout = GalleryLayout {
            thumbnail_size: 150,
            items_per_row: 5,
            grid_spacing: 15.0,
            show_filenames: false,
        };

        use_case.update_layout(&mut state, new_layout);
        assert_eq!(state.layout.thumbnail_size, 150);
        assert_eq!(state.layout.items_per_row, 5);
    }

    #[test]
    fn test_navigate_use_case_calculate_items_per_row() {
        let use_case = NavigateGalleryUseCase;
        let mut state = GalleryState::default();
        state.layout.thumbnail_size = 100;
        state.layout.grid_spacing = 10.0;

        let items = use_case.calculate_items_per_row(&state, 500.0);
        assert_eq!(items, 4);
    }

    // ManageConfigUseCase 测试
    #[test]
    fn test_manage_config_use_case_new() {
        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ManageConfigUseCase::new(Arc::new(MockStorage));
        drop(use_case);
    }

    #[test]
    fn test_manage_config_use_case_update_window_state() {
        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ManageConfigUseCase::new(Arc::new(MockStorage));
        let mut config = AppConfig::default();

        let new_state = WindowState {
            width: 1920.0,
            height: 1080.0,
            x: Some(100.0),
            y: Some(100.0),
            maximized: true,
        };

        use_case.update_window_state(&mut config, new_state);
        assert_eq!(config.window.width, 1920.0);
        assert_eq!(config.window.height, 1080.0);
        assert!(config.window.maximized);
    }

    #[test]
    fn test_manage_config_use_case_set_last_directory() {
        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ManageConfigUseCase::new(Arc::new(MockStorage));
        let mut config = AppConfig::default();

        use_case.set_last_directory(&mut config, PathBuf::from("/home/user/images"));
        assert_eq!(
            config.last_opened_directory,
            Some(PathBuf::from("/home/user/images"))
        );
    }

    #[test]
    fn test_manage_config_use_case_validate_config() {
        struct MockStorage;
        impl Storage for MockStorage {
            fn load_config(&self) -> Result<AppConfig> {
                Ok(AppConfig::default())
            }
            fn save_config(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
            fn request_save(&self, _config: &AppConfig) -> Result<()> {
                Ok(())
            }
        }

        let use_case = ManageConfigUseCase::new(Arc::new(MockStorage));
        let mut config = AppConfig::default();

        config.gallery.thumbnail_size = 50; // 太小
        config.viewer.zoom_step = 0.5; // 太小

        let validated = use_case.validate_config(&config);
        assert!(validated.gallery.thumbnail_size >= 60);
        assert!(validated.viewer.zoom_step >= 1.01);
    }

    struct TestImageSource;
    impl ImageSource for TestImageSource {
        fn load_metadata(&self, _path: &Path) -> Result<ImageMetadata> {
            Ok(ImageMetadata::default())
        }
        fn load_image_data(&self, _path: &Path) -> Result<(u32, u32, Vec<u8>)> {
            Ok((100, 100, vec![0u8; 40000]))
        }
        fn scan_directory(&self, _path: &Path) -> Result<Vec<PathBuf>> {
            Ok(vec![])
        }
        fn is_supported(&self, _path: &Path) -> bool {
            true
        }
        fn generate_thumbnail(&self, _path: &Path, _max_size: u32) -> Result<(u32, u32, Vec<u8>)> {
            Ok((100, 100, vec![0u8; 40000]))
        }
    }

    struct TestStorage;
    impl Storage for TestStorage {
        fn load_config(&self) -> Result<AppConfig> {
            Ok(AppConfig::default())
        }
        fn save_config(&self, _config: &AppConfig) -> Result<()> {
            Ok(())
        }
        fn request_save(&self, _config: &AppConfig) -> Result<()> {
            Ok(())
        }
    }

    fn build_test_service() -> OASImageViewerService {
        let view_use_case = ViewImageUseCase::new(Arc::new(TestImageSource), Arc::new(TestStorage));
        let navigate_use_case = NavigateGalleryUseCase;
        let config_use_case = ManageConfigUseCase::new(Arc::new(TestStorage));
        OASImageViewerService::new(view_use_case, navigate_use_case, config_use_case)
    }

    #[test]
    fn test_oas_image_viewer_service_new() {
        let service = build_test_service();

        let state = service.get_state().unwrap();
        assert_eq!(state.view.view_mode, ViewMode::Gallery);
    }

    #[test]
    fn test_oas_image_viewer_service_update_state() {
        let service = build_test_service();

        service
            .update_state(|state| {
                state.view.view_mode = ViewMode::Viewer;
            })
            .unwrap();

        let state = service.get_state().unwrap();
        assert_eq!(state.view.view_mode, ViewMode::Viewer);
    }

    #[test]
    fn test_oas_image_viewer_service_get_selected_gallery_image_for_open() {
        let service = build_test_service();
        service
            .update_state(|state| {
                state.gallery.gallery.add_image(Image::new("a", "/a.png"));
                state.gallery.gallery.select_image(0);
                state.view.view_mode = ViewMode::Gallery;
                state.config.viewer.fit_to_window = false;
            })
            .unwrap();

        let selected = service.get_selected_gallery_image_for_open().unwrap();
        assert_eq!(
            selected,
            Some((PathBuf::from("/a.png"), false))
        );
    }

    #[test]
    fn test_oas_image_viewer_service_get_current_view_image_path_if_viewer() {
        let service = build_test_service();
        service
            .update_state(|state| {
                state.view.current_image = Some(Image::new("a", "/a.png"));
                state.view.view_mode = ViewMode::Viewer;
            })
            .unwrap();

        let path = service.get_current_view_image_path_if_viewer().unwrap();
        assert_eq!(path, Some(PathBuf::from("/a.png")));
    }

    #[test]
    fn test_oas_image_viewer_service_update_and_get_config() {
        let service = build_test_service();
        service
            .update_config(|config| {
                config.language = Language::English;
                config.theme = Theme::Light;
            })
            .unwrap();

        assert_eq!(service.get_language().unwrap(), Language::English);
        assert_eq!(service.get_theme().unwrap(), Theme::Light);
    }

    // ViewState 测试
    #[test]
    fn test_view_state_default_values() {
        let state = ViewState::default();
        assert!(state.current_image.is_none());
        assert!((state.scale.value() - 1.0).abs() < 0.001);
        assert_eq!(state.offset.x, 0.0);
        assert_eq!(state.offset.y, 0.0);
        assert_eq!(state.view_mode, ViewMode::Gallery);
        assert!(!state.user_zoomed);
    }

    // GalleryState 测试
    #[test]
    fn test_gallery_state_default_values() {
        let state = GalleryState::default();
        assert!(state.gallery.is_empty());
        assert_eq!(state.layout.thumbnail_size, 100);
        assert_eq!(state.items_per_row, 0);
    }

    // AppState 测试
    #[test]
    fn test_app_state_clone_consistency() {
        let state = AppState {
            view: ViewState::default(),
            gallery: GalleryState::default(),
            config: AppConfig::default(),
        };

        let cloned = state.clone();
        assert_eq!(state.view.view_mode, cloned.view.view_mode);
        assert_eq!(state.gallery.items_per_row, cloned.gallery.items_per_row);
    }
}
