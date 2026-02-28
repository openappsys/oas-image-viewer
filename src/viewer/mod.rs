//! 图像查看器模块 - 用于显示全尺寸图像
//!
//! 支持缩放、平移和多种显示模式。

use egui::{Color32, Rect, Sense, Ui, Vec2, Context, TextureHandle};
use std::path::PathBuf;
use tracing::debug;

use crate::config::ViewerConfig;

/// 图像查看器状态和渲染
pub struct Viewer {
    config: ViewerConfig,
    current_image: Option<ViewImage>,
    scale: f32,
    offset: Vec2,
    dragging: bool,
    ctx: Option<Context>,
    user_zoomed: bool,  // 标记用户是否手动缩放过
}

#[derive(Clone)]
pub struct ViewImage {
    pub path: PathBuf,
    pub texture: Option<TextureHandle>,
    pub dimensions: Option<(u32, u32)>,
}

impl Viewer {
    /// 使用给定配置创建新的查看器
    pub fn new(config: ViewerConfig) -> Self {
        debug!("初始化查看器，配置: {:?}", config);
        
        Self {
            config,
            current_image: None,
            scale: 1.0,
            offset: Vec2::ZERO,
            dragging: false,
            ctx: None,
            user_zoomed: false,
        }
    }

    /// 设置egui上下文
    pub fn set_ctx(&mut self, ctx: Context) {
        self.ctx = Some(ctx);
    }
    
    /// 获取上下文引用
    pub fn get_ctx(&self) -> Option<&Context> {
        self.ctx.as_ref()
    }

    /// 设置当前要显示的图像
    pub fn set_image(&mut self, path: PathBuf) {
        self.current_image = Some(ViewImage {
            path,
            texture: None,
            dimensions: None,
        });
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;  // 重置用户缩放标志
    }

    /// 设置图像和纹理
    pub fn set_image_with_texture(&mut self, path: PathBuf, texture: TextureHandle, size: [usize; 2]) {
        self.current_image = Some(ViewImage {
            path,
            texture: Some(texture),
            dimensions: Some((size[0] as u32, size[1] as u32)),
        });
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;  // 重置用户缩放标志
    }

    /// 清除当前图像
    pub fn clear(&mut self) {
        self.current_image = None;
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;  // 重置用户缩放标志
    }

