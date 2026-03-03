//! Viewer Widget - 查看器 UI 组件

use crate::core::domain::ViewerSettings;
use crate::core::use_cases::ViewState;
use egui::{Color32, Rect, Sense, Ui, Vec2};

/// 查看器组件
#[derive(Default)]
pub struct ViewerWidget {
    dragging: bool,
}

impl ViewerWidget {
    /// 渲染查看器
    pub fn ui(&mut self, ui: &mut Ui, state: &ViewState, settings: &ViewerSettings) {
        let available_size = ui.available_size();
        let bg_color = Color32::from_rgb(
            settings.background_color.r,
            settings.background_color.g,
            settings.background_color.b,
        );

        let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // 处理双击全屏
        if response.double_clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                    !ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false)),
                ));
        }

        // 处理拖拽平移
        if response.dragged() {
            // 这里应该更新 state.offset，但需要 mutable access
            self.dragging = true;
        } else {
            self.dragging = false;
        }

        // 处理滚轮缩放
        if response.hovered() && !self.dragging {
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 && settings.smooth_scroll {
                // 这里应该更新 state.scale
            }
        }

        // 渲染图像或占位符
        if let Some(ref image) = state.current_image {
            self.render_image(ui, image, state, rect, &response, settings);
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
    ) {
        // 这里将渲染实际的图像纹理
        // 由于 Core 层不持有纹理，需要在 Adapter 层管理
        // 这里显示文件名作为占位

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

        // 缩放指示
        if state.user_zoomed {
            let zoom_text = format!("{:.0}%", state.scale.percentage());
            ui.painter().text(
                center + Vec2::new(0.0, 30.0),
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
