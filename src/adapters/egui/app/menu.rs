use super::types::EguiApp;
use crate::core::domain::{NavigationDirection, ViewMode};
use crate::is_chinese_supported;
use egui::{Color32, Context, CornerRadius, RichText, Stroke, Vec2};

struct MenuStyle {
    bg_color: Color32,
    hover_bg: Color32,
    active_bg: Color32,
    text_color: Color32,
    shortcut_color: Color32,
    icon_color: Color32,
    corner_radius: u8,
    item_height: f32,
    menu_width: f32,
}

impl MenuStyle {
    fn new(ctx: &Context) -> Self {
        let is_dark = ctx.style().visuals.dark_mode;

        if is_dark {
            Self {
                bg_color: Color32::from_rgb(45, 45, 48),
                hover_bg: Color32::from_rgb(60, 60, 65),
                active_bg: Color32::from_rgb(0, 122, 204),
                text_color: Color32::from_rgb(240, 240, 240),
                shortcut_color: Color32::from_rgb(160, 160, 160),
                icon_color: Color32::from_rgb(200, 200, 200),
                corner_radius: 6,
                item_height: 28.0,
                menu_width: 220.0,
            }
        } else {
            Self {
                bg_color: Color32::from_rgb(250, 250, 250),
                hover_bg: Color32::from_rgb(230, 240, 250),
                active_bg: Color32::from_rgb(0, 122, 204),
                text_color: Color32::from_rgb(30, 30, 30),
                shortcut_color: Color32::from_rgb(120, 120, 120),
                icon_color: Color32::from_rgb(80, 80, 80),
                corner_radius: 6,
                item_height: 28.0,
                menu_width: 220.0,
            }
        }
    }
}

impl EguiApp {
    pub(crate) fn render_menu_bar(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            return;
        }

        let style = MenuStyle::new(ctx);

