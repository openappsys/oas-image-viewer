//! 顶部菜单与下拉交互渲染逻辑

use super::types::EguiApp;
use super::types::UiTaskStatus;
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::Language;
use egui::{Color32, Context, CornerRadius, RichText, Stroke, Vec2};
use std::sync::mpsc;
use std::thread;

mod integration;
mod menu_file;
mod menu_help;
mod menu_image;
mod menu_specs;
mod menu_view;
mod popup;
mod sections;
mod style;

use integration::{integration_success_text, perform_integration_action, IntegrationAction};
use menu_specs::popup_item_specs;
use popup::PopupMenuParams;
use style::MenuStyle;

impl EguiApp {
    fn calculate_popup_width(
        &self,
        ui: &egui::Ui,
        idx: usize,
        style: &MenuStyle,
        language: Language,
    ) -> f32 {
        let icon_and_left_padding = 12.0 + 26.0;
        let right_padding = 12.0;
        let shortcut_gap = 16.0;
        let mut required = style.menu_min_width;

        for (label, shortcut) in popup_item_specs(idx, language) {
            let label_width = ui
                .painter()
                .layout_no_wrap(
                    label,
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                )
                .size()
                .x;
            let shortcut_width = shortcut
                .map(|s| {
                    ui.painter()
                        .layout_no_wrap(s, egui::FontId::monospace(12.0), egui::Color32::WHITE)
                        .size()
                        .x
                })
                .unwrap_or(0.0);
            let row_required = icon_and_left_padding
                + label_width
                + right_padding
                + if shortcut_width > 0.0 {
                    shortcut_width + shortcut_gap
                } else {
                    0.0
                };
            required = required.max(row_required);
        }

        required.clamp(style.menu_min_width, style.menu_max_width)
    }

    fn run_integration_action_async(&mut self, action: IntegrationAction, language: Language) {
        if self.integration_task_running {
            self.task_state.status = UiTaskStatus::Cancelled;
            self.task_state.message = Some(get_text("integration_processing", language).to_string());
            return;
        }

        self.integration_task_running = true;
        self.task_state.status = UiTaskStatus::Running;
        self.task_state.message = Some(get_text("integration_processing", language).to_string());
        self.last_context_menu_result = self.task_state.message.clone();

        let (tx, rx) = mpsc::channel::<String>();
        self.integration_task_receiver = Some(rx);

        thread::spawn(move || {
            let result = perform_integration_action(action, language);

            let message = match result {
                Ok(()) => integration_success_text(action, language),
                Err(e) => format!("{}: {}", get_text("operation_failed", language), e),
            };

            if tx.send(message).is_err() {
                tracing::debug!("集成任务结果发送失败：接收端已关闭");
            }
        });
    }

    pub(crate) fn render_menu_bar(
        &mut self,
        ctx: &Context,
        _frame: &mut eframe::Frame,
        language: Language,
    ) {
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            return;
        }

        let style = MenuStyle::new(ctx);

        egui::TopBottomPanel::top("menu_bar")
            .exact_height(40.0)
            .show(ctx, |ui| {
                self.setup_modern_menu_style(ui, &style);
                self.render_modern_menu_buttons(ui, ctx, &style, language);
            });
    }

    fn setup_modern_menu_style(&self, ui: &mut egui::Ui, style: &MenuStyle) {
        let visual = ui.visuals_mut();

        visual.panel_fill = style.bg_color;
        visual.widgets.inactive.weak_bg_fill = style.bg_color;
        visual.widgets.inactive.bg_fill = style.bg_color;
        visual.widgets.hovered.weak_bg_fill = style.hover_bg;
        visual.widgets.hovered.bg_fill = style.hover_bg;
        visual.widgets.active.weak_bg_fill = style.active_bg;
        visual.widgets.active.bg_fill = style.active_bg;

        visual.widgets.inactive.corner_radius = CornerRadius::same(style.corner_radius);
        visual.widgets.hovered.corner_radius = CornerRadius::same(style.corner_radius);
        visual.widgets.active.corner_radius = CornerRadius::same(style.corner_radius);

        visual.override_text_color = Some(style.text_color);

        ui.spacing_mut().button_padding = Vec2::new(16.0, 8.0);
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
    }

    fn render_modern_menu_buttons(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        style: &MenuStyle,
        language: Language,
    ) {
        ui.horizontal_centered(|ui| {
            ui.add_space(8.0);

            let open_menu_id = ui.id().with("open_menu");
            let mut open_menu: Option<usize> = ui.ctx().data(|d| d.get_temp(open_menu_id));

            let menus = [
                (get_text("menu_file", language), "📁"),
                (get_text("menu_view", language), "👁"),
                (get_text("menu_image", language), "🖼"),
                (get_text("menu_help", language), "❓"),
            ];

            let mut responses: Vec<egui::Response> = Vec::new();

            for (idx, (title, icon)) in menus.iter().enumerate() {
                let is_open = open_menu == Some(idx);

                let button_text = format!("{} {}", icon, title);
                let button = if is_open {
                    egui::Button::new(
                        RichText::new(&button_text)
                            .color(style.text_color)
                            .size(14.0),
                    )
                    .fill(style.active_bg)
                    .corner_radius(CornerRadius::same(style.corner_radius))
                    .stroke(Stroke::new(1.0, style.active_bg))
                } else {
                    egui::Button::new(
                        RichText::new(&button_text)
                            .color(style.text_color)
                            .size(14.0),
                    )
                    .fill(Color32::TRANSPARENT)
                    .corner_radius(CornerRadius::same(style.corner_radius))
                };

                let response = ui.add(button);
                responses.push(response);
            }

            let mut new_open = open_menu;
            for (idx, response) in responses.iter().enumerate() {
                if response.clicked() {
                    new_open = if open_menu == Some(idx) {
                        None
                    } else {
                        Some(idx)
                    };
                }
                if response.hovered() && open_menu.is_some() && open_menu != Some(idx) {
                    new_open = Some(idx);
                }
            }

            if new_open != open_menu {
                open_menu = new_open;
                ui.ctx().data_mut(|d| {
                    if let Some(idx) = open_menu {
                        d.insert_temp(open_menu_id, idx);
                    } else {
                        d.remove_temp::<usize>(open_menu_id);
                    }
                });
            }

            if let Some(idx) = open_menu {
                self.render_modern_popup_menu(
                    ui,
                    PopupMenuParams {
                        ctx,
                        idx,
                        responses: &responses,
                        open_menu_id,
                        style,
                        language,
                    },
                );
            }

            ui.add_space(8.0);
        });
    }

}
