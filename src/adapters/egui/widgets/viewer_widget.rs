//! Viewer Widget - 查看器 UI 组件

use crate::core::domain::ViewerSettings;
use crate::core::use_cases::ViewState;
use egui::{Color32, Rect, Sense, Ui, Vec2};

/// 查看器组件
#[derive(Default)]
pub struct ViewerWidget {
    dragging: bool,
    /// 累积的滚轮增量，用于平滑缩放
    zoom_accumulator: f32,
}

impl ViewerWidget {
    /// 渲染查看器
    /// 返回 (是否双击全屏, 缩放变化量, 拖拽偏移量)
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        state: &ViewState,
        settings: &ViewerSettings,
        texture: Option<&(String, egui::TextureHandle)>,
    ) -> (bool, f32, Option<Vec2>) {
        let available_size = ui.available_size();
        let bg_color = Color32::from_rgb(
            settings.background_color.r,
            settings.background_color.g,
            settings.background_color.b,
        );

        let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // 处理双击全屏
        let double_clicked = response.double_clicked();

        // 处理拖拽平移
        let drag_delta = if response.dragged() {
            self.dragging = true;
            ui.input(|i| i.pointer.delta())
        } else {
            self.dragging = false;
            Vec2::ZERO
        };

        // 处理滚轮缩放
        let mut zoom_delta = 0.0;
        if response.hovered() && !self.dragging {
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                // 累积滚轮增量
                self.zoom_accumulator += scroll_delta;
                
                // 每累积一定量就触发一次缩放
                const ZOOM_THRESHOLD: f32 = 10.0;
                if self.zoom_accumulator.abs() >= ZOOM_THRESHOLD {
                    // 根据方向确定缩放因子
                    zoom_delta = if self.zoom_accumulator > 0.0 { 1.1 } else { 0.9 };
                    self.zoom_accumulator = 0.0; // 重置累积器
                }
            } else {
                // 没有滚轮输入时，逐渐减少累积值（平滑过渡）
                self.zoom_accumulator *= 0.9;
                if self.zoom_accumulator.abs() < 1.0 {
                    self.zoom_accumulator = 0.0;
                }
            }
        }

        // 渲染图像或占位符
        if let Some(ref image) = state.current_image {
            self.render_image(ui, image, state, rect, &response, settings, texture);
        } else {
            // 无图像占位符
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "未选择图像\n按 Ctrl+O 打开图像或从图库中选择\n也可以直接拖拽图像到窗口",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
        }

        // 渲染缩放指示器
        self.render_zoom_indicator(ui, rect, state);

        // 渲染尺寸指示器
        self.render_dimensions_indicator(ui, rect, state);
        
        let drag_offset = if drag_delta != Vec2::ZERO { Some(drag_delta) } else { None };
        (double_clicked, zoom_delta, drag_offset)
    }

    /// 渲染图像
    fn render_image(
        &self,
        ui: &mut Ui,
        image: &crate::core::domain::Image,
        state: &ViewState,
        rect: Rect,
        _response: &egui::Response,
        _settings: &ViewerSettings,
        texture: Option<&(String, egui::TextureHandle)>,
    ) {
        // 如果有纹理，渲染实际图像
        if let Some((_, texture_handle)) = texture {
            // 计算缩放后的图像尺寸
            let img_size = texture_handle.size_vec2();
            let scale = state.scale.value();
            let scaled_size = img_size * scale;

            // 计算居中位置（考虑偏移）
            let center = rect.center() + Vec2::new(state.offset.x, state.offset.y);
            let image_rect = Rect::from_center_size(center, scaled_size);

            // 渲染图像纹理
            ui.painter().image(
                texture_handle.id(),
                image_rect,
                Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // 纹理加载中或失败，显示文件名作为占位
            let center = rect.center();
            let text = format!(
                "{}\n{}x{}",
                image.file_name().unwrap_or("Unknown"),
                image.metadata().width,
                image.metadata().height,
            );

            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        }

        // 缩放指示
        if state.user_zoomed {
            let zoom_text = format!("{:.0}%", state.scale.percentage());
            ui.painter().text(
                rect.center() + Vec2::new(0.0, 30.0),
                egui::Align2::CENTER_CENTER,
                zoom_text,
                egui::FontId::proportional(12.0),
                Color32::GRAY,
            );
        }
    }

    /// 渲染缩放指示器
    fn render_zoom_indicator(&self, ui: &mut Ui, rect: Rect, state: &ViewState) {
        let zoom_text = format!("{:.0}%", state.scale.percentage());
        let pos = rect.right_bottom() - Vec2::new(10.0, 10.0);
        let font = egui::FontId::proportional(12.0);

        let text_size = ui
            .painter()
            .layout(
                zoom_text.clone(),
                font.clone(),
                Color32::WHITE,
                f32::INFINITY,
            )
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

    /// 渲染尺寸指示器
    fn render_dimensions_indicator(&self, ui: &mut Ui, rect: Rect, state: &ViewState) {
        let dimensions_text = if let Some(ref image) = state.current_image {
            let mp = image.megapixels();
            format!(
                "{}x{} / {:.1} MP",
                image.metadata().width,
                image.metadata().height,
                mp
            )
        } else {
            "-".to_string()
        };

        let pos = rect.left_bottom() + Vec2::new(10.0, -10.0);
        let font = egui::FontId::proportional(12.0);

        let text_size = ui
            .painter()
            .layout(
                dimensions_text.clone(),
                font.clone(),
                Color32::WHITE,
                f32::INFINITY,
            )
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
}
