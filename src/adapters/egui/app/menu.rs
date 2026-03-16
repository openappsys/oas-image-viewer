use super::types::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::adapters::platform::SystemIntegration;
use crate::core::domain::{Language, NavigationDirection, Theme, ViewMode};
use egui::{Color32, Context, CornerRadius, RichText, Stroke, Vec2};
use std::sync::mpsc;
use std::thread;

struct MenuStyle {
    bg_color: Color32,
    hover_bg: Color32,
    active_bg: Color32,
    text_color: Color32,
    shortcut_color: Color32,
    icon_color: Color32,
    corner_radius: u8,
    item_height: f32,
    menu_min_width: f32,
    menu_max_width: f32,
}

#[derive(Clone, Copy)]
enum IntegrationAction {
    SetDefault,
    UnsetDefault,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    AddContextMenu,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    RemoveContextMenu,
    #[cfg(target_os = "macos")]
    RefreshOpenWith,
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
                menu_min_width: 220.0,
                menu_max_width: 320.0,
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
                menu_min_width: 220.0,
                menu_max_width: 320.0,
            }
        }
    }
}

impl EguiApp {
    fn popup_item_specs(&self, idx: usize, language: Language) -> Vec<(String, Option<String>)> {
        match idx {
            0 => vec![
                (
                    get_text("open", language).to_string(),
                    Some("Ctrl+O".to_string()),
                ),
                (
                    get_text("exit", language).to_string(),
                    Some(if cfg!(target_os = "macos") {
                        "Cmd+Q".to_string()
                    } else {
                        "Alt+F4".to_string()
                    }),
                ),
            ],
            1 => vec![
                (
                    get_text("gallery", language).to_string(),
                    Some("G".to_string()),
                ),
                (
                    get_text("viewer", language).to_string(),
                    Some("V".to_string()),
                ),
                (
                    get_text("fullscreen", language).to_string(),
                    Some("F11".to_string()),
                ),
                (get_text("language_chinese", language).to_string(), None),
                (get_text("language_english", language).to_string(), None),
                (get_text("theme_system", language).to_string(), None),
                (get_text("theme_light", language).to_string(), None),
                (get_text("theme_dark", language).to_string(), None),
                (get_text("theme_oled", language).to_string(), None),
            ],
            2 => vec![
                (
                    get_text("previous", language).to_string(),
                    Some("←".to_string()),
                ),
                (
                    get_text("next", language).to_string(),
                    Some("→".to_string()),
                ),
                (
                    get_text("zoom_in", language).to_string(),
                    Some("+".to_string()),
                ),
                (
                    get_text("zoom_out", language).to_string(),
                    Some("-".to_string()),
                ),
                (
                    get_text("fit_to_window", language).to_string(),
                    Some("F".to_string()),
                ),
                (
                    get_text("original_size", language).to_string(),
                    Some("1".to_string()),
                ),
            ],
            3 => {
                let mut items = vec![
                    (
                        get_text("shortcuts_title", language).to_string(),
                        Some("?".to_string()),
                    ),
                    (get_text("set_default_app", language).to_string(), None),
                    (get_text("unset_default_app", language).to_string(), None),
                    (get_text("about_app", language).to_string(), None),
                ];
                #[cfg(target_os = "windows")]
                {
                    items.push((get_text("add_context_menu", language).to_string(), None));
                    items.push((get_text("remove_context_menu", language).to_string(), None));
                }
                #[cfg(target_os = "linux")]
                {
                    items.push((get_text("add_context_menu", language).to_string(), None));
                    items.push((get_text("remove_context_menu", language).to_string(), None));
                }
                #[cfg(target_os = "macos")]
                {
                    items.push((get_text("refresh_open_with", language).to_string(), None));
                }
                items
            }
            _ => Vec::new(),
        }
    }

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

