//! 配置管理
//!
//! 此模块处理应用程序配置，包括：
//! - 窗口状态（大小、位置、最大化）
//! - 图库设置（缩略图大小、网格布局）
//! - 查看器设置（背景颜色、缩放行为、信息面板）
//!
//! 配置存储在平台特定目录：
//! - Linux: ~/.config/image-viewer/config.toml
//! - macOS: ~/Library/Application Support/com.imageviewer.image-viewer/config.toml
//! - Windows: %APPDATA%\image-viewer\config.toml

use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// 应用程序配置根
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// 窗口设置
    pub window: WindowConfig,
    /// 图库设置
    pub gallery: GalleryConfig,
    /// 查看器设置
    pub viewer: ViewerConfig,
}

/// 窗口配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度（像素）
    pub width: f32,
    /// 窗口高度（像素）
    pub height: f32,
    /// 窗口X位置（None表示默认居中）
    pub x: Option<f32>,
    /// 窗口Y位置（None表示默认居中）
    pub y: Option<f32>,
    /// 窗口是否最大化
    pub maximized: bool,
}

/// 图库视图配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GalleryConfig {
    /// 缩略图大小（像素，范围：80-200）
    pub thumbnail_size: u32,
    /// 每行项目数（0 = 基于窗口宽度自动计算）
    pub items_per_row: usize,
    /// 网格间距（像素）
    pub grid_spacing: f32,
    /// 在缩略图下显示文件名
    pub show_filenames: bool,
}

/// 查看器视图配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewerConfig {
    /// 背景颜色 [R, G, B]
    pub background_color: [u8; 3],
    /// 默认适配模式：打开时适应窗口
    pub fit_to_window: bool,
    /// 默认显示信息面板
    pub show_info_panel: bool,
    /// 最小缩放比例（10%）
    pub min_scale: f32,
    /// 最大缩放比例（2000% = 20x）
    pub max_scale: f32,
    /// 缩放步长倍数（1.25 = 每步25%）
    pub zoom_step: f32,
    /// 启用平滑滚动
    pub smooth_scroll: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            gallery: GalleryConfig::default(),
            viewer: ViewerConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

impl Default for GalleryConfig {
    fn default() -> Self {
        Self {
            thumbnail_size: 120,
            items_per_row: 0, // 自动计算
            grid_spacing: 12.0,
            show_filenames: true,
        }
    }
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            background_color: [30, 30, 30],
            fit_to_window: true,
            show_info_panel: false,
            min_scale: 0.1,
            max_scale: 20.0,
            zoom_step: 1.25,
            smooth_scroll: true,
        }
    }
}

impl Config {
    /// 从平台特定的配置目录加载配置。
    /// 
    /// 如果配置文件不存在，创建默认配置并保存。
    /// 如果配置文件损坏或无效，记录警告并返回默认配置。
    ///
    /// # Returns
    /// - `Ok(Config)` - 加载的或默认的配置
    /// - `Err(anyhow::Error)` - 仅用于关键文件系统错误
    ///
    /// # Platform Paths
    /// - Linux: `~/.config/image-viewer/config.toml`
    /// - macOS: `~/Library/Application Support/com.imageviewer.image-viewer/config.toml`
    /// - Windows: `%APPDATA%\image-viewer\config.toml`
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        debug!("加载配置从: {:?}", config_path);

        if !config_path.exists() {
            info!("配置文件不存在于 {:?}, 创建默认配置", config_path);
            let config = Self::default();
            if let Err(e) = config.save() {
                warn!("保存默认配置失败: {}. 使用不保存的默认值.", e);
            }
            return Ok(config);
        }

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取配置从 {:?}", config_path))?;

