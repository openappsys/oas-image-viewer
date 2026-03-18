use super::style::MenuStyle;
use super::EguiApp;
use egui::{Color32, CornerRadius, Vec2};

impl EguiApp {
    pub(super) fn render_menu_item(
        &mut self,
        ui: &mut egui::Ui,
        icon: &str,
        label: &str,
        shortcut: Option<&str>,
        style: &MenuStyle,
        enabled: bool,
    ) -> bool {
        let mut clicked = false;

        ui.add_enabled_ui(enabled, |ui| {
            let available_width = ui.available_width();
            let text_color = if enabled {
                style.text_color
            } else {
                style.shortcut_color
            };
            let icon_color = if enabled {
                style.icon_color
            } else {
                style.shortcut_color
            };
            let shortcut_width = shortcut
                .map(|text| {
                    ui.painter()
                        .layout_no_wrap(
                            text.to_string(),
                            egui::FontId::monospace(12.0),
                            style.shortcut_color,
                        )
                        .size()
                        .x
                })
                .unwrap_or(0.0);
            let label_left_padding = 12.0 + 26.0;
            let label_right_padding = 12.0
                + if shortcut_width > 0.0 {
                    shortcut_width + 16.0
                } else {
                    0.0
                };
            let label_max_width =
                (available_width - label_left_padding - label_right_padding).max(80.0);
            let label_galley = ui.painter().layout(
                label.to_string(),
                egui::FontId::proportional(14.0),
                text_color,
                label_max_width,
            );
            let row_height = (label_galley.size().y + 10.0).max(style.item_height);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(available_width, row_height), egui::Sense::click());

            let is_hovered = response.hovered();
            let is_active = response.is_pointer_button_down_on();

            let bg_color = if is_active {
                style.active_bg
            } else if is_hovered {
                style.hover_bg
            } else {
                Color32::TRANSPARENT
            };

            if bg_color != Color32::TRANSPARENT {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(style.corner_radius), bg_color);
            }

            let mut left_x = rect.left() + 12.0;
            let center_y = rect.center().y;

            ui.painter().text(
                egui::pos2(left_x, center_y),
                egui::Align2::LEFT_CENTER,
                icon,
                egui::FontId::proportional(16.0),
                icon_color,
            );
            left_x += 26.0;

            ui.painter().galley(
                egui::pos2(left_x, center_y - label_galley.size().y / 2.0),
                label_galley,
                text_color,
            );

            if let Some(shortcut_text) = shortcut {
                let shortcut_x = rect.right() - 12.0;
                ui.painter().text(
                    egui::pos2(shortcut_x, center_y),
                    egui::Align2::RIGHT_CENTER,
                    shortcut_text,
                    egui::FontId::monospace(12.0),
                    style.shortcut_color,
                );
            }

            clicked = response.clicked();
        });

        clicked
    }

    pub(super) fn render_menu_separator(&self, ui: &mut egui::Ui, _style: &MenuStyle) {
        ui.add_space(6.0);
        ui.add(egui::Separator::default().spacing(0.0));
        ui.add_space(6.0);
    }
}
