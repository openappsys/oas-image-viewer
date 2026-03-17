//! 渲染模块

use super::types::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::{Language, ViewMode};
use egui::Context;

impl EguiApp {
    /// 渲染拖拽覆盖层
    pub(crate) fn render_drag_overlay(&self, ctx: &Context, language: Language) {
        if !self.drag_hovering {
            return;
        }

        let screen_rect = ctx.viewport_rect();
        let text = self.get_drag_text(ctx, language);

        egui::Area::new(egui::Id::new("drag_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                self.draw_drag_overlay(ui, screen_rect, &text);
            });
    }

    fn get_drag_text(&self, ctx: &Context, language: Language) -> String {
        super::utils::get_drag_preview_text(ctx, language)
            .map(|p| format!("📂 {}", p))
            .unwrap_or_else(|| format!("📂 {}", get_text("drag_hint", language)))
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
    pub(crate) fn render_about_window(&mut self, ctx: &Context, language: Language) {
        if !self.show_about {
            return;
        }

        let mut window = egui::Window::new(get_text("about_title", language))
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 200.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);

        if let Some(pos) = self.about_window_pos {
            window = window.current_pos(pos);
        }

        let version_label = format!(
            "{}: v{}",
            get_text("version", language),
            env!("CARGO_PKG_VERSION")
        );
        let license_label = format!("{}: MIT License", get_text("license", language));

        let response = window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("OAS Image Viewer");
                ui.add_space(10.0);
                ui.label(version_label);
                ui.add_space(5.0);
                ui.label("© 2026 OAS Image Viewer Contributors");
                ui.add_space(5.0);
                ui.label(license_label);
                ui.add_space(20.0);
                if ui.button(get_text("close", language)).clicked() {
                    self.show_about = false;
                }
            });
        });

        if let Some(inner) = response {
            self.about_window_pos = Some(inner.response.rect.left_top());
        }
    }

    /// 渲染信息面板
    pub(crate) fn render_info_panel(&mut self, ctx: &Context, language: Language) {
        self.sync_info_panel_visibility();
        self.update_info_panel_content();

        let closed_by_user = self.info_panel.ui(ctx, language);

        if closed_by_user {
            if let Err(e) = self.set_info_panel_visible(false) {
                tracing::error!(error = %e, "关闭信息面板失败");
            }
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
    pub(crate) fn render_shortcuts_help(&mut self, ctx: &Context, language: Language) {
        self.shortcuts_help_panel.ui(ctx, language);
    }

    /// 渲染系统集成操作结果通知
    pub(crate) fn render_integration_result(&mut self, ctx: &Context, _language: Language) {
        let Some(ref result) = self.last_context_menu_result else {
            return;
        };

        let id = egui::Id::new("integration_result_toast");

        // 检测 ESC 键提前关闭 Toast
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.last_context_menu_result = None;
            ctx.data_mut(|d| d.remove_temp::<f64>(id));
            return;
        }

        let current_time = ctx.input(|i| i.time);
        let start_time: f64 = ctx.data_mut(|d| d.get_temp(id).unwrap_or(current_time));

        // 超过 10 秒自动关闭
        if current_time - start_time > 10.0 {
            self.last_context_menu_result = None;
            ctx.data_mut(|d| d.remove_temp::<f64>(id));
            return;
        }

        // 判断成功/失败，设置背景色
        let is_error =
            result.contains("失败") || result.contains("failed") || result.contains("Error");
        let bg_color = if is_error {
            egui::Color32::from_rgb(200, 50, 50)
        } else {
            egui::Color32::from_rgb(50, 150, 80)
        };

        // 检测点击关闭 Toast（任意鼠标点击，但需显示 500ms 后）
        let elapsed = current_time - start_time;
        let pointer = ctx.input(|i| i.pointer.clone());
        let mouse_clicked = elapsed > 0.5 && pointer.any_click();

        // 使用 Frame 创建带背景的面板，内部用 label 显示文字
        // 位置锚定在屏幕底部中央
        egui::Area::new(id)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(20, 12))
                    .show(ui, |ui| {
                        ui.set_max_width(400.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(result)
                                    .color(egui::Color32::WHITE)
                                    .font(egui::FontId::proportional(14.0)),
                            );
                        });
                    });
            });

        // 如果点击了鼠标，立即关闭 Toast
        if mouse_clicked {
            self.last_context_menu_result = None;
            ctx.data_mut(|d| d.remove_temp::<f64>(id));
            return;
        }

        ctx.data_mut(|d| d.insert_temp(id, start_time));
    }
}