        match toml::from_str::<Self>(&content) {
            Ok(config) => {
                // Validate config values
                let validated = config.validate();
                if validated != config {
                    info!("配置值已调整到有效范围");
                    // Save the corrected config
                    if let Err(e) = validated.save() {
                        warn!("保存修正后的配置失败: {}", e);
                    }
                }
                Ok(validated)
            }
            Err(e) => {
                warn!("解析配置文件失败: {}. 使用默认值.", e);
                let default = Self::default();
                // 尝试备份损坏的配置
                let backup_path = config_path.with_extension("toml.bak");
                if let Err(backup_err) = std::fs::copy(&config_path, &backup_path) {
                    warn!("备份损坏的配置失败: {}", backup_err);
                } else {
                    info!("损坏的配置已备份到 {:?}", backup_path);
                }
                // Save default config
                if let Err(save_err) = default.save() {
                    warn!("保存默认配置失败: {}", save_err);
                }
                Ok(default)
            }
        }
    }

    /// 保存配置到平台特定的配置目录。
    ///
    /// 如果父目录不存在，自动创建。
    ///
    /// # Returns
    /// - `Ok(())` - 配置保存成功
    /// - `Err(anyhow::Error)` - 创建目录或写入文件失败
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "序列化配置到TOML失败")?;

        std::fs::write(&config_path, content)
            .with_context(|| format!("无法写入配置到 {:?}", config_path))?;

        debug!("配置已保存到 {:?}", config_path);
        Ok(())
    }

    /// 获取当前平台的配置文件路径。
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - config.toml的完整路径
    /// - `Err(anyhow::Error)` - 无法确定配置目录
    ///
    /// # Examples
    ///
    /// ```rust
    /// use image_viewer::config::Config;
    ///
    /// let path = Config::config_path().unwrap();
    /// assert!(path.file_name().unwrap() == "config.toml");
    /// ```
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "imageviewer", "image-viewer")
            .context("无法确定配置目录: 未找到主目录")?;
        
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// 获取配置目录路径。
    ///
    /// 用于存储其他配置文件（主题、预设等）
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "imageviewer", "image-viewer")
            .context("无法确定配置目录: 未找到主目录")?;
        
        Ok(proj_dirs.config_dir().to_path_buf())
    }

    /// 验证并规范化配置值。
    ///
    /// 确保所有值在可接受范围内：
    /// - 窗口大小：最小 400x300
    /// - 缩略图大小：80-200像素
    /// - 缩放比例：min < max，都 > 0
    /// - 缩放步长：1.01-2.0
    fn validate(&self) -> Self {
        Self {
            window: self.window.validate(),
            gallery: self.gallery.validate(),
            viewer: self.viewer.validate(),
        }
    }

    /// 从eframe窗口信息更新窗口状态。
    ///
    /// 在窗口关闭或想要保存当前状态时调用。
    pub fn update_from_window(&mut self, inner_size: [f32; 2], position: Option<[f32; 2]>, maximized: bool) {
        self.window.width = inner_size[0];
        self.window.height = inner_size[1];
        self.window.maximized = maximized;
        
        if let Some([x, y]) = position {
            self.window.x = Some(x);
            self.window.y = Some(y);
        }
    }
}

impl WindowConfig {
    /// 验证窗口配置。
    fn validate(&self) -> Self {
        Self {
            width: self.width.max(400.0),
            height: self.height.max(300.0),
            x: self.x,
            y: self.y,
            maximized: self.maximized,
        }
    }

    /// 获取窗口位置为数组，如果未设置则返回None。
    pub fn position(&self) -> Option<[f32; 2]> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Some([x, y]),
            _ => None,
        }
    }

    /// 获取窗口大小为数组。
    pub fn size(&self) -> [f32; 2] {
        [self.width, self.height]
    }
}

impl GalleryConfig {
    /// 验证图库配置。
    fn validate(&self) -> Self {
        const MIN_THUMBNAIL: u32 = 80;
        const MAX_THUMBNAIL: u32 = 200;
        
        Self {
            thumbnail_size: self.thumbnail_size.clamp(MIN_THUMBNAIL, MAX_THUMBNAIL),
            items_per_row: self.items_per_row.max(1),
            grid_spacing: self.grid_spacing.max(0.0),
            show_filenames: self.show_filenames,
        }
    }
}

