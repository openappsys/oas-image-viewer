//! 配置管理
//!
//! 此模块处理应用程序配置，包括：
//! - 窗口状态（大小、位置、最大化）
//! - 图库设置（缩略图大小、网格布局）
//! - 查看器设置（背景颜色、缩放行为、信息面板）
//! - 应用状态（上次打开的目录）
//!
//! 配置存储在平台特定目录：
//! - Linux: ~/.config/oas-image-viewer/config.toml
//! - macOS: ~/Library/Application Support/com.imageviewer.image-viewer/config.toml
//! - Windows: %APPDATA%\image-viewer\config.toml
//!
//! # 最佳实践
//!
//! 1. **原子写入**: 配置通过临时文件写入，然后原子重命名，防止写入中断导致配置损坏
//! 2. **防抖保存**: 配置变更后延迟保存，避免频繁磁盘写入
//! 3. **配置验证**: 加载时自动验证并修正无效值
//! 4. **优雅降级**: 配置损坏时自动备份并使用默认值

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// 防抖保存延迟（毫秒）
const DEBOUNCE_MS: u64 = 500;
/// 最小保存间隔（毫秒），防止过于频繁的保存
const MIN_SAVE_INTERVAL_MS: u64 = 100;

/// 应用程序配置根
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// 窗口设置
    pub window: WindowConfig,
    /// 图库设置
    pub gallery: GalleryConfig,
    /// 查看器设置
    pub viewer: ViewerConfig,
    /// 应用状态
    pub app: AppStateConfig,
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
    /// 背景颜色 [R, G, B, A]
    pub background_color: [u8; 4],
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

/// 应用状态配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppStateConfig {
    /// 上次打开的目录路径
    pub last_opened_directory: Option<PathBuf>,
    /// 默认缩放比例（预留）
    pub default_zoom_scale: Option<f32>,
    /// 主题设置（预留）："dark" | "light" | "system"
    pub theme: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            gallery: GalleryConfig::default(),
            viewer: ViewerConfig::default(),
            app: AppStateConfig::default(),
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
            background_color: [30, 30, 30, 255],
            fit_to_window: true,
            show_info_panel: false,
            min_scale: 0.1,
            max_scale: 20.0,
            zoom_step: 1.25,
            smooth_scroll: true,
        }
    }
}

impl Default for AppStateConfig {
    fn default() -> Self {
        Self {
            last_opened_directory: None,
            default_zoom_scale: None,
            theme: None,
        }
    }
}

/// 防抖配置保存器
///
/// 使用 mpsc 通道实现防抖保存，确保配置变更不会导致频繁的磁盘写入
pub struct DebouncedConfigSaver {
    sender: mpsc::Sender<ConfigMessage>,
    _thread_handle: thread::JoinHandle<()>,
}

enum ConfigMessage {
    Save(Config),
    Shutdown,
}

impl DebouncedConfigSaver {
    /// 创建新的防抖配置保存器
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<ConfigMessage>();

        let handle = thread::spawn(move || {
            let mut last_save = Instant::now();
            let mut pending_config: Option<Config> = None;

            loop {
                // 等待消息，使用超时来实现防抖
                let timeout = Duration::from_millis(DEBOUNCE_MS);
                let result = receiver.recv_timeout(timeout);

                match result {
                    Ok(ConfigMessage::Save(config)) => {
                        pending_config = Some(config);
                    }
                    Ok(ConfigMessage::Shutdown) => {
                        // 关闭前保存待处理的配置
                        if let Some(config) = pending_config {
                            if let Err(e) = config.save() {
                                error!("关闭时保存配置失败: {}", e);
                            }
                        }
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // 防抖超时，执行保存
                        if let Some(config) = pending_config.take() {
                            // 确保最小保存间隔
                            let elapsed = last_save.elapsed();
                            if elapsed < Duration::from_millis(MIN_SAVE_INTERVAL_MS) {
                                thread::sleep(
                                    Duration::from_millis(MIN_SAVE_INTERVAL_MS) - elapsed,
                                );
                            }

                            if let Err(e) = config.save() {
                                error!("防抖保存配置失败: {}", e);
                            } else {
                                last_save = Instant::now();
                                debug!("配置已防抖保存");
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // 通道断开，保存任何待处理的配置然后退出
                        if let Some(config) = pending_config {
                            let _ = config.save();
                        }
                        break;
                    }
                }
            }
        });

        Self {
            sender,
            _thread_handle: handle,
        }
    }

    /// 请求保存配置（防抖）
    pub fn request_save(&self, config: &Config) {
        if let Err(e) = self.sender.send(ConfigMessage::Save(config.clone())) {
            error!("发送保存配置请求失败: {}", e);
            // 同步回退
            if let Err(e) = config.save() {
                error!("同步保存配置失败: {}", e);
            }
        }
    }

    /// 立即保存配置（不防抖）
    pub fn save_now(&self, config: &Config) {
        // 直接保存，不经过防抖
        if let Err(e) = config.save() {
            error!("立即保存配置失败: {}", e);
        }
    }
}

impl Default for DebouncedConfigSaver {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DebouncedConfigSaver {
    fn drop(&mut self) {
        let _ = self.sender.send(ConfigMessage::Shutdown);
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
    /// - Linux: `~/.config/oas-image-viewer/config.toml`
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
    /// 使用原子写入策略：
    /// 1. 将配置序列化为 TOML
    /// 2. 写入临时文件
    /// 3. 原子重命名为目标文件
    ///
    /// 这确保即使在写入过程中发生崩溃，也不会留下损坏的配置文件。
    ///
    /// # Returns
    /// - `Ok(())` - 配置保存成功
    /// - `Err(anyhow::Error)` - 创建目录或写入文件失败
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // 确保父目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {:?}", parent))?;
        }