    /// 渲染查看器界面
    pub fn ui(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let bg_color = Color32::from_rgb(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
        );

        // 背景 - 使用整个可用区域
        let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // 处理双击全屏
        if response.double_clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false))
            ));
        }

        // 检查是否有纹理
        let has_texture = self.current_image.as_ref()
            .map(|img| img.texture.is_some())
            .unwrap_or(false);
        
        if has_texture {
            // 安全地克隆图像数据进行渲染
            let image_clone = self.current_image.clone().unwrap();
            self.render_image(ui, &image_clone, rect, &response);
        } else if self.current_image.is_some() {
            // 图像正在加载中
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "加载中...",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
        } else {
            // 没有加载图像 - 显示占位符
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "未选择图像\n按 Ctrl+O 打开图像或从图库中选择\n也可以直接拖拽图像到窗口",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
        }

        // 信息面板
        if self.config.show_info_panel {
            self.render_info_panel(ui);
        }

        // 缩放指示器
        self.render_zoom_indicator(ui, rect);
    }

    fn render_image(
        &mut self,
        ui: &mut Ui,
        image: &ViewImage,
        rect: Rect,
        response: &egui::Response,
    ) {
        if let Some(texture) = &image.texture {
            // 基于缩放和适配模式计算显示尺寸
            let texture_size = texture.size_vec2();
            let display_size = self.calculate_display_size(texture_size, rect.size());

            // 处理拖动（平移）
            if response.dragged() {
                self.offset += response.drag_delta();
                self.dragging = true;
            } else {
                self.dragging = false;
            }

            // 使用鼠标滚轮缩放
            if response.hovered() && !self.dragging {
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 && self.config.smooth_scroll {
                    let zoom_factor = 1.0 + scroll_delta * 0.001;
                    let new_scale = (self.scale * zoom_factor)
                        .clamp(0.1, 20.0); // 10% to 2000%
                    
                    // 向鼠标位置缩放
                    if new_scale != self.scale {
                        let mouse_pos = ui.input(|i| i.pointer.hover_pos())
                            .unwrap_or(rect.center());
                        let zoom_center = mouse_pos - rect.center() - self.offset;
                        self.offset -= zoom_center * (new_scale / self.scale - 1.0);
                        self.scale = new_scale;
                        self.user_zoomed = true;  // 标记用户手动缩放
                    }
                }
            }

            // 限制绘制区域，防止遮挡顶部菜单栏
            ui.set_clip_rect(rect);
            
            // 绘制带偏移的居中图像
            let center = rect.center() + self.offset;
            let image_rect = Rect::from_center_size(center, display_size);
            
            ui.painter().image(
                texture.id(),
                image_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // 加载状态
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "加载中...",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
        }
    }

    /// 基于当前缩放和适配模式计算显示尺寸
    pub fn calculate_display_size(
        &self,
        image_size: Vec2,
        container_size: Vec2,
    ) -> Vec2 {
        // 用户手动缩放后，不再自动适配窗口，使用原始图像尺寸作为基础
        let base_size = if self.config.fit_to_window && !self.user_zoomed {
            self.fit_to_rect(image_size, container_size)
        } else {
            image_size
        };
        
        base_size * self.scale
    }

    /// 计算在保持宽高比的同时适应容器的大小
    pub fn fit_to_rect(
        &self,
        image_size: Vec2,
        container_size: Vec2,
    ) -> Vec2 {
        let scale_x = container_size.x / image_size.x;
        let scale_y = container_size.y / image_size.y;
        let scale = scale_x.min(scale_y).min(1.0);
        
        image_size * scale
    }

    /// 渲染信息面板窗口
    fn render_info_panel(
        &self,
        ui: &mut Ui,
    ) {
        let current_image = self.current_image.clone(); 
        if let Some(ref image) = current_image {
            egui::Window::new("📋 图像信息")
                .default_pos([10.0, 10.0])
                .default_size([250.0, 150.0])
                .resizable(true)
                .collapsible(true)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("路径: {}", image.path.display()));
                    
                    if let Some((w, h)) = image.dimensions {
                        ui.label(format!("尺寸: {} x {}", w, h));
                        let mp = (w as f64 * h as f64) / 1_000_000.0;
                        ui.label(format!("百万像素: {:.2} MP", mp));
                    }
                    
                    ui.separator();
                    ui.label(format!("缩放: {:.1}%", self.scale * 100.0));
                    ui.label(format!("偏移: ({:.0}, {:.0})", self.offset.x, self.offset.y));
                });
        }
    }

    /// 渲染缩放百分比指示器
    fn render_zoom_indicator(
        &self,
        ui: &mut Ui,
        rect: Rect,
    ) {
        let zoom_text = format!("{:.0}%", self.scale * 100.0);
        let pos = rect.right_bottom() - Vec2::new(10.0, 10.0);
        
        // 背景药丸
        let font = egui::FontId::proportional(12.0);
        let text_size = ui.painter().layout(
            zoom_text.clone(),
            font.clone(),
            Color32::WHITE,
            f32::INFINITY,
        ).size();
        
        let pill_rect = Rect::from_center_size(
            pos - Vec2::new(text_size.x / 2.0 + 5.0, text_size.y / 2.0 + 5.0),
            text_size + Vec2::new(16.0, 10.0),
        );
        
        ui.painter().rect_filled(
            pill_rect,
            4.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );
        
        ui.painter().text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            zoom_text,
            font,
            Color32::WHITE,
        );
    }

    /// 放大一级
    pub fn zoom_in(&mut self) {
        self.scale = (self.scale * self.config.zoom_step)
            .min(self.config.max_scale);
        self.user_zoomed = true;  // 标记用户手动缩放
    }

    /// 缩小一级
    pub fn zoom_out(&mut self) {
        self.scale = (self.scale / self.config.zoom_step)
            .max(self.config.min_scale);
        self.user_zoomed = true;  // 标记用户手动缩放
    }

    /// 重置缩放到100%
    pub fn reset_zoom(&mut self) {
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = true;  // 标记为用户手动设置，防止 fit_to_window 覆盖
    }

    /// 适应窗口
    pub fn fit_to_window(&mut self) {
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;  // 重置用户缩放标志
    }

    /// 获取当前缩放
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// 获取当前偏移
    pub fn offset(&self) -> Vec2 {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 基础初始化测试
    // =========================================================================

    #[test]
    fn test_viewer_new() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    #[test]
    fn test_viewer_with_custom_config() {
        let config = ViewerConfig {
            background_color: [50, 50, 50],
            fit_to_window: false,
            show_info_panel: true,
            min_scale: 0.05,
            max_scale: 20.0,
            zoom_step: 1.5,
            smooth_scroll: false,
        };
        let viewer = Viewer::new(config);
        assert_eq!(viewer.scale(), 1.0);
    }

    // =========================================================================
    // 缩放操作测试
    // =========================================================================

    #[test]
    fn test_zoom_in() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        assert!(viewer.scale() > 1.0);
        assert!(viewer.scale() <= 20.0); // max_scale
    }

    #[test]
    fn test_zoom_out() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_out();
        assert!(viewer.scale() < 1.0);
        assert!(viewer.scale() >= 0.1); // min_scale
    }

    #[test]
    fn test_reset_zoom() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        viewer.zoom_in();
        viewer.reset_zoom();
        
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    #[test]
    fn test_zoom_limits() {
        let config = ViewerConfig {
            min_scale: 0.1,
            max_scale: 5.0,
            zoom_step: 2.0,
            ..Default::default()
        };
        let mut viewer = Viewer::new(config);
        
        // 超出最大值的放大
        for _ in 0..10 {
            viewer.zoom_in();
        }
        assert_eq!(viewer.scale(), 5.0);
        
        // 重置并超出最小值的缩小
        viewer.reset_zoom();
        for _ in 0..10 {
            viewer.zoom_out();
        }
        assert_eq!(viewer.scale(), 0.1);
    }

    #[test]
    fn test_zoom_multiple_operations() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        let scale_after_in = viewer.scale();
        
        viewer.zoom_out();
        viewer.zoom_out();
        let scale_after_out = viewer.scale();
        
        assert!(scale_after_in > 1.0);
        assert!(scale_after_out < 1.0);
    }

    #[test]
    fn test_zoom_exact_values() {
        let config = ViewerConfig {
            min_scale: 0.1,
            max_scale: 10.0,
            zoom_step: 2.0,
            ..Default::default()
        };
        let mut viewer = Viewer::new(config);
        
        // 测试精确缩放值
        viewer.zoom_in();
        assert_eq!(viewer.scale(), 2.0);
        
        viewer.zoom_in();
        assert_eq!(viewer.scale(), 4.0);
        
        viewer.zoom_out();
        assert_eq!(viewer.scale(), 2.0);
    }

    // =========================================================================
    // 缩放边界测试
    // =========================================================================

    #[test]
    fn test_viewer_min_max_scale() {
        let config = ViewerConfig {
            background_color: [30, 30, 30],
            fit_to_window: true,
            show_info_panel: false,
            min_scale: 0.1,
            max_scale: 5.0,
            zoom_step: 2.0,
            smooth_scroll: true,
        };
        let mut viewer = Viewer::new(config);
        
        // 尝试超出最小值缩小
        for _ in 0..20 {
            viewer.zoom_out();
        }
        assert!(viewer.scale() >= 0.1);
        
        // 尝试超出最大值放大
        viewer.reset_zoom();
        for _ in 0..20 {
            viewer.zoom_in();
        }
        assert!(viewer.scale() <= 5.0);
    }

    #[test]
    fn test_zoom_at_minimum() {
        let config = ViewerConfig {
            min_scale: 1.0,
            max_scale: 5.0,
            zoom_step: 1.5,
            ..Default::default()
        };
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_out();
        assert_eq!(viewer.scale(), 1.0);
    }

    #[test]
    fn test_zoom_at_maximum() {
        let config = ViewerConfig {
            min_scale: 0.1,
            max_scale: 1.0,
            zoom_step: 1.5,
            ..Default::default()
        };
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        assert_eq!(viewer.scale(), 1.0);
    }

    // =========================================================================
    // 适应窗口测试
    // =========================================================================

    #[test]
    fn test_viewer_fit_to_window() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        viewer.zoom_in();
        viewer.fit_to_window();
        
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    // =========================================================================
    // 偏移测试
    // =========================================================================

    #[test]
    fn test_viewer_offset_after_operations() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.set_image(std::path::PathBuf::from("test.png"));
        assert_eq!(viewer.offset(), Vec2::ZERO);
        
        viewer.zoom_in();
        viewer.reset_zoom();
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    // =========================================================================
    // 图像设置测试
    // =========================================================================

    #[test]
    fn test_set_and_clear_image() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.set_image(std::path::PathBuf::from("test.png"));
        // 图像应该已设置（在没有egui上下文的情况下不易验证）
        
        viewer.clear();
        // 应该重置状态
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    #[test]
    fn test_set_image_resets_state() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        viewer.zoom_in();
        assert!(viewer.scale() > 1.0);
        
        viewer.set_image(std::path::PathBuf::from("new_image.png"));
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    #[test]
    fn test_clear_when_empty() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        // 清空已经空的查看器
        viewer.clear();
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    // =========================================================================
    // 显示尺寸计算测试
    // =========================================================================

    #[test]
    fn test_calculate_display_size_no_fit() {
        let config = ViewerConfig {
            fit_to_window: false,
            ..Default::default()
        };
        let viewer = Viewer::new(config);
        
        let image_size = Vec2::new(100.0, 100.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.calculate_display_size(image_size, container_size);
        assert_eq!(result, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_calculate_display_size_with_fit() {
        let config = ViewerConfig {
            fit_to_window: true,
            ..Default::default()
        };
        let viewer = Viewer::new(config);
        
        // 图像大于容器
        let image_size = Vec2::new(1000.0, 1000.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.calculate_display_size(image_size, container_size);
        assert_eq!(result, Vec2::new(500.0, 500.0));
    }

    #[test]
    fn test_calculate_display_size_with_scale() {
        let config = ViewerConfig {
            fit_to_window: true,
            ..Default::default()
        };
        let mut viewer = Viewer::new(config);
        
        // 放大后不应再适配
        viewer.zoom_in(); // scale = 1.25
        
        let image_size = Vec2::new(100.0, 100.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.calculate_display_size(image_size, container_size);
        assert_eq!(result, Vec2::new(125.0, 125.0)); // 100 * 1.25
    }

    #[test]
    fn test_calculate_display_size_scaled() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        viewer.zoom_in();
        viewer.zoom_in(); // scale = 1.5625
        
        let image_size = Vec2::new(100.0, 100.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.calculate_display_size(image_size, container_size);
        assert_eq!(result.x, 156.25);
        assert_eq!(result.y, 156.25);
    }

    // =========================================================================
    // 适应矩形测试
    // =========================================================================

    #[test]
    fn test_fit_to_rect_smaller_image() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 图像小于容器 - 不应放大（max 1.0）
        let image_size = Vec2::new(100.0, 100.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_fit_to_rect_larger_image() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 图像大于容器 - 应缩小
        let image_size = Vec2::new(1000.0, 1000.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(500.0, 500.0));
    }

    #[test]
    fn test_fit_to_rect_wide_image() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 宽图像在窄容器中
        let image_size = Vec2::new(1000.0, 500.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(500.0, 250.0));
    }

    #[test]
    fn test_fit_to_rect_tall_image() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 高图像在矮容器中
        let image_size = Vec2::new(500.0, 1000.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(250.0, 500.0));
    }

    #[test]
    fn test_fit_to_rect_equal_size() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 图像和容器相同大小
        let image_size = Vec2::new(500.0, 500.0);
        let container_size = Vec2::new(500.0, 500.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(500.0, 500.0));
    }

    #[test]
    fn test_fit_to_rect_zero_container() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 零大小容器
        let image_size = Vec2::new(100.0, 100.0);
        let container_size = Vec2::new(0.0, 0.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_fit_to_rect_aspect_ratio_preservation() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        
        // 测试保持宽高比
        let image_size = Vec2::new(1600.0, 900.0);
        let container_size = Vec2::new(800.0, 800.0);
        
        let result = viewer.fit_to_rect(image_size, container_size);
        // 应适配宽度，高度按比例缩放
        assert_eq!(result.x, 800.0);
        assert_eq!(result.y, 450.0);
    }

    // =========================================================================
    // 边界条件测试
    // =========================================================================

    #[test]
    fn test_very_small_zoom_step() {
        let config = ViewerConfig {
            min_scale: 0.01,
            max_scale: 100.0,
            zoom_step: 1.01,
            ..Default::default()
        };
        let mut viewer = Viewer::new(config);
        
        // 小步长多次缩放
        for _ in 0..100 {
            viewer.zoom_in();
        }
        assert!(viewer.scale() <= 100.0);
    }

    #[test]
    fn test_alternating_zoom() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        
        // 交替放大和缩小
        for _ in 0..10 {
            viewer.zoom_in();
            viewer.zoom_out();
        }
        
        // 由于浮点精度，可能不完全等于1.0
        assert!((viewer.scale() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_multiple_viewers() {
        let config1 = ViewerConfig::default();
        let config2 = ViewerConfig {
            zoom_step: 2.0,
            ..Default::default()
        };
        
        let mut viewer1 = Viewer::new(config1);
        let mut viewer2 = Viewer::new(config2);
        
        viewer1.zoom_in();
        viewer2.zoom_in();
        
        // 不同缩放步长应产生不同结果
        assert_ne!(viewer1.scale(), viewer2.scale());
    }
}
