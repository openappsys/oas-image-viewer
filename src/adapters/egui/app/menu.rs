//! 菜单渲染模块

use super::types::EguiApp;
use crate::core::domain::{NavigationDirection, ViewMode};

use egui::Context;

impl EguiApp {
    /// 渲染菜单栏
    pub(crate) fn render_menu_bar(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            return;
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.setup_menu_style(ui);
            self.render_menu_buttons(ui, ctx);
        });
    }

    fn setup_menu_style(&self, ui: &mut egui::Ui) {
        ui.style_mut().visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(52, 152, 219);
        ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
            egui::Color32::from_rgb(100, 180, 230);
        ui.style_mut().spacing.button_padding = egui::vec2(12.0, 6.0);
    }

    fn render_menu_buttons(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            let open_menu_id = ui.id().with("open_menu");
            let mut open_menu: Option<usize> = ui.ctx().data(|d| d.get_temp(open_menu_id));

            let menus = ["文件", "视图", "图片", "帮助"];
            let mut responses: Vec<egui::Response> = Vec::new();

            for (idx, title) in menus.iter().enumerate() {
                let button = if open_menu == Some(idx) {
                    egui::Button::new(*title).selected(true)
                } else {
                    egui::Button::new(*title)
                };
                responses.push(ui.add(button));
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
                self.render_opened_menu(ui, ctx, idx, &responses, open_menu_id);
            }
        });
    }

    fn render_opened_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        idx: usize,
        responses: &[egui::Response],
        open_menu_id: egui::Id,
    ) {
        if let Some(button) = responses.get(idx) {
            let popup_id = ui.id().with(format!("popup_{}", idx));
            let anchor = egui::PopupAnchor::from(button.rect);
            let layer_id = egui::LayerId::new(egui::Order::Foreground, popup_id.with("layer"));

            let mut should_close = false;

            egui::Popup::new(popup_id, ui.ctx().clone(), anchor, layer_id)
                .kind(egui::PopupKind::Menu)
                .show(|ui| {
                    ui.set_min_width(160.0);
                    let clicked = match idx {
                        0 => self.render_file_menu(ui, ctx),
                        1 => self.render_view_menu(ui, ctx),
                        2 => self.render_image_menu(ui, ctx),
                        3 => self.render_help_menu(ui, ctx),
                        _ => false,
                    };
                    if clicked {
                        should_close = true;
                    }
                });

            if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                should_close = true;
            }

            if self.should_close_menu(ui, popup_id) {
                should_close = true;
            }

            if should_close {
                ui.ctx().data_mut(|d| {
                    d.remove_temp::<usize>(open_menu_id);
                });
            }
        }
    }

    fn should_close_menu(&self, ui: &egui::Ui, popup_id: egui::Id) -> bool {
        if !ui.ctx().input(|i| i.pointer.any_click()) {
            return false;
        }

        let pointer_pos = ui.ctx().input(|i| i.pointer.interact_pos());
        let menu_bar_rect = ui.min_rect();

        if let Some(pos) = pointer_pos {
            let in_menu_bar = menu_bar_rect.contains(pos);
            let popup_response = ui.ctx().read_response(popup_id);
            let in_popup = popup_response.is_some_and(|r| r.rect.contains(pos));
            !in_menu_bar && !in_popup
        } else {
            true
        }
    }

    fn render_file_menu(&mut self, ui: &mut egui::Ui, _ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("打开... (Ctrl+O)").clicked() {
            self.handle_open_dialog();
            clicked = true;
        }
        ui.separator();
        if ui.button("退出").clicked() {
            clicked = true;
        }
        clicked
    }

    fn render_view_menu(&mut self, ui: &mut egui::Ui, ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("图库").clicked() {
            let _ = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Gallery);
            clicked = true;
        }
        if ui.button("查看器").clicked() {
            let _ = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Viewer);
            clicked = true;
        }
        ui.separator();
        if ui.button("全屏切换 (F11)").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
            clicked = true;
        }
        clicked
    }

    fn render_image_menu(&mut self, ui: &mut egui::Ui, ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("上一张 (左箭头)").clicked() {
            self.navigate_and_open(ctx, NavigationDirection::Previous);
            clicked = true;
        }
        if ui.button("下一张 (右箭头)").clicked() {
            self.navigate_and_open(ctx, NavigationDirection::Next);
            clicked = true;
        }
        ui.separator();
        if ui.button("放大 (Ctrl++)").clicked() {
            self.handle_zoom_in();
            clicked = true;
        }
        if ui.button("缩小 (Ctrl+-)").clicked() {
            self.handle_zoom_out();
            clicked = true;
        }
        clicked
    }

    fn render_help_menu(&mut self, ui: &mut egui::Ui, _ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("快捷键帮助 (?)").clicked() {
            self.shortcuts_help_panel.toggle();
            clicked = true;
        }
        if ui.button("关于").clicked() {
            self.show_about = true;
            clicked = true;
        }
        clicked
    }
}
