//! 渲染模块

use super::types::EguiApp;
use crate::core::domain::ViewMode;
use egui::Context;

impl EguiApp {
    /// 渲染拖拽覆盖层
    pub(crate) fn render_drag_overlay(&self, ctx: &Context) {
        if !self.drag_hovering {
            return;
        }

        let screen_rect = ctx.viewport_rect();
        let text = self.get_drag_text(ctx);

        egui::Area::new(egui::Id::new("drag_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                self.draw_drag_overlay(ui, screen_rect, &text);
            });
    }

    fn get_drag_text(&self, ctx: &Context) -> String {
        super::utils::get_drag_preview_text(ctx)
            .map(|p| format!("📂 {}", p))
            .unwrap_or_else(|| "📂 释放以打开图片".to_string())
    }

    fn draw_drag_overlay(&self, ui: &mut egui::Ui, screen_rect: egui::Rect, text: &str) {
        let painter = ui.painter();

        painter.rect_filled(
            screen_rect,
            0.0,
            egui::Color32::from_rgba_premultiplied(52, 152, 219, 30),
        );

        painter.rect_stroke(
            screen_rect.shrink(2.0),
            4.0,
            egui::Stroke::new(4.0, egui::Color32::from_rgb(52, 152, 219)),
            egui::StrokeKind::Outside,
        );

        painter.rect_stroke(
            screen_rect.shrink(8.0),
            4.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 230)),
            egui::StrokeKind::Outside,
        );

        self.draw_drag_text(ui, screen_rect.center(), text);
    }

    fn draw_drag_text(&self, ui: &mut egui::Ui, center: egui::Pos2, text: &str) {
        let painter = ui.painter();
        let font = egui::FontId::proportional(20.0);

        let text_size = painter
            .layout(
                text.to_string(),
                font.clone(),
                egui::Color32::WHITE,
                f32::INFINITY,
            )
            .size();

        let pill_rect =
            egui::Rect::from_center_size(center, text_size + egui::Vec2::new(40.0, 24.0));

        painter.rect_filled(
            pill_rect,
            8.0,
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        painter.text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font,
            egui::Color32::WHITE,
        );
    }

    /// 渲染关于窗口
    pub(crate) fn render_about_window(&mut self, ctx: &Context) {
        if !self.show_about {
            return;
        }

        let mut window = egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 200.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);

        if let Some(pos) = self.about_window_pos {
            window = window.current_pos(pos);
        }

        let response = window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Image-Viewer");
                ui.add_space(10.0);
                ui.label("版本: v0.3.0");
                ui.add_space(5.0);
                ui.label("© 2026 Image-Viewer Contributors");
                ui.add_space(5.0);
                ui.label("许可证: MIT License");
                ui.add_space(20.0);
                if ui.button("关闭").clicked() {
                    self.show_about = false;
                }
            });
        });

        if let Some(inner) = response {
            self.about_window_pos = Some(inner.response.rect.left_top());
        }
    }

    /// 渲染信息面板
    pub(crate) fn render_info_panel(&mut self, ctx: &Context) {
        self.sync_info_panel_visibility();
        self.update_info_panel_content();

        let closed_by_user = self.info_panel.ui(ctx);

        if closed_by_user {
            let _ = self.service.update_state(|state| {
                state.config.viewer.show_info_panel = false;
            });
        }
    }

    fn sync_info_panel_visibility(&mut self) {
        let Ok(state) = self.service.get_state() else {
            return;
        };

        let should_show = state.config.viewer.show_info_panel
            && state.view.current_image.is_some()
            && state.view.view_mode == ViewMode::Viewer;

        if should_show != self.info_panel.is_visible() {
            if should_show {
                self.info_panel.show();
            } else {
                self.info_panel.hide();
            }
        }
    }

    fn update_info_panel_content(&mut self) {
        let Ok(state) = self.service.get_state() else {
            return;
        };

        if let Some(ref image) = state.view.current_image {
            let new_path = image.path().to_path_buf();
            if self.current_image_path.as_ref() != Some(&new_path) {
                self.current_image_path = Some(new_path.clone());
                self.info_panel.set_image_info(
                    &new_path,
                    (image.metadata().width, image.metadata().height),
                    &format!("{:?}", image.metadata().format),
                );
            }
        } else if self.current_image_path.is_some() {
            self.current_image_path = None;
            self.info_panel.clear();
        }
    }

    /// 渲染快捷键帮助
    pub(crate) fn render_shortcuts_help(&mut self, ctx: &Context) {
        self.shortcuts_help_panel.ui(ctx);
    }
}
