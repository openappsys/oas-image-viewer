//! 图像查看器模块 - 用于显示全尺寸图像
//!
//! 支持缩放、平移和多种显示模式。

use egui::{Color32, Context, Rect, Sense, TextureHandle, Ui, Vec2};
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::clipboard::ClipboardManager;
use crate::config::ViewerConfig;
use crate::info_panel::InfoPanel;

/// 图像查看器状态和渲染
pub struct Viewer {
    config: ViewerConfig,
    current_image: Option<ViewImage>,
    scale: f32,
    offset: Vec2,
    dragging: bool,
    ctx: Option<Context>,
    user_zoomed: bool,
    base_scale: f32,  // 自适应窗口时的基准缩放比例
    info_panel: InfoPanel,
    clipboard: ClipboardManager,
    context_menu_open: bool,
    last_copy_result: Option<String>,
}

#[derive(Clone)]
pub struct ViewImage {
    pub path: PathBuf,
    pub texture: Option<TextureHandle>,
    pub dimensions: Option<(u32, u32)>,
    pub texture_data: Option<Vec<u8>>,
}

impl Viewer {
    pub fn new(config: ViewerConfig) -> Self {
        debug!("初始化查看器");
        let info_panel = InfoPanel::with_visibility(config.show_info_panel);
        let clipboard = ClipboardManager::new();

        Self {
            config,
            current_image: None,
            scale: 1.0,
            offset: Vec2::ZERO,
            dragging: false,
            ctx: None,
            user_zoomed: false,
            base_scale: 1.0,
            info_panel,
            clipboard,
            context_menu_open: false,
            last_copy_result: None,
        }
    }

    pub fn set_ctx(&mut self, ctx: Context) {
        self.ctx = Some(ctx);
    }

    pub fn get_ctx(&self) -> Option<&Context> {
        self.ctx.as_ref()
    }

    pub fn set_image(&mut self, path: PathBuf) {
        self.current_image = Some(ViewImage {
            path,
            texture: None,
            dimensions: None,
            texture_data: None,
        });
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;
        self.base_scale = 1.0;
        self.info_panel.clear();
    }

    pub fn set_image_with_texture(
        &mut self,
        path: PathBuf,
        texture: TextureHandle,
        size: [usize; 2],
    ) {
        let dimensions = (size[0] as u32, size[1] as u32);
        let format = Self::detect_image_format(&path);
        self.info_panel.set_image_info(&path, dimensions, &format);

        self.current_image = Some(ViewImage {
            path,
            texture: Some(texture),
            dimensions: Some(dimensions),
            texture_data: None,
        });
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;
        self.base_scale = 1.0;
    }

    pub fn set_image_with_texture_and_data(
        &mut self,
        path: PathBuf,
        texture: TextureHandle,
        size: [usize; 2],
        rgba_data: Vec<u8>,
    ) {
        let dimensions = (size[0] as u32, size[1] as u32);
        let format = Self::detect_image_format(&path);
        self.info_panel.set_image_info(&path, dimensions, &format);

        self.current_image = Some(ViewImage {
            path,
            texture: Some(texture),
            dimensions: Some(dimensions),
            texture_data: Some(rgba_data),
        });
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;
        self.base_scale = 1.0;
    }

    pub fn clear(&mut self) {
        self.current_image = None;
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;
        self.base_scale = 1.0;
        self.info_panel.clear();
    }

