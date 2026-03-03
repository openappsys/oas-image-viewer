//! 值对象 - 用于业务逻辑的类型定义
//!
//! 这些类型表示领域中的数值概念，具有不变性和验证

/// 缩放比例值对象
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Scale {
    value: f32,
}

/// 二维位置值对象
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// 尺寸值对象
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// 颜色值对象
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// 显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisplayMode {
    /// 适应窗口
    FitToWindow,
    /// 原始大小
    OriginalSize,
    /// 自定义缩放
    CustomScale,
}

/// 视图状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViewMode {
    /// 画廊视图
    Gallery,
    /// 单图查看视图
    Viewer,
}

/// 导航方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NavigationDirection {
    Next,
    Previous,
    First,
    Last,
}

/// 画廊布局配置
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GalleryLayout {
    pub thumbnail_size: u32,
    pub items_per_row: usize,
    pub grid_spacing: f32,
    pub show_filenames: bool,
}

/// 查看器配置
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewerSettings {
    pub background_color: Color,
    pub fit_to_window: bool,
    pub show_info_panel: bool,
    pub min_scale: f32,
    pub max_scale: f32,
    pub zoom_step: f32,
    pub smooth_scroll: bool,
    /// 关于窗口位置（x, y）
    pub about_window_pos: Option<Position>,
    /// 快捷键帮助窗口位置（x, y）
    pub shortcuts_window_pos: Option<Position>,
}

/// 窗口状态
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub maximized: bool,
}

impl Scale {
    /// 创建新的缩放比例，自动限制在有效范围内
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value: value.clamp(min, max),
        }
    }

    /// 创建默认缩放（1.0 = 100%）
    pub fn default_value() -> Self {
        Self { value: 1.0 }
    }

    /// 获取原始值
    pub fn value(&self) -> f32 {
        self.value
    }

    /// 增加缩放
    pub fn zoom_in(&mut self, step: f32, max: f32) {
        self.value = (self.value * step).min(max);
    }

    /// 减小缩放
    pub fn zoom_out(&mut self, step: f32, min: f32) {
        self.value = (self.value / step).max(min);
    }

    /// 重置为 1.0
    pub fn reset(&mut self) {
        self.value = 1.0;
    }

    /// 计算百分比显示
    pub fn percentage(&self) -> i32 {
        (self.value * 100.0).round() as i32
    }
}

impl Position {
    /// 创建新位置
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// 移动位置
    pub fn translate(&mut self, delta_x: f32, delta_y: f32) {
        self.x += delta_x;
        self.y += delta_y;
    }

    /// 重置为原点
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }

    /// 计算两点距离
    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Dimensions {
    /// 创建新尺寸
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 从宽高比和宽度计算高度
    pub fn from_aspect_ratio(width: u32, aspect_ratio: f32) -> Self {
        let height = (width as f32 / aspect_ratio).round() as u32;
        Self { width, height }
    }

    /// 计算面积
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// 计算宽高比
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            return 0.0;
        }
        self.width as f32 / self.height as f32
    }

    /// 计算适应目标尺寸的缩放比例
    pub fn fit_scale(&self, target: &Dimensions) -> f32 {
        let scale_x = target.width as f32 / self.width as f32;
        let scale_y = target.height as f32 / self.height as f32;
        scale_x.min(scale_y).min(1.0)
    }

    /// 缩放到指定比例
    pub fn scale(&self, scale: f32) -> (f32, f32) {
        (self.width as f32 * scale, self.height as f32 * scale)
    }
}

impl Color {
    /// 创建 RGB 颜色（不透明）
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// 创建 RGBA 颜色
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// 预乘 Alpha
    pub fn premultiply(&self) -> [u8; 4] {
        let alpha = self.a as f32 / 255.0;
        [
            (self.r as f32 * alpha) as u8,
            (self.g as f32 * alpha) as u8,
            (self.b as f32 * alpha) as u8,
            self.a,
        ]
    }

    /// 转换为 u32 RGBA
    #[allow(clippy::wrong_self_convention)]
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(0, 0, 0)
    }
}

impl GalleryLayout {
    /// 验证并修正配置
    pub fn validated(&self) -> Self {
        const MIN_THUMBNAIL: u32 = 80;
        const MAX_THUMBNAIL: u32 = 200;

        Self {
            thumbnail_size: self.thumbnail_size.clamp(MIN_THUMBNAIL, MAX_THUMBNAIL),
            items_per_row: self.items_per_row.max(1),
            grid_spacing: self.grid_spacing.max(0.0),
            show_filenames: self.show_filenames,
        }
    }