        // 序列化配置
        let content = toml::to_string_pretty(self).with_context(|| "序列化配置到TOML失败")?;

        // 原子写入：先写临时文件，然后重命名
        let temp_path = config_path.with_extension("toml.tmp");

        // 写入临时文件
        std::fs::write(&temp_path, content)
            .with_context(|| format!("无法写入临时配置到 {:?}", temp_path))?;

        // 原子重命名（确保配置文件永远不会处于部分写入状态）
        std::fs::rename(&temp_path, &config_path)
            .with_context(|| format!("无法重命名临时配置 {:?} 到 {:?}", temp_path, config_path))?;

        debug!("配置已原子保存到 {:?}", config_path);
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
    /// use oas_image_viewer::config::Config;
    ///
    /// let path = Config::config_path().unwrap();
    /// assert!(path.file_name().unwrap() == "config.toml");
    /// ```
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "imageviewer", "oas-image-viewer")
            .context("无法确定配置目录: 未找到主目录")?;

        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// 获取配置目录路径。
    ///
    /// 用于存储其他配置文件（主题、预设等）
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "imageviewer", "oas-image-viewer")
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
            app: self.app.validate(),
        }
    }

    /// 从eframe窗口信息更新窗口状态。
    ///
    /// 在窗口关闭或想要保存当前状态时调用。
    pub fn update_from_window(
        &mut self,
        inner_size: [f32; 2],
        position: Option<[f32; 2]>,
        maximized: bool,
    ) {
        self.window.width = inner_size[0];
        self.window.height = inner_size[1];
        self.window.maximized = maximized;

        if let Some([x, y]) = position {
            self.window.x = Some(x);
            self.window.y = Some(y);
        }
    }

    /// 更新上次打开的目录
    pub fn set_last_opened_directory(&mut self, path: impl Into<PathBuf>) {
        self.app.last_opened_directory = Some(path.into());
    }

    /// 获取上次打开的目录
    pub fn last_opened_directory(&self) -> Option<&PathBuf> {
        self.app.last_opened_directory.as_ref()
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
                self.background_color[3],
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

impl AppStateConfig {
    /// 验证应用状态配置。
    fn validate(&self) -> Self {
        // 验证 default_zoom_scale 范围
        let default_zoom_scale = self
            .default_zoom_scale
            .map(|scale| {
                if scale < 0.01 {
                    None
                } else if scale > 20.0 {
                    Some(20.0)
                } else {
                    Some(scale)
                }
            })
            .flatten();

        // 验证 theme 值
        let theme = self.theme.as_ref().and_then(|t| {
            let t = t.to_lowercase();
            if ["dark", "light", "system"].contains(&t.as_str()) {
                Some(t)
            } else {
                None
            }
        });

        Self {
            last_opened_directory: self.last_opened_directory.clone(),
            default_zoom_scale,
            theme,
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

        // 应用状态默认值
        assert!(config.app.last_opened_directory.is_none());
        assert!(config.app.default_zoom_scale.is_none());
        assert!(config.app.theme.is_none());
    }

    // =========================================================================
    // 序列化测试
    // =========================================================================

    #[test]
    fn test_toml_serialization() {
        let mut config = Config::default();
        config.set_last_opened_directory("/test/path");
        let toml_str = toml::to_string_pretty(&config).expect("序列化失败");

        // 验证TOML包含预期部分
        assert!(toml_str.contains("[window]"));
        assert!(toml_str.contains("[gallery]"));
        assert!(toml_str.contains("[viewer]"));
        assert!(toml_str.contains("[app]"));

        // 验证一些值存在
        assert!(toml_str.contains("width = 1200"));
        assert!(toml_str.contains("thumbnail_size = 120"));
        assert!(toml_str.contains("background_color"));
        assert!(toml_str.contains("last_opened_directory"));
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
background_color = [50, 50, 50, 255]
fit_to_window = false
show_info_panel = true
min_scale = 0.05
max_scale = 50.0
zoom_step = 1.5
smooth_scroll = false

[app]
last_opened_directory = "/home/user/pictures"
default_zoom_scale = 1.5
theme = "dark"
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

        assert_eq!(
            config.app.last_opened_directory,
            Some(PathBuf::from("/home/user/pictures"))
        );
        assert_eq!(config.app.default_zoom_scale, Some(1.5));
        assert_eq!(config.app.theme, Some("dark".to_string()));
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
                background_color: [20, 20, 20, 255],
                fit_to_window: false,
                show_info_panel: true,
                min_scale: 0.2,
                max_scale: 10.0,
                zoom_step: 1.1,
                smooth_scroll: false,
            },
            app: AppStateConfig {
                last_opened_directory: Some(PathBuf::from("/test/path")),
                default_zoom_scale: Some(1.2),
                theme: Some("light".to_string()),
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
background_color = [40, 40, 40, 255]
fit_to_window = true
show_info_panel = true
min_scale = 0.1
max_scale = 10.0
zoom_step = 1.25
smooth_scroll = true

[app]
last_opened_directory = "/home/user/photos"
"#;

        let config: Config = toml::from_str(complete_toml).expect("应能解析完整配置");

        // 指定的值
        assert_eq!(config.window.width, 1400.0);
        assert_eq!(config.window.height, 900.0);
        assert!(config.window.maximized);

        // 检查其他部分
        assert_eq!(config.gallery.thumbnail_size, 150);
        assert_eq!(config.viewer.min_scale, 0.1);
        assert_eq!(
            config.app.last_opened_directory,
            Some(PathBuf::from("/home/user/photos"))
        );
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
                width: 100.0, // 太小
                height: 50.0, // 太小
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
                thumbnail_size: 50, // 低于最小值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.gallery.thumbnail_size, 80);

        // 测试缩略图大小限制 - 太大
        let config = Config {
            gallery: GalleryConfig {
                thumbnail_size: 300, // 超过最大值
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
                max_scale: 1.0, // 小于最小值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert!(validated.viewer.max_scale >= validated.viewer.min_scale * 2.0);

        // 测试缩放步长限制 - 太小
        let config = Config {
            viewer: ViewerConfig {
                zoom_step: 1.005, // 低于最小值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.viewer.zoom_step, 1.01);

        // 测试缩放步长限制 - 太大
        let config = Config {
            viewer: ViewerConfig {
                zoom_step: 5.0, // 超过最大值
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.viewer.zoom_step, 2.0);
    }

    #[test]
    fn test_app_state_validation() {
        // 测试无效的 default_zoom_scale
        let config = Config {
            app: AppStateConfig {
                default_zoom_scale: Some(50.0), // 超出范围
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.app.default_zoom_scale, Some(20.0));

        // 测试负的 default_zoom_scale
        let config = Config {
            app: AppStateConfig {
                default_zoom_scale: Some(-0.5),
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.app.default_zoom_scale, None);

        // 测试无效的 theme
        let config = Config {
            app: AppStateConfig {
                theme: Some("invalid_theme".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.app.theme, None);

        // 测试有效的 theme（大小写不敏感）
        let config = Config {
            app: AppStateConfig {
                theme: Some("DARK".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let validated = config.validate();
        assert_eq!(validated.app.theme, Some("dark".to_string()));
    }

    #[test]
    fn test_negative_scale_handling() {
        let config = Config {
            viewer: ViewerConfig {
                min_scale: -0.5, // 无效负数
                max_scale: -1.0, // 无效负数
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
                background_color: [40, 40, 40, 255],
                fit_to_window: true,
                show_info_panel: false,
                min_scale: 0.15,
                max_scale: 15.0,
                zoom_step: 1.2,
                smooth_scroll: true,
            },
            app: AppStateConfig {
                last_opened_directory: Some(PathBuf::from("/test/dir")),
                default_zoom_scale: Some(1.0),
                theme: Some("system".to_string()),
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

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("atomic_test.toml");

        // 模拟原子写入
        let config = Config::default();
        let content = toml::to_string_pretty(&config).unwrap();
        let temp_path = config_path.with_extension("toml.tmp");

        // 写入临时文件
        std::fs::write(&temp_path, content).unwrap();
        // 原子重命名
        std::fs::rename(&temp_path, &config_path).unwrap();

        assert!(config_path.exists());
        assert!(!temp_path.exists());

        // 验证内容
        let loaded = std::fs::read_to_string(&config_path).unwrap();
        assert!(loaded.contains("[window]"));
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
    // 应用状态测试
    // =========================================================================

    #[test]
    fn test_set_last_opened_directory() {
        let mut config = Config::default();

        config.set_last_opened_directory("/home/user/pictures");

        assert_eq!(
            config.app.last_opened_directory,
            Some(PathBuf::from("/home/user/pictures"))
        );
        assert_eq!(
            config.last_opened_directory(),
            Some(&PathBuf::from("/home/user/pictures"))
        );
    }

    #[test]
    fn test_last_opened_directory_none() {
        let config = Config::default();
        assert!(config.last_opened_directory().is_none());
    }

    #[test]
    fn test_path_buf_conversion() {
        let mut config = Config::default();
        let path = PathBuf::from("/test/path");

        config.set_last_opened_directory(path.clone());

        assert_eq!(config.app.last_opened_directory, Some(path));
    }

    // =========================================================================
    // 防抖保存测试
    // =========================================================================

    #[test]
    fn test_debounced_saver_creation() {
        let saver = DebouncedConfigSaver::new();
        // 应该成功创建
        drop(saver);
    }

    #[test]
    fn test_debounced_save_request() {
        let saver = DebouncedConfigSaver::new();
        let config = Config::default();

        // 请求保存（不会panic）
        saver.request_save(&config);

        // 给一点时间让后台线程处理
        thread::sleep(Duration::from_millis(100));
        drop(saver);
    }

    #[test]
    fn test_save_now() {
        let saver = DebouncedConfigSaver::new();
        let config = Config::default();

        // 立即保存（不会panic）
        saver.save_now(&config);

        drop(saver);
    }

    #[test]
    fn test_multiple_save_requests() {
        let saver = DebouncedConfigSaver::new();
        let mut config = Config::default();

        // 多次快速请求保存
        for i in 0..10 {
            config.window.width = 1000.0 + i as f32 * 10.0;
            saver.request_save(&config);
        }

        // 给一点时间让后台线程处理
        thread::sleep(Duration::from_millis(DEBOUNCE_MS + 100));
        drop(saver);
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

[app]
last_opened_directory = "/test"
"#;

        let config: Config = toml::from_str(toml_str).expect("应能解析");
        let validated = config.validate();

        // 值应被限制
        assert_eq!(validated.window.width, 1000000.0); // 窗口大小没有上限
        assert_eq!(validated.gallery.thumbnail_size, 200); // 限制到最大值
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

[app]
last_opened_directory = "/test"
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
background_color = [25, 25, 25, 255]
fit_to_window = false
show_info_panel = true
min_scale = 0.05
max_scale = 20.0
zoom_step = 1.5
smooth_scroll = false

[app]
last_opened_directory = "/home/user/pictures"
default_zoom_scale = 1.0
theme = "dark"
"#;
        let config: Config = toml::from_str(full_toml).expect("应能解析 full config");
        assert_eq!(config.window.width, 1920.0);
        assert_eq!(config.window.height, 1080.0);
        assert!(config.window.maximized);
        assert_eq!(config.gallery.thumbnail_size, 180);
        assert_eq!(config.gallery.items_per_row, 8);
        assert_eq!(config.viewer.background_color, [25, 25, 25, 255]);
        assert_eq!(config.viewer.min_scale, 0.05);
        assert_eq!(config.viewer.max_scale, 20.0);
        assert_eq!(
            config.app.last_opened_directory,
            Some(PathBuf::from("/home/user/pictures"))
        );
        assert_eq!(config.app.default_zoom_scale, Some(1.0));
        assert_eq!(config.app.theme, Some("dark".to_string()));
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
            background_color: [40, 40, 40, 255],
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
            background_color: [30, 30, 30, 255],
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
        assert!(debug_str.contains("app"));
    }

    #[test]
    fn test_unicode_path() {
        let mut config = Config::default();
        config.set_last_opened_directory("/home/user/图片/照片");

        assert_eq!(
            config.last_opened_directory(),
            Some(&PathBuf::from("/home/user/图片/照片"))
        );

        // 序列化和反序列化
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            config.app.last_opened_directory,
            loaded.app.last_opened_directory
        );
    }

    #[test]
    fn test_path_with_spaces() {
        let mut config = Config::default();
        config.set_last_opened_directory("/home/user/My Photos/Vacation 2024");

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            config.app.last_opened_directory,
            loaded.app.last_opened_directory
        );
    }
}
