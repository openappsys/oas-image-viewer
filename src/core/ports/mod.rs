//! Ports - 端口接口定义
//!
//! Core 层通过这些接口与外部世界交互
//! 实现依赖倒置原则：Core 层定义接口，外层实现接口

use crate::core::domain::{GalleryLayout, Image, ImageMetadata, ViewerSettings, WindowState};
use crate::core::Result;
use std::path::{Path, PathBuf};

/// 图像数据源端口
///
/// 负责图像的加载和元数据获取
pub trait ImageSource: Send + Sync {
    /// 从文件路径加载图像元数据
    fn load_metadata(&self, path: &Path) -> Result<ImageMetadata>;

    /// 加载图像原始数据
    ///
    /// 返回 (width, height, rgba_data)
    fn load_image_data(&self, path: &Path) -> Result<(u32, u32, Vec<u8>)>;

    /// 扫描目录获取图像文件列表
    fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// 检查文件是否为支持的图像
    fn is_supported(&self, path: &Path) -> bool;

    /// 生成缩略图数据
    ///
    /// 返回 (width, height, rgba_data)
    fn generate_thumbnail(&self, path: &Path, max_size: u32) -> Result<(u32, u32, Vec<u8>)>;
}

/// 存储端口
///
/// 负责配置的持久化
pub trait Storage: Send + Sync {
    /// 加载配置
    fn load_config(&self) -> Result<AppConfig>;

    /// 保存配置
    fn save_config(&self, config: &AppConfig) -> Result<()>;

    /// 配置变更时调用（支持防抖）
    fn request_save(&self, config: &AppConfig) -> Result<()>;
}

/// 应用配置数据结构
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub window: WindowState,
    pub gallery: GalleryLayout,
    pub viewer: ViewerSettings,
    pub last_opened_directory: Option<PathBuf>,
}

/// UI 端口
///
/// 负责与用户界面的交互
pub trait UiPort: Send + Sync {
    /// 请求更新显示
    fn request_repaint(&self);

    /// 显示错误消息
    fn show_error(&self, message: &str);

    /// 显示状态消息
    fn show_status(&self, message: &str);

    /// 切换全屏模式
    fn toggle_fullscreen(&self);

    /// 检查是否处于全屏模式
    fn is_fullscreen(&self) -> bool;

    /// 退出应用程序
    fn exit(&self);

    /// 获取当前窗口大小
    fn window_size(&self) -> (f32, f32);
}

/// 剪贴板端口
///
/// 负责剪贴板操作
pub trait ClipboardPort: Send + Sync {
    /// 复制图像数据到剪贴板
    fn copy_image(&self, width: usize, height: usize, data: &[u8]) -> Result<()>;

    /// 复制文件路径到剪贴板
    fn copy_path(&self, path: &Path) -> Result<()>;

    /// 检查剪贴板是否可用
    fn is_available(&self) -> bool;

    /// 在文件管理器中显示文件
    fn show_in_folder(&self, path: &Path) -> Result<()>;
}

/// 文件对话框端口
///
/// 负责文件选择对话框
pub trait FileDialogPort: Send + Sync {
    /// 打开文件选择对话框
    fn open_files(&self) -> Option<Vec<PathBuf>>;

    /// 打开目录选择对话框
    fn open_directory(&self) -> Option<PathBuf>;
}

/// 图像加载完成回调
pub type ImageLoadedCallback = Box<dyn FnOnce(Result<Image>) + Send>;

/// 缩略图加载完成回调
pub type ThumbnailLoadedCallback = Box<dyn FnOnce(usize, Result<Vec<u8>>) + Send>;

/// 异步图像源扩展
///
/// 支持异步加载的图像源
pub trait AsyncImageSource: ImageSource {
    /// 异步加载图像
    fn load_image_async(&self, path: &Path, callback: ImageLoadedCallback);

    /// 异步生成缩略图
    fn generate_thumbnail_async(
        &self,
        path: &Path,
        max_size: u32,
        index: usize,
        callback: ThumbnailLoadedCallback,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.last_opened_directory.is_none());
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig::default();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    // 从旧代码迁移的额外测试

    #[test]
    fn test_app_config_default_values() {
        let config = AppConfig::default();
        assert_eq!(config.window.width, 1200.0);
        assert_eq!(config.window.height, 800.0);
        assert!(!config.window.maximized);
        assert_eq!(config.gallery.thumbnail_size, 120);
        assert!(config.viewer.fit_to_window);
        assert!((config.viewer.min_scale - 0.1).abs() < 0.001);
        assert!((config.viewer.max_scale - 20.0).abs() < 0.001);
        assert!((config.viewer.zoom_step - 1.25).abs() < 0.001);
        assert!(config.viewer.smooth_scroll);
    }

    #[test]
    fn test_app_config_equality() {
        let config1 = AppConfig::default();
        let config2 = AppConfig::default();
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_app_config_debug() {
        let config = AppConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("window"));
        assert!(debug_str.contains("gallery"));
        assert!(debug_str.contains("viewer"));
    }

    #[test]
    fn test_app_config_serialize_deserialize() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_app_config_with_last_directory() {
        let mut config = AppConfig::default();
        config.last_opened_directory = Some(PathBuf::from("/home/user/images"));
        assert_eq!(
            config.last_opened_directory,
            Some(PathBuf::from("/home/user/images"))
        );
    }

    #[test]
    fn test_app_config_window_state() {
        let config = AppConfig::default();
        let size = config.window.size();
        assert_eq!(size, [1200.0, 800.0]);
        assert!(config.window.position().is_none());
    }

    #[test]
    fn test_app_config_gallery_layout() {
        let config = AppConfig::default();
        assert_eq!(config.gallery.thumbnail_size, 120);
        assert_eq!(config.gallery.items_per_row, 0);
        assert!((config.gallery.grid_spacing - 12.0).abs() < 0.001);
        assert!(config.gallery.show_filenames);
    }

    #[test]
    fn test_app_config_viewer_settings() {
        let config = AppConfig::default();
        assert!(config.viewer.fit_to_window);
        assert!(!config.viewer.show_info_panel); // Bug 2 修复: F 键控制信息面板
        assert!((config.viewer.min_scale - 0.1).abs() < 0.001);
        assert!((config.viewer.max_scale - 20.0).abs() < 0.001);
        assert!((config.viewer.zoom_step - 1.25).abs() < 0.001);
        assert!(config.viewer.smooth_scroll);
    }

    #[test]
    fn test_app_config_viewer_background_color() {
        let config = AppConfig::default();
        assert_eq!(config.viewer.background_color.r, 30);
        assert_eq!(config.viewer.background_color.g, 30);
        assert_eq!(config.viewer.background_color.b, 30);
        assert_eq!(config.viewer.background_color.a, 255);
    }
}