    /// 基于可用宽度计算每行项目数
    pub fn calculate_items_per_row(&self, available_width: f32) -> usize {
        if self.items_per_row > 0 {
            return self.items_per_row;
        }
        let item_width = self.thumbnail_size as f32 + self.grid_spacing;
        (available_width / item_width).max(1.0) as usize
    }
}

impl Default for GalleryLayout {
    fn default() -> Self {
        Self {
            thumbnail_size: 120,
            items_per_row: 0, // 自动计算
            grid_spacing: 12.0,
            show_filenames: true,
        }
    }
}

impl ViewerSettings {
    /// 验证并修正配置
    pub fn validated(&self) -> Self {
        let min_scale = self.min_scale.max(0.01);
        let max_scale = self.max_scale.max(min_scale * 2.0);
        let zoom_step = self.zoom_step.clamp(1.01, 2.0);

        Self {
            background_color: self.background_color,
            fit_to_window: self.fit_to_window,
            show_info_panel: self.show_info_panel,
            min_scale,
            max_scale,
            zoom_step,
            smooth_scroll: self.smooth_scroll,
            about_window_pos: self.about_window_pos,
            shortcuts_window_pos: self.shortcuts_window_pos,
        }
    }
}

impl Default for ViewerSettings {
    fn default() -> Self {
        Self {
            background_color: Color::rgb(30, 30, 30),
            fit_to_window: true,
            show_info_panel: false,
            min_scale: 0.1,
            max_scale: 20.0,
            zoom_step: 1.25,
            smooth_scroll: true,
            about_window_pos: None,
            shortcuts_window_pos: None,
        }
    }
}

impl WindowState {
    /// 获取位置数组
    pub fn position(&self) -> Option<[f32; 2]> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Some([x, y]),
            _ => None,
        }
    }

    /// 获取大小数组
    pub fn size(&self) -> [f32; 2] {
        [self.width, self.height]
    }
}