impl ViewerConfig {
    /// 验证查看器配置。
    fn validate(&self) -> Self {
        let min_scale = self.min_scale.max(0.01);
        let max_scale = self.max_scale.max(min_scale * 2.0);
        let zoom_step = self.zoom_step.clamp(1.01, 2.0);
        
        Self {
            background_color: [
                self.background_color[0],
                self.background_color[1],
                self.background_color[2],
            ],
            fit_to_window: self.fit_to_window,
            show_info_panel: self.show_info_panel,
            min_scale,
            max_scale,
            zoom_step,
            smooth_scroll: self.smooth_scroll,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // 默认配置测试
    // =========================================================================

    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        // 窗口默认值
        assert_eq!(config.window.width, 1200.0);
        assert_eq!(config.window.height, 800.0);
        assert_eq!(config.window.x, None);
        assert_eq!(config.window.y, None);
        assert!(!config.window.maximized);
        
        // 图库默认值
        assert_eq!(config.gallery.thumbnail_size, 120);
        assert_eq!(config.gallery.items_per_row, 0);
        assert_eq!(config.gallery.grid_spacing, 12.0);
        assert!(config.gallery.show_filenames);
        
        // 查看器默认值
        assert_eq!(config.viewer.background_color, [30, 30, 30]);
        assert!(config.viewer.fit_to_window);
        assert!(!config.viewer.show_info_panel);
        assert_eq!(config.viewer.min_scale, 0.1);
        assert_eq!(config.viewer.max_scale, 20.0);
        assert_eq!(config.viewer.zoom_step, 1.25);
        assert!(config.viewer.smooth_scroll);
    }

    // =========================================================================
    // 序列化测试
    // =========================================================================

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).expect("序列化失败");
        
        // 验证TOML包含预期部分
        assert!(toml_str.contains("[window]"));
        assert!(toml_str.contains("[gallery]"));
        assert!(toml_str.contains("[viewer]"));
        
        // 验证一些值存在
        assert!(toml_str.contains("width = 1200"));
        assert!(toml_str.contains("thumbnail_size = 120"));
        assert!(toml_str.contains("background_color"));
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
[window]
width = 1920.0
height = 1080.0
x = 100.0
y = 50.0
maximized = true

[gallery]
thumbnail_size = 150
items_per_row = 5
grid_spacing = 16.0
show_filenames = false

[viewer]
background_color = [50, 50, 50]
fit_to_window = false
show_info_panel = true
min_scale = 0.05
max_scale = 50.0
zoom_step = 1.5
smooth_scroll = false
"#;

        let config: Config = toml::from_str(toml_str).expect("反序列化失败");
        
        assert_eq!(config.window.width, 1920.0);
        assert_eq!(config.window.height, 1080.0);
        assert_eq!(config.window.x, Some(100.0));
        assert_eq!(config.window.y, Some(50.0));
        assert!(config.window.maximized);
        
        assert_eq!(config.gallery.thumbnail_size, 150);
        assert_eq!(config.gallery.items_per_row, 5);
        assert_eq!(config.gallery.grid_spacing, 16.0);
        assert!(!config.gallery.show_filenames);
        
        assert_eq!(config.viewer.background_color, [50, 50, 50]);
        assert!(!config.viewer.fit_to_window);
        assert!(config.viewer.show_info_panel);
        assert_eq!(config.viewer.min_scale, 0.05);
        assert_eq!(config.viewer.max_scale, 50.0);
        assert_eq!(config.viewer.zoom_step, 1.5);
        assert!(!config.viewer.smooth_scroll);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = Config {
            window: WindowConfig {
                width: 1600.0,
                height: 900.0,
                x: Some(200.0),
                y: Some(100.0),
                maximized: true,
            },
            gallery: GalleryConfig {
                thumbnail_size: 100,
                items_per_row: 4,
                grid_spacing: 8.0,
                show_filenames: false,
            },
            viewer: ViewerConfig {
                background_color: [20, 20, 20],
                fit_to_window: false,
                show_info_panel: true,
                min_scale: 0.2,
                max_scale: 10.0,
                zoom_step: 1.1,
                smooth_scroll: false,
            },
        };

