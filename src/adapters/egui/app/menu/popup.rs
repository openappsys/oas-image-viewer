use super::style::MenuStyle;
use super::EguiApp;
use crate::core::domain::Language;
use egui::Context;

pub(super) struct PopupMenuParams<'a> {
    pub(super) ctx: &'a Context,
    pub(super) idx: usize,
    pub(super) responses: &'a [egui::Response],
    pub(super) open_menu_id: egui::Id,
    pub(super) style: &'a MenuStyle,
    pub(super) language: Language,
}

impl EguiApp {
    pub(super) fn render_modern_popup_menu(
        &mut self,
        ui: &mut egui::Ui,
        params: PopupMenuParams<'_>,
    ) {
        if let Some(button) = params.responses.get(params.idx) {
            let popup_id = ui.id().with(format!("popup_{}", params.idx));
            let anchor = egui::PopupAnchor::from(button.rect);
            let layer_id = egui::LayerId::new(egui::Order::Foreground, popup_id.with("layer"));

            let mut should_close = false;

            egui::Popup::new(popup_id, ui.ctx().clone(), anchor, layer_id)
                .kind(egui::PopupKind::Menu)
                .show(|ui| {
                    ui.style_mut().visuals.panel_fill = params.style.bg_color;
                    ui.style_mut().visuals.window_fill = params.style.bg_color;
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = params.style.bg_color;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = params.style.hover_bg;
                    ui.style_mut().visuals.widgets.active.weak_bg_fill = params.style.active_bg;
                    ui.style_mut().spacing.menu_margin =
                        egui::Margin::same(params.style.layout.popup_margin);

                    let popup_width =
                        self.calculate_popup_width(ui, params.idx, params.style, params.language);
                    ui.set_min_width(popup_width);
                    ui.set_max_width(popup_width);
                    let viewport_height = params.ctx.viewport_rect().height();
                    let max_popup_height =
                        (viewport_height - params.style.layout.popup_height_margin).clamp(
                            params.style.layout.popup_min_height,
                            params.style.layout.popup_max_height,
                        );

                    ui.add_space(params.style.layout.popup_inner_vertical_padding);

                    let mut clicked = false;
                    egui::ScrollArea::vertical()
                        .max_height(max_popup_height)
                        .show(ui, |ui| {
                            clicked = match params.idx {
                                0 => self.render_modern_file_menu(
                                    ui,
                                    params.ctx,
                                    params.style,
                                    params.language,
                                ),
                                1 => self.render_modern_view_menu(
                                    ui,
                                    params.ctx,
                                    params.style,
                                    params.language,
                                ),
                                2 => self.render_modern_image_menu(
                                    ui,
                                    params.ctx,
                                    params.style,
                                    params.language,
                                ),
                                3 => self.render_modern_help_menu(
                                    ui,
                                    params.ctx,
                                    params.style,
                                    params.language,
                                ),
                                _ => false,
                            };
                        });

                    if clicked {
                        should_close = true;
                    }

                    ui.add_space(params.style.layout.popup_inner_vertical_padding);
                });

            if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                should_close = true;
            }

            if ui.ctx().input(|i| i.pointer.any_click()) {
                let pointer_pos = ui.ctx().input(|i| i.pointer.interact_pos());
                let menu_bar_rect = ui.min_rect();
                let click_outside = if let Some(pos) = pointer_pos {
                    let in_menu_bar = menu_bar_rect.contains(pos);
                    let popup_response = ui.ctx().read_response(popup_id);
                    let in_popup = popup_response.is_some_and(|r| r.rect.contains(pos));
                    !in_menu_bar && !in_popup
                } else {
                    true
                };
                if click_outside {
                    should_close = true;
                }
            }

            if should_close {
                ui.ctx().data_mut(|d| {
                    d.remove_temp::<usize>(params.open_menu_id);
                });
            }
        }
    }
}