        egui::TopBottomPanel::top("menu_bar")
            .exact_height(40.0)
            .show(ctx, |ui| {
                self.setup_modern_menu_style(ui, &style);
                self.render_modern_menu_buttons(ui, ctx, &style);
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

    fn render_modern_menu_buttons(&mut self, ui: &mut egui::Ui, ctx: &Context, style: &MenuStyle) {
        ui.horizontal_centered(|ui| {
            ui.add_space(8.0);

            let open_menu_id = ui.id().with("open_menu");
            let mut open_menu: Option<usize> = ui.ctx().data(|d| d.get_temp(open_menu_id));

            let menus = if is_chinese_supported() {
            [("文件", "📁"), ("视图", "👁"), ("图片", "🖼"), ("帮助", "❓")]
        } else {
            [("File", "📁"), ("View", "👁"), ("Image", "🖼"), ("Help", "❓")]
        };

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
                self.render_modern_popup_menu(ui, ctx, idx, &responses, open_menu_id, style);
            }

            ui.add_space(8.0);
        });
    }

    fn render_modern_popup_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        idx: usize,
        responses: &[egui::Response],
        open_menu_id: egui::Id,
        style: &MenuStyle,
    ) {
        if let Some(button) = responses.get(idx) {
            let popup_id = ui.id().with(format!("popup_{}", idx));
            let anchor = egui::PopupAnchor::from(button.rect);
            let layer_id = egui::LayerId::new(egui::Order::Foreground, popup_id.with("layer"));

            let mut should_close = false;

            egui::Popup::new(popup_id, ui.ctx().clone(), anchor, layer_id)
                .kind(egui::PopupKind::Menu)
                .show(|ui| {
                    ui.style_mut().visuals.panel_fill = style.bg_color;
                    ui.style_mut().visuals.window_fill = style.bg_color;
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = style.bg_color;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = style.hover_bg;
                    ui.style_mut().visuals.widgets.active.weak_bg_fill = style.active_bg;
                    ui.style_mut().spacing.menu_margin = egui::Margin::same(6);

                    ui.set_min_width(style.menu_width);
                    ui.set_max_width(style.menu_width);

                    ui.add_space(4.0);

                    let clicked = match idx {
                        0 => self.render_modern_file_menu(ui, ctx, style),
                        1 => self.render_modern_view_menu(ui, ctx, style),
                        2 => self.render_modern_image_menu(ui, ctx, style),
                        3 => self.render_modern_help_menu(ui, ctx, style),
                        _ => false,
                    };

                    if clicked {
                        should_close = true;
                    }

                    ui.add_space(4.0);
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
                    d.remove_temp::<usize>(open_menu_id);
                });
            }
        }
    }

    fn render_menu_item(
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
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(available_width, style.item_height),
                egui::Sense::click(),
            );

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

            ui.painter().text(
                egui::pos2(left_x, center_y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(14.0),
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

    fn render_menu_separator(&self, ui: &mut egui::Ui, _style: &MenuStyle) {
        ui.add_space(6.0);
        ui.add(egui::Separator::default().spacing(0.0));
        ui.add_space(6.0);
    }

    fn render_modern_file_menu(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &Context,
        style: &MenuStyle,
    ) -> bool {
        let mut clicked = false;

        ui.label(RichText::new("常用").size(11.0).color(style.shortcut_color));
        ui.add_space(4.0);

        if self.render_menu_item(ui, "📂", if is_chinese_supported() { "打开..." } else { "Open..." }, Some("Ctrl+O"), style, true) {
            self.handle_open_dialog();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(RichText::new("操作").size(11.0).color(style.shortcut_color));
        ui.add_space(4.0);

        let quit_shortcut = if cfg!(target_os = "macos") { "Cmd+Q" } else { "Alt+F4" };
        if self.render_menu_item(ui, "❌", if is_chinese_supported() { "退出" } else { "Exit" }, Some(quit_shortcut), style, true) {
            _ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            clicked = true;
        }

        clicked
    }

    fn render_modern_view_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        style: &MenuStyle,
    ) -> bool {
        let mut clicked = false;

        ui.label(
            RichText::new(if is_chinese_supported() { "视图模式" } else { "View Mode" })
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(ui, "🖼", if is_chinese_supported() { "图库视图" } else { "Gallery" }, Some("G"), style, true) {
            let _ = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Gallery);
            clicked = true;
        }

        if self.render_menu_item(ui, "🔍", "查看器", Some("G"), style, true) {
            let _ = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Viewer);
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(RichText::new("显示").size(11.0).color(style.shortcut_color));
        ui.add_space(4.0);

        if self.render_menu_item(ui, "⛶", if is_chinese_supported() { "全屏切换" } else { "Fullscreen" }, Some("F11"), style, true) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
            clicked = true;
        }

        clicked
    }

    fn render_modern_image_menu(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &Context,
        style: &MenuStyle,
    ) -> bool {
        let mut clicked = false;

        ui.label(RichText::new("导航").size(11.0).color(style.shortcut_color));
        ui.add_space(4.0);

        if self.render_menu_item(ui, "⬅", "上一张", Some("←"), style, true) {
            self.navigate_and_open(_ctx, NavigationDirection::Previous);
            clicked = true;
        }

        if self.render_menu_item(ui, "➡", "下一张", Some("→"), style, true) {
            self.navigate_and_open(_ctx, NavigationDirection::Next);
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(RichText::new(if is_chinese_supported() { "缩放" } else { "Zoom" }).size(11.0).color(style.shortcut_color));
        ui.add_space(4.0);

        if self.render_menu_item(ui, "🔍+", "放大", Some("Ctrl++"), style, true) {
            self.handle_zoom_in();
            clicked = true;
        }

        if self.render_menu_item(ui, "🔍-", "缩小", Some("Ctrl+-"), style, true) {
            self.handle_zoom_out();
            clicked = true;
        }

        if self.render_menu_item(ui, "📐", "适应窗口", Some("Ctrl+0"), style, true) {
            self.handle_fit_to_window(_ctx);
            clicked = true;
        }

        if self.render_menu_item(ui, "🔢", "原始尺寸", Some("Ctrl+1"), style, true) {
            self.handle_reset_zoom();
            clicked = true;
        }

        clicked
    }

    fn render_modern_help_menu(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &Context,
        style: &MenuStyle,
    ) -> bool {
        let mut clicked = false;

        if self.render_menu_item(ui, "⌨", "快捷键", Some("?"), style, true) {
            self.shortcuts_help_panel.toggle();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        if self.render_menu_item(ui, "ℹ", if is_chinese_supported() { "关于 Image-Viewer" } else { "About" }, None, style, true) {
            self.show_about = true;
            clicked = true;
        }

        clicked
    }
}