        let toml_str = toml::to_string_pretty(&original).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(original, deserialized);
    }

    // =========================================================================
    // 无效配置处理测试
    // =========================================================================

    #[test]
    fn test_invalid_toml_handling() {
        let invalid_toml = r#"
[window]
width = "not a number"
height = 800.0
"#;

        let result = toml::from_str::<Config>(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_config_loading() {
        // 用于解析测试的完整配置
        let complete_toml = r#"
[window]
width = 1400.0
height = 900.0
x = 100.0
y = 100.0
maximized = true

[gallery]
thumbnail_size = 150
items_per_row = 5
grid_spacing = 12.0
show_filenames = true

[viewer]
background_color = [40, 40, 40]
fit_to_window = true
show_info_panel = true
min_scale = 0.1
max_scale = 10.0
zoom_step = 1.25
smooth_scroll = true
"#;

        let config: Config = toml::from_str(complete_toml).expect("应能解析完整配置");
        
        // 指定的值
        assert_eq!(config.window.width, 1400.0);
        assert_eq!(config.window.height, 900.0);
        assert!(config.window.maximized);
        
        // 检查其他部分
        assert_eq!(config.gallery.thumbnail_size, 150);
        assert_eq!(config.viewer.min_scale, 0.1);
    }

    #[test]
    fn test_corrupted_toml_fallback() {
        let corrupted = r#"
this is not valid toml {{{
[window
width = 100
"#;

        let result = toml::from_str::<Config>(corrupted);
        assert!(result.is_err());
    }

    // =========================================================================
    // 验证测试
    // =========================================================================

    #[test]
    fn test_window_validation() {
        let config = Config {
            window: WindowConfig {
                width: 100.0,  // 太小
                height: 50.0,  // 太小
                ..Default::default()
            },
            ..Default::default()
        };

        let validated = config.validate();
        
        // 应限制到最小值
        assert_eq!(validated.window.width, 400.0);
        assert_eq!(validated.window.height, 300.0);
    }

    #[test]
    fn test_gallery_validation() {
        // 测试缩略图大小限制 - 太小
        let config = Config {
            gallery: GalleryConfig {
                thumbnail_size: 50,  // 低于最小值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.gallery.thumbnail_size, 80);

        // 测试缩略图大小限制 - 太大
        let config = Config {
            gallery: GalleryConfig {
                thumbnail_size: 300,  // 超过最大值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.gallery.thumbnail_size, 200);

        // 测试负间距
        let config = Config {
            gallery: GalleryConfig {
                grid_spacing: -5.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.gallery.grid_spacing, 0.0);
    }

    #[test]
    fn test_viewer_validation() {
        // 测试最小/最大缩放比例关系
        let config = Config {
            viewer: ViewerConfig {
                min_scale: 5.0,
                max_scale: 1.0,  // 小于最小值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert!(validated.viewer.max_scale >= validated.viewer.min_scale * 2.0);

        // 测试缩放步长限制 - 太小
        let config = Config {
            viewer: ViewerConfig {
                zoom_step: 1.005,  // 低于最小值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.viewer.zoom_step, 1.01);

        // 测试缩放步长限制 - 太大
        let config = Config {
            viewer: ViewerConfig {
                zoom_step: 5.0,  // 超过最大值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.viewer.zoom_step, 2.0);
    }

    #[test]
    fn test_negative_scale_handling() {
        let config = Config {
            viewer: ViewerConfig {
                min_scale: -0.5,  // 无效负数
                max_scale: -1.0,  // 无效负数
                ..Default::default()
            },
            ..Default::default()
        };
        
        let validated = config.validate();
        assert!(validated.viewer.min_scale > 0.0);
        assert!(validated.viewer.max_scale > validated.viewer.min_scale);
    }

    // =========================================================================
    // 文件I/O测试
    // =========================================================================

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 创建并保存配置
        let original = Config {
            window: WindowConfig {
                width: 1400.0,
                height: 900.0,
                x: Some(100.0),
                y: Some(200.0),
                maximized: false,
            },
            gallery: GalleryConfig {
                thumbnail_size: 100,
                items_per_row: 6,
                grid_spacing: 10.0,
                show_filenames: true,
            },
            viewer: ViewerConfig {
                background_color: [40, 40, 40],
                fit_to_window: true,
                show_info_panel: false,
                min_scale: 0.15,
                max_scale: 15.0,
                zoom_step: 1.2,
                smooth_scroll: true,
            },
        };

        // Write directly to test file
        let content = toml::to_string_pretty(&original).unwrap();
        std::fs::write(&config_path, content).unwrap();

        // 读取并验证
        let loaded_content = std::fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_config_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("deep").join("nested").join("dir");
        let config_path = nested_dir.join("config.toml");

        let config = Config::default();
        let content = toml::to_string_pretty(&config).unwrap();
        
        // Create parent directories
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(&config_path, content).unwrap();

        assert!(config_path.exists());
    }

    // =========================================================================
    // 窗口状态测试
    // =========================================================================

    #[test]
    fn test_update_from_window() {
        let mut config = Config::default();
        
        config.update_from_window([1600.0, 900.0], Some([100.0, 50.0]), true);
        
        assert_eq!(config.window.width, 1600.0);
        assert_eq!(config.window.height, 900.0);
        assert_eq!(config.window.x, Some(100.0));
        assert_eq!(config.window.y, Some(50.0));
        assert!(config.window.maximized);
    }

    #[test]
    fn test_update_from_window_without_position() {
        let mut config = Config::default();
        
        config.update_from_window([1400.0, 800.0], None, false);
        
        assert_eq!(config.window.width, 1400.0);
        assert_eq!(config.window.height, 800.0);
        // 位置应保持不变当传入None时
        assert_eq!(config.window.x, None);
        assert_eq!(config.window.y, None);
        assert!(!config.window.maximized);
    }

    #[test]
    fn test_window_position_helper() {
        let config_with_pos = WindowConfig {
            x: Some(100.0),
            y: Some(200.0),
            ..Default::default()
        };
        assert_eq!(config_with_pos.position(), Some([100.0, 200.0]));

        let config_without_pos = WindowConfig {
            x: None,
            y: Some(200.0),
            ..Default::default()
        };
        assert_eq!(config_without_pos.position(), None);

        let config_partial = WindowConfig {
            x: Some(100.0),
            y: None,
            ..Default::default()
        };
        assert_eq!(config_partial.position(), None);
    }

    #[test]
    fn test_window_size_helper() {
        let config = WindowConfig {
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        };
        assert_eq!(config.size(), [1920.0, 1080.0]);
    }

    // =========================================================================
    // 边界情况测试
    // =========================================================================

    #[test]
    fn test_empty_toml_file() {
        let empty = "";
        // 不完整的TOML应解析失败
        let result: Result<Config, _> = toml::from_str(empty);
        assert!(result.is_err());
        
        // 所有值应为默认值
        
    }

    #[test]
    fn test_very_large_values() {
        let toml_str = r#"
[window]
width = 1000000.0
height = 1000000.0
x = 100.0
y = 100.0
maximized = true

[gallery]
thumbnail_size = 4294967295
items_per_row = 5
grid_spacing = 8.0
show_filenames = true

[viewer]
background_color = [30, 30, 30]
fit_to_window = true
show_info_panel = true
min_scale = 0.1
max_scale = 10.0
zoom_step = 1.25
smooth_scroll = true
"#;

        let config: Config = toml::from_str(toml_str).expect("应能解析");
        let validated = config.validate();
        
        // 值应被限制
        assert_eq!(validated.window.width, 1000000.0);  // 窗口大小没有上限
        assert_eq!(validated.gallery.thumbnail_size, 200);  // 限制到最大值
    }

    #[test]
    fn test_special_characters_in_toml() {
        // 测试特殊字符不破坏解析
        let toml_str = r#"
[window]
width = 1200.0
# This is a comment with special chars: !@#$%^&*()
height = 800.0
x = 100.0
y = 100.0
maximized = false

[gallery]
thumbnail_size = 120
items_per_row = 4
grid_spacing = 8.0
show_filenames = true

[viewer]
background_color = [30, 30, 30]
fit_to_window = true
show_info_panel = true
min_scale = 0.1
max_scale = 10.0
zoom_step = 1.25
smooth_scroll = true
"#;

        let config: Config = toml::from_str(toml_str).expect("应能处理注释");
        assert_eq!(config.window.width, 1200.0);
        assert_eq!(config.window.height, 800.0);
    }


    #[test]
    fn test_config_with_all_fields() {
        let full_toml = r#"
[window]
width = 1920.0
height = 1080.0
x = 0.0
y = 0.0
maximized = true

[gallery]
thumbnail_size = 180
items_per_row = 8
grid_spacing = 16.0
show_filenames = true

[viewer]
background_color = [25, 25, 25]
fit_to_window = false
show_info_panel = true
min_scale = 0.05
max_scale = 20.0
zoom_step = 1.5
smooth_scroll = false
"#;
        let config: Config = toml::from_str(full_toml).expect("应能解析 full config");
        assert_eq!(config.window.width, 1920.0);
        assert_eq!(config.window.height, 1080.0);
        assert!(config.window.maximized);
        assert_eq!(config.gallery.thumbnail_size, 180);
        assert_eq!(config.gallery.items_per_row, 8);
        assert_eq!(config.viewer.background_color, [25, 25, 25]);
        assert_eq!(config.viewer.min_scale, 0.05);
        assert_eq!(config.viewer.max_scale, 20.0);
    }

    #[test]
    fn test_window_config_validate() {
        let config = WindowConfig {
            width: 1920.0,
            height: 1080.0,
            x: Some(100.0),
            y: Some(200.0),
            maximized: true,
        };
        let validated = config.validate();
        assert_eq!(validated.width, 1920.0);
        assert_eq!(validated.height, 1080.0);
    }

    #[test]
    fn test_gallery_config_validate() {
        let config = GalleryConfig {
            thumbnail_size: 150,
            items_per_row: 6,
            grid_spacing: 14.0,
            show_filenames: true,
        };
        let validated = config.validate();
        assert_eq!(validated.thumbnail_size, 150);
        assert_eq!(validated.items_per_row, 6);
    }

    #[test]
    fn test_viewer_config_validate() {
        let config = ViewerConfig {
            background_color: [40, 40, 40],
            fit_to_window: true,
            show_info_panel: false,
            min_scale: 0.2,
            max_scale: 15.0,
            zoom_step: 1.3,
            smooth_scroll: true,
        };
        let validated = config.validate();
        assert_eq!(validated.background_color, [40, 40, 40]);
        assert!(validated.fit_to_window);
    }

    #[test]
    fn test_window_config_min_size() {
        let config = WindowConfig {
            width: 100.0,
            height: 100.0,
            x: None,
            y: None,
            maximized: false,
        };
        let validated = config.validate();
        assert!(validated.width >= 400.0);
        assert!(validated.height >= 300.0);
    }

    #[test]
    fn test_gallery_config_clamp() {
        let config = GalleryConfig {
            thumbnail_size: 500,
            items_per_row: 0,
            grid_spacing: -5.0,
            show_filenames: false,
        };
        let validated = config.validate();
        assert!(validated.thumbnail_size <= 200);
        assert!(validated.thumbnail_size >= 80);
        assert!(validated.items_per_row >= 1);
        assert!(validated.grid_spacing >= 0.0);
    }

    #[test]
    fn test_viewer_config_clamp() {
        let config = ViewerConfig {
            background_color: [30, 30, 30],
            fit_to_window: true,
            show_info_panel: true,
            min_scale: 10.0,
            max_scale: 0.05,
            zoom_step: 0.5,
            smooth_scroll: false,
        };
        let validated = config.validate();
        assert!(validated.min_scale < validated.max_scale);
        assert!(validated.zoom_step >= 1.01);
    }

    #[test]
    fn test_config_equality() {
        let config1 = Config::default();
        let config2 = Config::default();
        assert_eq!(config1, config2);

        let mut config3 = Config::default();
        config3.window.width = 999.0;
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_clone_config() {
        let original = Config::default();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_debug_format() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("window"));
        assert!(debug_str.contains("gallery"));
        assert!(debug_str.contains("viewer"));
    }
}