    fn detect_image_format(path: &PathBuf) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| if e.is_empty() { "Unknown".to_string() } else { e.to_uppercase() })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn handle_input(&mut self, ctx: &Context) -> bool {
        if self.info_panel.handle_input(ctx) {
            return true;
        }
        false
    }

    fn copy_image_to_clipboard(&mut self) {
        if let Some(ref image) = self.current_image {
            let result = if let Some(ref data) = image.texture_data {
                if let Some((w, h)) = image.dimensions {
                    self.clipboard.copy_image(data, w as usize, h as usize)
                } else {
                    Err(crate::clipboard::ClipboardError::InvalidImage(
                        "未知的图片尺寸".to_string(),
                    ))
                }
            } else {
                self.clipboard.copy_image_from_file(&image.path, None)
            };

            match result {
                Ok(_) => {
                    info!("图片已复制到剪贴板");
                    self.last_copy_result = Some("图片已复制".to_string());
                }
                Err(e) => {
                    error!("复制图片失败: {}", e);
                    self.last_copy_result = Some(format!("复制失败: {}", e));
                }
            }
        }
    }

    fn copy_path_to_clipboard(&mut self) {
        if let Some(ref image) = self.current_image {
            match self.clipboard.copy_image_path(&image.path) {
                Ok(_) => {
                    info!("路径已复制到剪贴板");
                    self.last_copy_result = Some("路径已复制".to_string());
                }
                Err(e) => {
                    error!("复制路径失败: {}", e);
                    self.last_copy_result = Some(format!("复制失败: {}", e));
                }
            }
        }
    }

    fn show_in_folder(&self) {
        if let Some(ref image) = self.current_image {
            if let Err(e) = ClipboardManager::show_in_folder(&image.path) {
                error!("无法打开文件夹: {}", e);
            }
        }
    }

    fn render_context_menu(&mut self, ui: &mut Ui, _response: &egui::Response) {
        let has_image = self.current_image.is_some();
        let clipboard_available = self.clipboard.is_available();

        ui.set_min_width(150.0);

        let copy_image_btn = ui.add_enabled(
            has_image && clipboard_available,
            egui::Button::new("📋 复制图片"),
        );
        if copy_image_btn.clicked() {
            self.copy_image_to_clipboard();
            ui.close_menu();
        }

        let copy_path_btn = ui.add_enabled(
            has_image && clipboard_available,
            egui::Button::new("📂 复制文件路径"),
        );
        if copy_path_btn.clicked() {
            self.copy_path_to_clipboard();
            ui.close_menu();
        }

        ui.separator();

        let show_in_folder_btn = ui.add_enabled(
            has_image,
            egui::Button::new("📁 在文件夹中显示"),
        );
        if show_in_folder_btn.clicked() {
            self.show_in_folder();
            ui.close_menu();
        }

        if let Some(ref result) = self.last_copy_result {
            ui.separator();
            ui.label(egui::RichText::new(result).size(11.0).color(ui.visuals().weak_text_color()));
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        self.info_panel.ui(ui.ctx());

        let available_size = ui.available_size();
        let bg_color = Color32::from_rgb(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
        );

        let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
        ui.painter().rect_filled(rect, 0.0, bg_color);

        if response.double_clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                    !ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false)),
                ));
        }

        response.clone().context_menu(|ui| {
            self.context_menu_open = true;
            self.render_context_menu(ui, &response);
        });

        let has_texture = self
            .current_image
            .as_ref()
            .map(|img| img.texture.is_some())
            .unwrap_or(false);

        if has_texture {
            let image_clone = self.current_image.clone().unwrap();
            self.render_image(ui, &image_clone, rect, &response);
        } else if self.current_image.is_some() {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "加载中...",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
        } else {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "未选择图像\n按 Ctrl+O 打开图像或从图库中选择\n也可以直接拖拽图像到窗口",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
        }

        self.render_zoom_indicator(ui, rect);
        self.render_dimensions_indicator(ui, rect);
    }

    fn render_image(
        &mut self, ui: &mut Ui, image: &ViewImage, rect: Rect, response: &egui::Response,
    ) {
        if let Some(texture) = &image.texture {
            let texture_size = texture.size_vec2();
            let display_size = self.calculate_display_size(texture_size, rect.size());

            if response.dragged() {
                self.offset += response.drag_delta();
                self.dragging = true;
            } else {
                self.dragging = false;
            }

            if response.hovered() && !self.dragging {
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 && self.config.smooth_scroll {
                    let zoom_factor = 1.0 + scroll_delta * 0.001;
                    let new_scale = (self.scale * zoom_factor).clamp(0.1, 20.0);

                    if new_scale != self.scale {
                        let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(rect.center());
                        let zoom_center = mouse_pos - rect.center() - self.offset;
                        self.offset -= zoom_center * (new_scale / self.scale - 1.0);
                        self.scale = new_scale;
                        self.user_zoomed = true;
                    }
                }
            }

            ui.set_clip_rect(rect);

            let center = rect.center() + self.offset;
            let image_rect = Rect::from_center_size(center, display_size);

            ui.painter().image(
                texture.id(),
                image_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "加载中...",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
        }
    }

    pub fn calculate_display_size(&self, image_size: Vec2, container_size: Vec2) -> Vec2 {
        // 计算自适应尺寸和对应的基准缩放比例
        let fitted_size = self.fit_to_rect(image_size, container_size);
        let fitted_scale = fitted_size.x / image_size.x; // 自适应时的缩放比例
        
        if self.config.fit_to_window && !self.user_zoomed {
            // 首次显示或重置后，使用自适应尺寸
            fitted_size
        } else {
            // 用户手动缩放后，基于自适应尺寸进行缩放
            // 实际缩放比例 = 自适应比例 * 用户缩放倍数
            let effective_scale = fitted_scale * self.scale;
            image_size * effective_scale
        }
    }
    
    /// 获取当前实际缩放比例（相对于原始尺寸）
    pub fn current_scale(&self, image_size: Vec2, container_size: Vec2) -> f32 {
        let fitted_scale = self.fit_to_rect(image_size, container_size).x / image_size.x;
        if self.config.fit_to_window && !self.user_zoomed {
            fitted_scale
        } else {
            fitted_scale * self.scale
        }
    }

    pub fn fit_to_rect(&self, image_size: Vec2, container_size: Vec2) -> Vec2 {
        let scale_x = container_size.x / image_size.x;
        let scale_y = container_size.y / image_size.y;
        let scale = scale_x.min(scale_y).min(1.0);
        image_size * scale
    }

    fn render_zoom_indicator(&self, ui: &mut Ui, rect: Rect) {
        let zoom_text = format!("{:.0}%", self.scale * 100.0);
        let pos = rect.right_bottom() - Vec2::new(10.0, 10.0);
        let font = egui::FontId::proportional(12.0);
        let text_size = ui
            .painter()
            .layout(zoom_text.clone(), font.clone(), Color32::WHITE, f32::INFINITY)
            .size();
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

    fn render_dimensions_indicator(&self, ui: &mut Ui, rect: Rect) {
        let dimensions_text = if let Some(ref image) = self.current_image {
            if let Some((width, height)) = image.dimensions {
                let mp = (width as f64 * height as f64) / 1_000_000.0;
                format!("{}x{} / {:.1} MP", width, height, mp)
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        };
        let pos = rect.left_bottom() + Vec2::new(10.0, -10.0);
        let font = egui::FontId::proportional(12.0);
        let text_size = ui
            .painter()
            .layout(dimensions_text.clone(), font.clone(), Color32::WHITE, f32::INFINITY)
            .size();
        let pill_rect = Rect::from_center_size(
            pos + Vec2::new(text_size.x / 2.0 + 5.0, -text_size.y / 2.0 - 5.0),
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
            dimensions_text,
            font,
            Color32::WHITE,
        );
    }

    pub fn zoom_in(&mut self) {
        // 首次缩放时，从当前显示比例开始（scale=1.0 对应自适应尺寸）
        if !self.user_zoomed {
            self.scale = 1.0;
            self.user_zoomed = true;
        }
        self.scale = (self.scale * self.config.zoom_step).min(self.config.max_scale);
    }

    pub fn zoom_out(&mut self) {
        if !self.user_zoomed {
            self.scale = 1.0;
            self.user_zoomed = true;
        }
        self.scale = (self.scale / self.config.zoom_step).max(self.config.min_scale);
    }

    pub fn reset_zoom(&mut self) {
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = true;
    }

    pub fn fit_to_window(&mut self) {
        self.scale = 1.0;
        self.offset = Vec2::ZERO;
        self.user_zoomed = false;
        self.base_scale = 1.0;
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    pub fn info_panel(&self) -> &InfoPanel {
        &self.info_panel
    }

    pub fn info_panel_mut(&mut self) -> &mut InfoPanel {
        &mut self.info_panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_new() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        assert_eq!(viewer.scale(), 1.0);
        assert_eq!(viewer.offset(), Vec2::ZERO);
    }

    #[test]
    fn test_zoom_in() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        viewer.zoom_in();
        assert!(viewer.scale() > 1.0);
    }

    #[test]
    fn test_zoom_out() {
        let config = ViewerConfig::default();
        let mut viewer = Viewer::new(config);
        viewer.zoom_out();
        assert!(viewer.scale() < 1.0);
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
    fn test_fit_to_rect_smaller_image() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        let image_size = Vec2::new(100.0, 100.0);
        let container_size = Vec2::new(500.0, 500.0);
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_fit_to_rect_larger_image() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        let image_size = Vec2::new(1000.0, 1000.0);
        let container_size = Vec2::new(500.0, 500.0);
        let result = viewer.fit_to_rect(image_size, container_size);
        assert_eq!(result, Vec2::new(500.0, 500.0));
    }

    #[test]
    fn test_detect_image_format() {
        assert_eq!(
            Viewer::detect_image_format(&std::path::PathBuf::from("test.png")),
            "PNG"
        );
        assert_eq!(
            Viewer::detect_image_format(&std::path::PathBuf::from("test.jpg")),
            "JPG"
        );
    }

    #[test]
    fn test_clipboard_availability() {
        let config = ViewerConfig::default();
        let viewer = Viewer::new(config);
        let _available = viewer.clipboard.is_available();
    }
}