impl Default for WindowState {
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

impl NavigationDirection {
    /// 从字符串解析导航方向
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "next" | "forward" => Some(Self::Next),
            "prev" | "previous" | "back" => Some(Self::Previous),
            "first" => Some(Self::First),
            "last" => Some(Self::Last),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_new() {
        let scale = Scale::new(1.5, 0.1, 5.0);
        assert!((scale.value() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_scale_clamp() {
        let scale_low = Scale::new(0.01, 0.1, 5.0);
        assert!((scale_low.value() - 0.1).abs() < 0.001);

        let scale_high = Scale::new(10.0, 0.1, 5.0);
        assert!((scale_high.value() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_scale_zoom() {
        let mut scale = Scale::default_value();
        scale.zoom_in(1.2, 5.0);
        assert!(scale.value() > 1.0);

        scale.zoom_out(1.2, 0.1);
        assert!((scale.value() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_percentage() {
        let scale = Scale::new(1.5, 0.1, 5.0);
        assert_eq!(scale.percentage(), 150);
    }

    #[test]
    fn test_position() {
        let mut pos = Position::new(10.0, 20.0);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);

        pos.translate(5.0, -5.0);
        assert_eq!(pos.x, 15.0);
        assert_eq!(pos.y, 15.0);

        pos.reset();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_dimensions() {
        let dim = Dimensions::new(1920, 1080);
        assert_eq!(dim.width, 1920);
        assert_eq!(dim.height, 1080);
        assert_eq!(dim.area(), 1920 * 1080);
        assert!((dim.aspect_ratio() - 1.777).abs() < 0.01);
    }

    #[test]
    fn test_dimensions_from_aspect_ratio() {
        let dim = Dimensions::from_aspect_ratio(1920, 16.0 / 9.0);
        assert_eq!(dim.width, 1920);
        assert_eq!(dim.height, 1080);
    }

    #[test]
    fn test_dimensions_fit_scale() {
        let image = Dimensions::new(1000, 1000);
        let container = Dimensions::new(500, 500);
        assert!((image.fit_scale(&container) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color() {
        let color = Color::rgb(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_rgba() {
        let color = Color::rgba(255, 128, 64, 128);
        assert_eq!(color.a, 128);
    }

    #[test]
    fn test_gallery_layout_validated() {
        let layout = GalleryLayout {
            thumbnail_size: 50, // 太小
            items_per_row: 0,
            grid_spacing: -5.0, // 负数
            show_filenames: true,
        };
        let validated = layout.validated();
        assert_eq!(validated.thumbnail_size, 80); // 被限制到最小值
        assert_eq!(validated.items_per_row, 1); // 被限制到最小值
        assert_eq!(validated.grid_spacing, 0.0); // 负数被修正
    }

    #[test]
    fn test_gallery_layout_calculate_items() {
        let layout = GalleryLayout {
            thumbnail_size: 100,
            items_per_row: 0, // 自动计算
            grid_spacing: 10.0,
            show_filenames: true,
        };
        // 可用宽度 500，每个项目 110，应该能放 4 个
        let items = layout.calculate_items_per_row(500.0);
        assert_eq!(items, 4);
    }

    #[test]
    fn test_viewer_settings_validated() {
        let settings = ViewerSettings {
            min_scale: 5.0,
            max_scale: 1.0, // 小于最小值
            zoom_step: 0.5, // 太小
            ..Default::default()
        };
        let validated = settings.validated();
        assert!(validated.max_scale >= validated.min_scale * 2.0);
        assert!(validated.zoom_step >= 1.01);
    }

    #[test]
    fn test_navigation_direction() {
        assert_eq!(
            NavigationDirection::from_str("next"),
            Some(NavigationDirection::Next)
        );
        assert_eq!(
            NavigationDirection::from_str("PREV"),
            Some(NavigationDirection::Previous)
        );
        assert_eq!(NavigationDirection::from_str("invalid"), None);
    }

    #[test]
    fn test_window_state() {
        let state = WindowState {
            width: 1920.0,
            height: 1080.0,
            x: Some(100.0),
            y: Some(200.0),
            maximized: true,
        };
        assert_eq!(state.size(), [1920.0, 1080.0]);
        assert_eq!(state.position(), Some([100.0, 200.0]));
    }

    #[test]
    fn test_window_state_partial_position() {
        let state = WindowState {
            x: Some(100.0),
            y: None,
            ..Default::default()
        };
        assert_eq!(state.position(), None);
    }

    #[test]
    fn test_display_mode() {
        assert_ne!(DisplayMode::FitToWindow, DisplayMode::OriginalSize);
        assert_eq!(DisplayMode::FitToWindow, DisplayMode::FitToWindow);
    }

    #[test]
    fn test_view_mode() {
        assert_ne!(ViewMode::Gallery, ViewMode::Viewer);
        assert_eq!(ViewMode::Gallery, ViewMode::Gallery);
    }

    #[test]
    fn test_dimensions_zero_height() {
        let dim = Dimensions::new(100, 0);
        assert_eq!(dim.aspect_ratio(), 0.0);
    }

    #[test]
    fn test_color_premultiply() {
        let color = Color::rgba(255, 128, 64, 128);
        let premultiplied = color.premultiply();
        // 128/255 ≈ 0.5，所以 R 应该是约 128
        assert!(premultiplied[0] < 255);
        assert_eq!(premultiplied[3], 128); // Alpha 不变
    }

    // =========================================================================
    // Bug 修复回归测试
    // =========================================================================

    /// Bug 3 回归测试：验证 ViewerSettings 支持窗口位置记忆
    #[test]
    fn test_viewer_settings_window_position_memory() {
        let settings = ViewerSettings {
            about_window_pos: Some(Position::new(100.0, 200.0)),
            shortcuts_window_pos: Some(Position::new(300.0, 400.0)),
            ..Default::default()
        };

        // 验证位置正确保存
        assert_eq!(settings.about_window_pos.unwrap().x, 100.0);
        assert_eq!(settings.about_window_pos.unwrap().y, 200.0);
        assert_eq!(settings.shortcuts_window_pos.unwrap().x, 300.0);
        assert_eq!(settings.shortcuts_window_pos.unwrap().y, 400.0);

        // 验证 validated 保留位置信息
        let validated = settings.validated();
        assert!(validated.about_window_pos.is_some());
        assert!(validated.shortcuts_window_pos.is_some());
    }

    /// Bug 3 回归测试：验证默认情况下窗口位置为 None
    #[test]
    fn test_viewer_settings_default_window_positions() {
        let settings = ViewerSettings::default();
        assert!(settings.about_window_pos.is_none());
        assert!(settings.shortcuts_window_pos.is_none());
    }

    /// Bug 2 回归测试：验证 ViewerSettings 支持 show_info_panel 字段
    #[test]
    fn test_viewer_settings_show_info_panel() {
        let settings = ViewerSettings {
            show_info_panel: true,
            ..Default::default()
        };
        assert!(settings.show_info_panel);

        let settings = ViewerSettings::default();
        assert!(!settings.show_info_panel);
    }
}