        for (label, shortcut) in self.popup_item_specs(idx, language) {
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

    fn integration_success_text(action: IntegrationAction, language: Language) -> String {
        match action {
            IntegrationAction::SetDefault => get_text("default_app_set", language).to_string(),
            IntegrationAction::UnsetDefault => get_text("default_app_unset", language).to_string(),
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            IntegrationAction::AddContextMenu => {
                get_text("context_menu_added", language).to_string()
            }
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            IntegrationAction::RemoveContextMenu => {
                get_text("context_menu_removed", language).to_string()
            }
            #[cfg(target_os = "macos")]
            IntegrationAction::RefreshOpenWith => {
                get_text("open_with_refreshed", language).to_string()
            }
        }
    }

    fn run_integration_action_async(&mut self, action: IntegrationAction, language: Language) {
        if self.integration_task_running {
            return;
        }

        self.integration_task_running = true;
        self.last_context_menu_result =
            Some(get_text("integration_processing", language).to_string());

        let (tx, rx) = mpsc::channel::<String>();
        self.integration_task_receiver = Some(rx);

        thread::spawn(move || {
            let integration = crate::adapters::platform::PlatformIntegration::new();
            let result = match action {
                IntegrationAction::SetDefault => integration.set_as_default(language),
                IntegrationAction::UnsetDefault => integration.unset_default(language),
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                IntegrationAction::AddContextMenu => integration.add_context_menu(language),
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                IntegrationAction::RemoveContextMenu => integration.remove_context_menu(language),
                #[cfg(target_os = "macos")]
                IntegrationAction::RefreshOpenWith => {
                    integration.refresh_open_with_registration(language)
                }
            };

            let message = match result {
                Ok(()) => Self::integration_success_text(action, language),
                Err(e) => format!("{}: {}", get_text("operation_failed", language), e),
            };

            let _ = tx.send(message);
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
                    ctx,
                    idx,
                    &responses,
                    open_menu_id,
                    style,
                    language,
                );
            }

            ui.add_space(8.0);
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_modern_popup_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        idx: usize,
        responses: &[egui::Response],
        open_menu_id: egui::Id,
        style: &MenuStyle,
        language: Language,
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

                    let popup_width = self.calculate_popup_width(ui, idx, style, language);
                    ui.set_min_width(popup_width);
                    ui.set_max_width(popup_width);

                    ui.add_space(4.0);

                    let clicked = match idx {
                        0 => self.render_modern_file_menu(ui, ctx, style, language),
                        1 => self.render_modern_view_menu(ui, ctx, style, language),
                        2 => self.render_modern_image_menu(ui, ctx, style, language),
                        3 => self.render_modern_help_menu(ui, ctx, style, language),
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
            let (rect, response) = ui
                .allocate_exact_size(Vec2::new(available_width, row_height), egui::Sense::click());

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
        language: Language,
    ) -> bool {
        let mut clicked = false;

        ui.label(
            RichText::new(get_text("common", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "📂",
            get_text("open", language),
            Some("Ctrl+O"),
            style,
            true,
        ) {
            self.handle_open_dialog();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("actions", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let quit_shortcut = if cfg!(target_os = "macos") {
            "Cmd+Q"
        } else {
            "Alt+F4"
        };
        if self.render_menu_item(
            ui,
            "❌",
            get_text("exit", language),
            Some(quit_shortcut),
            style,
            true,
        ) {
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
        language: Language,
    ) -> bool {
        let mut clicked = false;

        ui.label(
            RichText::new(get_text("view_mode", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "🖼",
            get_text("gallery", language),
            Some("G"),
            style,
            true,
        ) {
            if let Err(e) = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Gallery)
            {
                tracing::error!(error = %e, "切换到图库视图失败");
            }
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "🔍",
            get_text("viewer", language),
            Some("G"),
            style,
            true,
        ) {
            if let Err(e) = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Viewer)
            {
                tracing::error!(error = %e, "切换到查看器视图失败");
            }
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("display", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "⛶",
            get_text("fullscreen", language),
            Some("F11"),
            style,
            true,
        ) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        // 语言切换子菜单
        ui.label(
            RichText::new(get_text("language", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        // 中文选项
        let chinese_label = get_text("language_chinese", language).to_string();
        if self.render_menu_item(
            ui,
            "🇨🇳",
            &chinese_label,
            None,
            style,
            language != Language::Chinese,
        ) {
            if let Err(e) = self.service.update_state(|s| {
                s.config.language = Language::Chinese;
                // 同时更新中文字体支持标志
                crate::set_chinese_supported(true);
            }) {
                tracing::error!(error = %e, "切换语言失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
            clicked = true;
        }

        // 英文选项
        let english_label = get_text("language_english", language).to_string();
        if self.render_menu_item(
            ui,
            "🇺🇸",
            &english_label,
            None,
            style,
            language != Language::English,
        ) {
            if let Err(e) = self.service.update_state(|s| {
                s.config.language = Language::English;
                // 同时更新中文字体支持标志
                crate::set_chinese_supported(false);
            }) {
                tracing::error!(error = %e, "切换语言失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        // 主题切换子菜单
        ui.label(
            RichText::new(get_text("theme", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        // 获取当前主题设置
        let current_theme = self
            .service
            .get_state()
            .map(|s| s.config.theme)
            .unwrap_or_default();

        // 跟随系统选项
        let system_label = get_text("theme_system", language).to_string();
        if self.render_menu_item(
            ui,
            "🖥",
            &system_label,
            None,
            style,
            current_theme != Theme::System,
        ) {
            if let Err(e) = self.service.update_state(|s| {
                s.config.theme = Theme::System;
            }) {
                tracing::error!(error = %e, "切换主题失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
            clicked = true;
        }

        // 浅色选项
        let light_label = get_text("theme_light", language).to_string();
        if self.render_menu_item(
            ui,
            "☀",
            &light_label,
            None,
            style,
            current_theme != Theme::Light,
        ) {
            if let Err(e) = self.service.update_state(|s| {
                s.config.theme = Theme::Light;
            }) {
                tracing::error!(error = %e, "切换主题失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
            clicked = true;
        }

        // 深色选项
        let dark_label = get_text("theme_dark", language).to_string();
        if self.render_menu_item(
            ui,
            "🌙",
            &dark_label,
            None,
            style,
            current_theme != Theme::Dark,
        ) {
            if let Err(e) = self.service.update_state(|s| {
                s.config.theme = Theme::Dark;
            }) {
                tracing::error!(error = %e, "切换主题失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
            clicked = true;
        }

        // 纯黑选项（OLED）
        let oled_label = get_text("theme_oled", language).to_string();
        if self.render_menu_item(
            ui,
            "⬛",
            &oled_label,
            None,
            style,
            current_theme != Theme::OLED,
        ) {
            if let Err(e) = self.service.update_state(|s| {
                s.config.theme = Theme::OLED;
            }) {
                tracing::error!(error = %e, "切换主题失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
            clicked = true;
        }

        clicked
    }

    fn render_modern_image_menu(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &Context,
        style: &MenuStyle,
        language: Language,
    ) -> bool {
        let mut clicked = false;

        ui.label(
            RichText::new(get_text("navigation", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "⬅",
            get_text("previous", language),
            Some("←"),
            style,
            true,
        ) {
            self.navigate_and_open(_ctx, NavigationDirection::Previous);
            clicked = true;
        }

        if self.render_menu_item(ui, "➡", get_text("next", language), Some("→"), style, true) {
            self.navigate_and_open(_ctx, NavigationDirection::Next);
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("zoom", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "🔍+",
            get_text("zoom_in", language),
            Some("Ctrl++"),
            style,
            true,
        ) {
            self.handle_zoom_in();
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "🔍-",
            get_text("zoom_out", language),
            Some("Ctrl+-"),
            style,
            true,
        ) {
            self.handle_zoom_out();
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "📐",
            get_text("fit_to_window", language),
            Some("Ctrl+0"),
            style,
            true,
        ) {
            self.handle_fit_to_window(_ctx);
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "🔢",
            get_text("original_size", language),
            Some("Ctrl+1"),
            style,
            true,
        ) {
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
        language: Language,
    ) -> bool {
        let mut clicked = false;

        if self.render_menu_item(
            ui,
            "⌨",
            get_text("shortcuts_title", language),
            Some("?"),
            style,
            true,
        ) {
            self.shortcuts_help_panel.toggle();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        // 系统集成子菜单
        ui.label(
            RichText::new(get_text("system_integration", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        // 使用新的 platform 模块获取集成实例
        let integration = crate::adapters::platform::PlatformIntegration::new();
        let integration_enabled = !self.integration_task_running;

        // 设置为默认图片查看器（带勾选状态）
        let is_default = integration.is_default();
        let default_label = if is_default {
            format!("✓ {}", get_text("set_default_app", language))
        } else {
            get_text("set_default_app", language).to_string()
        };

        if self.render_menu_item(
            ui,
            if is_default { "✓" } else { "⭐" },
            &default_label,
            None,
            style,
            integration_enabled,
        ) {
            self.run_integration_action_async(IntegrationAction::SetDefault, language);
            clicked = true;
        }

        let unset_label = get_text("unset_default_app", language).to_string();
        if self.render_menu_item(ui, "↺", &unset_label, None, style, integration_enabled) {
            self.run_integration_action_async(IntegrationAction::UnsetDefault, language);
            clicked = true;
        }

        // Windows 平台：添加/移除右键菜单
        #[cfg(target_os = "windows")]
        {
            // 添加到右键菜单
            let add_label = get_text("add_context_menu", language).to_string();
            if self.render_menu_item(ui, "📝", &add_label, None, style, integration_enabled) {
                self.run_integration_action_async(IntegrationAction::AddContextMenu, language);
                clicked = true;
            }

            // 从右键菜单移除
            let remove_label = get_text("remove_context_menu", language).to_string();
            if self.render_menu_item(ui, "🗑", &remove_label, None, style, integration_enabled) {
                self.run_integration_action_async(IntegrationAction::RemoveContextMenu, language);
                clicked = true;
            }
        }

        #[cfg(target_os = "linux")]
        {
            let add_label = get_text("add_context_menu", language).to_string();
            if self.render_menu_item(ui, "📝", &add_label, None, style, integration_enabled) {
                self.run_integration_action_async(IntegrationAction::AddContextMenu, language);
                clicked = true;
            }

            let remove_label = get_text("remove_context_menu", language).to_string();
            if self.render_menu_item(ui, "🗑", &remove_label, None, style, integration_enabled) {
                self.run_integration_action_async(IntegrationAction::RemoveContextMenu, language);
                clicked = true;
            }
        }

        #[cfg(target_os = "macos")]
        {
            let refresh_label = get_text("refresh_open_with", language).to_string();
            if self.render_menu_item(ui, "🔄", &refresh_label, None, style, integration_enabled) {
                self.run_integration_action_async(IntegrationAction::RefreshOpenWith, language);
                clicked = true;
            }
        }

        self.render_menu_separator(ui, style);

        if self.render_menu_item(ui, "ℹ", get_text("about_app", language), None, style, true) {
            self.show_about = true;
            clicked = true;
        }

        clicked
    }
}
