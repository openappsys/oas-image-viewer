//! Use Cases - 业务用例/服务
//!
//! 协调领域对象和端口，实现核心业务逻辑

use crate::core::domain::{
    Gallery, GalleryLayout, Image, NavigationDirection, Position, Scale, ViewMode, ViewerSettings,
    WindowState,
};
use crate::core::ports::{AppConfig, ImageSource, Storage};
use crate::core::{CoreError, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 图像查看用例
pub struct ViewImageUseCase {
    image_source: Arc<dyn ImageSource>,
    #[allow(dead_code)]
    storage: Arc<dyn Storage>,
}

/// 画廊导航用例
pub struct NavigateGalleryUseCase;

/// 配置管理用例
pub struct ManageConfigUseCase {
    storage: Arc<dyn Storage>,
}

/// 图像查看状态
#[derive(Debug, Clone)]
pub struct ViewState {
    pub current_image: Option<Image>,
    pub scale: Scale,
    pub offset: Position,
    pub view_mode: ViewMode,
    pub user_zoomed: bool,
}

/// 画廊导航状态
#[derive(Debug, Clone)]
pub struct GalleryState {
    pub gallery: Gallery,
    pub layout: GalleryLayout,
    pub items_per_row: usize,
}

/// 应用程序状态
#[derive(Debug, Clone)]
pub struct AppState {
    pub view: ViewState,
    pub gallery: GalleryState,
    pub config: AppConfig,
}

impl ViewImageUseCase {
    /// 创建新的图像查看用例
    pub fn new(image_source: Arc<dyn ImageSource>, storage: Arc<dyn Storage>) -> Self {
        Self {
            image_source,
            storage,
        }
    }

    /// 打开图像
    ///
    /// # Arguments
    /// * `path` - 图像文件路径
    /// * `state` - 视图状态
    /// * `window_width` - 窗口宽度（用于计算适应窗口的缩放）
    /// * `window_height` - 窗口高度
    /// * `fit_to_window` - 是否适应窗口
    pub fn open_image(
        &self,
        path: &Path,
        state: &mut ViewState,
        window_width: Option<f32>,
        window_height: Option<f32>,
        fit_to_window: bool,
    ) -> Result<()> {
        if !self.image_source.is_supported(path) {
            return Err(CoreError::InvalidImageFormat(
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

        // 根据 fit_to_window 设置计算缩放比例
        if fit_to_window {
            if let (Some(win_w), Some(win_h)) = (window_width, window_height) {
                let img_w = metadata.width as f32;
                let img_h = metadata.height as f32;

                // 计算适应窗口的缩放比例（保持宽高比）
                let scale_x = win_w / img_w;
                let scale_y = win_h / img_h;
                let fit_scale = scale_x.min(scale_y).min(1.0); // 不超过原始尺寸

                // 使用默认值作为范围限制
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

    /// 打开图像并加载数据
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

    /// 关闭当前图像
    pub fn close_image(&self, state: &mut ViewState) {
        state.current_image = None;
        state.scale.reset();
        state.offset.reset();
        state.user_zoomed = false;
    }

    /// 缩放图像
    pub fn zoom(&self, state: &mut ViewState, factor: f32, min: f32, max: f32) {
        if factor > 1.0 {
            state.scale.zoom_in(factor, max);
        } else {
            state.scale.zoom_out(1.0 / factor, min);
        }
        state.user_zoomed = true;
    }

    /// 放大
    pub fn zoom_in(&self, state: &mut ViewState, step: f32, max: f32) {
        state.scale.zoom_in(step, max);
        state.user_zoomed = true;
    }

    /// 缩小
    pub fn zoom_out(&self, state: &mut ViewState, step: f32, min: f32) {
        state.scale.zoom_out(step, min);
        state.user_zoomed = true;
    }

    /// 重置缩放
    pub fn reset_zoom(&self, state: &mut ViewState) {
        state.scale.reset();
        state.offset.reset();
        state.user_zoomed = true;
    }

    /// 适应窗口
    pub fn fit_to_window(&self, state: &mut ViewState) {
        state.scale.reset();
        state.offset.reset();
        state.user_zoomed = false;
    }

    /// 根据窗口尺寸计算适应窗口的缩放比例
    ///
    /// # Arguments
    /// * `image_width` - 图像宽度
    /// * `image_height` - 图像高度
    /// * `window_width` - 窗口宽度
    /// * `window_height` - 窗口高度
    ///
    /// # Returns
    /// 适应窗口的缩放比例（不超过1.0，保持宽高比）
    pub fn calculate_fit_scale(
        image_width: u32,
        image_height: u32,
        window_width: f32,
        window_height: f32,
    ) -> f32 {
        let img_w = image_width as f32;
        let img_h = image_height as f32;

        // 计算适应窗口的缩放比例（保持宽高比）
        let scale_x = window_width / img_w;
        let scale_y = window_height / img_h;
        scale_x.min(scale_y).min(1.0) // 不超过原始尺寸
    }

    /// 平移图像
    pub fn pan(&self, state: &mut ViewState, delta_x: f32, delta_y: f32) {
        state.offset.translate(delta_x, delta_y);
    }

    /// 切换视图模式
    pub fn toggle_view_mode(&self, state: &mut ViewState) {
        state.view_mode = match state.view_mode {
            ViewMode::Gallery => ViewMode::Viewer,
            ViewMode::Viewer => ViewMode::Gallery,
        };
    }

    /// 设置视图模式
    pub fn set_view_mode(&self, state: &mut ViewState, mode: ViewMode) {
        state.view_mode = mode;
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            current_image: None,
            scale: Scale::new(1.0, 0.1, 20.0),
            offset: Position::default(),
            view_mode: ViewMode::Gallery,
            user_zoomed: false,
        }
    }
}

impl NavigateGalleryUseCase {
    /// 导航到指定图像，返回是否成功
    pub fn navigate_to(&self, state: &mut GalleryState, index: usize) -> Result<bool> {
        if state.gallery.select_image(index) {
            Ok(true)
        } else {
            Err(CoreError::NavigationError(format!(
                "Invalid index: {}",
                index
            )))
        }
    }

    /// 按方向导航，返回新选中索引
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

    /// 在网格中上下导航，返回新选中索引
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

    /// 加载目录到画廊
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

    /// 添加图像到画廊
    pub fn add_image(&self, state: &mut GalleryState, image: Image) {
        state.gallery.add_image(image);
    }

    /// 从画廊移除图像
    pub fn remove_image(&self, state: &mut GalleryState, index: usize) -> Option<Image> {
        state.gallery.remove_image(index)
    }

    /// 通过路径查找图像
    pub fn find_by_path(&self, state: &GalleryState, path: &Path) -> Option<usize> {
        state.gallery.index_by_path(path)
    }

    /// 更新布局
    pub fn update_layout(&self, state: &mut GalleryState, layout: GalleryLayout) {
        state.layout = layout.validated();
    }

    /// 基于可用宽度计算每行项目数
    pub fn calculate_items_per_row(&self, state: &GalleryState, available_width: f32) -> usize {
        state.layout.calculate_items_per_row(available_width)
    }
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            gallery: Gallery::new("Default"),
            layout: GalleryLayout::default(),
            items_per_row: 0,
        }
    }
}

impl ManageConfigUseCase {
    /// 创建新的配置管理用例
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// 加载配置
    pub fn load_config(&self) -> Result<AppConfig> {
        self.storage.load_config()
    }

    /// 保存配置
    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        self.storage.save_config(config)
    }

    /// 请求保存配置（防抖）
    pub fn request_save(&self, config: &AppConfig) -> Result<()> {
        self.storage.request_save(config)
    }

    /// 更新窗口状态
    pub fn update_window_state(&self, config: &mut AppConfig, state: WindowState) {
        config.window = state;
    }

    /// 更新画廊布局
    pub fn update_gallery_layout(&self, config: &mut AppConfig, layout: GalleryLayout) {
        config.gallery = layout.validated();
    }

    /// 更新查看器设置
    pub fn update_viewer_settings(&self, config: &mut AppConfig, settings: ViewerSettings) {
        config.viewer = settings.validated();
    }

    /// 设置最后打开的目录
    pub fn set_last_directory(&self, config: &mut AppConfig, path: PathBuf) {
        config.last_opened_directory = Some(path);
    }

    /// 验证配置
    pub fn validate_config(&self, config: &AppConfig) -> AppConfig {
        AppConfig {
            window: config.window,
            gallery: config.gallery.validated(),
            viewer: config.viewer.validated(),
            last_opened_directory: config.last_opened_directory.clone(),
        }
    }
}

/// 应用程序服务 - 协调所有用例
pub struct ImageViewerService {
    pub view_use_case: ViewImageUseCase,
    pub navigate_use_case: NavigateGalleryUseCase,
    pub config_use_case: ManageConfigUseCase,
    pub state: Mutex<AppState>,
}

impl ImageViewerService {
    /// 创建新的应用服务
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

    /// 初始化配置
    /// 如果传入了 config 则使用它，否则从存储加载
    pub fn initialize(&self, config: Option<AppConfig>) -> Result<()> {
        let config = if let Some(cfg) = config {
            cfg
        } else {
            self.config_use_case.load_config()?
        };

        let mut state = self
            .state
            .lock()
            .map_err(|_| CoreError::ConfigError("Lock poisoned".to_string()))?;
        state.config = config;
        state.gallery.layout = state.config.gallery;
        Ok(())
    }

    /// 获取当前状态
    pub fn get_state(&self) -> Result<AppState> {
        self.state
            .lock()
            .map_err(|_| CoreError::ConfigError("Lock poisoned".to_string()))
            .map(|s| s.clone())
    }

    /// 更新状态
    pub fn update_state(&self, f: impl FnOnce(&mut AppState)) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CoreError::ConfigError("Lock poisoned".to_string()))?;
        f(&mut state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{ImageFormat, ImageMetadata};

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

        // 先缩放和移动
        state.scale.zoom_in(2.0, 10.0);
        state.offset.translate(100.0, 100.0);
        state.user_zoomed = true;

        // fit_to_window 应该重置
        use_case.fit_to_window(&mut state);
        assert!((state.scale.value() - 1.0).abs() < 0.001);
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
        let mut state = ViewState::default();

        // 模拟有图片状态
        state.current_image = Some(Image::new("1", "/test.png"));
        state.scale.zoom_in(2.0, 10.0);
        state.offset.translate(100.0, 100.0);
        state.user_zoomed = true;

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
        assert!(validated.gallery.thumbnail_size >= 80);
        assert!(validated.viewer.zoom_step >= 1.01);
    }

    // ImageViewerService 测试
    #[test]
    fn test_image_viewer_service_new() {
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

        let view_use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let navigate_use_case = NavigateGalleryUseCase;
        let config_use_case = ManageConfigUseCase::new(Arc::new(MockStorage));

        let service = ImageViewerService::new(view_use_case, navigate_use_case, config_use_case);

        let state = service.get_state().unwrap();
        assert_eq!(state.view.view_mode, ViewMode::Gallery);
    }

    #[test]
    fn test_image_viewer_service_update_state() {
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

        let view_use_case = ViewImageUseCase::new(Arc::new(MockImageSource), Arc::new(MockStorage));
        let navigate_use_case = NavigateGalleryUseCase;
        let config_use_case = ManageConfigUseCase::new(Arc::new(MockStorage));

        let service = ImageViewerService::new(view_use_case, navigate_use_case, config_use_case);

        service
            .update_state(|state| {
                state.view.view_mode = ViewMode::Viewer;
            })
            .unwrap();

        let state = service.get_state().unwrap();
        assert_eq!(state.view.view_mode, ViewMode::Viewer);
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
        assert_eq!(state.layout.thumbnail_size, 120);
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
