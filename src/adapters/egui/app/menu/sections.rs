use super::style::MenuStyle;
use super::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::adapters::platform::SystemIntegration;
use crate::adapters::egui::shortcut_labels::{
    copy_image, copy_path, fit_to_window, open_file, open_folder, original_size, quit, zoom_in,
    zoom_out, ShortcutTextStyle,
};
use crate::core::domain::{Language, NavigationDirection, Theme, ViewMode};
use crate::core::ports::ClipboardPort;
use egui::{Color32, Context, CornerRadius, RichText, Vec2};

use super::integration::IntegrationAction;

impl EguiApp {
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

    fn render_menu_separator(&self, ui: &mut egui::Ui, _style: &MenuStyle) {
        ui.add_space(6.0);
        ui.add(egui::Separator::default().spacing(0.0));
        ui.add_space(6.0);
    }

    pub(super) fn render_modern_file_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
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
            Some(&open_file(ShortcutTextStyle::Compact)),
            style,
            true,
        ) {
            self.handle_open_dialog();
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "🗂",
            get_text("open_folder", language),
            Some(&open_folder(ShortcutTextStyle::Compact)),
            style,
            true,
        ) {
            self.handle_open_directory_dialog();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("actions", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let quit_shortcut = quit(ShortcutTextStyle::Compact);
        if self.render_menu_item(
            ui,
            "❌",
            get_text("exit", language),
            Some(&quit_shortcut),
            style,
            true,
        ) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            clicked = true;
        }

        clicked
    }

    pub(super) fn render_modern_view_menu(
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
            if let Err(e) = self.update_view_mode(ViewMode::Gallery) {
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
            if let Err(e) = self.update_view_mode(ViewMode::Viewer) {
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

        ui.label(
            RichText::new(get_text("language", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let chinese_label = get_text("language_chinese", language).to_string();
        if self.render_menu_item(
            ui,
            "🇨🇳",
            &chinese_label,
            None,
            style,
            language != Language::Chinese,
        ) {
            if let Err(e) = self.set_language_and_save(Language::Chinese) {
                tracing::error!(error = %e, "切换语言失败");
            }
            crate::set_chinese_supported(true);
            clicked = true;
        }

        let english_label = get_text("language_english", language).to_string();
        if self.render_menu_item(
            ui,
            "🇺🇸",
            &english_label,
            None,
            style,
            language != Language::English,
        ) {
            if let Err(e) = self.set_language_and_save(Language::English) {
                tracing::error!(error = %e, "切换语言失败");
            }
            crate::set_chinese_supported(false);
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("theme", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let current_theme = self.service.get_theme().unwrap_or_default();

        let system_label = get_text("theme_system", language).to_string();
        if self.render_menu_item(
            ui,
            "🖥",
            &system_label,
            None,
            style,
            current_theme != Theme::System,
        ) {
            if let Err(e) = self.set_theme_and_save(Theme::System) {
                tracing::error!(error = %e, "切换主题失败");
            }
            clicked = true;
        }

        let light_label = get_text("theme_light", language).to_string();
        if self.render_menu_item(
            ui,
            "☀",
            &light_label,
            None,
            style,
            current_theme != Theme::Light,
        ) {
            if let Err(e) = self.set_theme_and_save(Theme::Light) {
                tracing::error!(error = %e, "切换主题失败");
            }
            clicked = true;
        }

        let dark_label = get_text("theme_dark", language).to_string();
        if self.render_menu_item(
            ui,
            "🌙",
            &dark_label,
            None,
            style,
            current_theme != Theme::Dark,
        ) {
            if let Err(e) = self.set_theme_and_save(Theme::Dark) {
                tracing::error!(error = %e, "切换主题失败");
            }
            clicked = true;
        }

        let oled_label = get_text("theme_oled", language).to_string();
        if self.render_menu_item(
            ui,
            "⬛",
            &oled_label,
            None,
            style,
            current_theme != Theme::OLED,
        ) {
            if let Err(e) = self.set_theme_and_save(Theme::OLED) {
                tracing::error!(error = %e, "切换主题失败");
            }
            clicked = true;
        }

        clicked
    }

    pub(super) fn render_modern_image_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
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
            self.navigate_and_open(ctx, NavigationDirection::Previous);
            clicked = true;
        }

        if self.render_menu_item(ui, "➡", get_text("next", language), Some("→"), style, true) {
            self.navigate_and_open(ctx, NavigationDirection::Next);
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
            Some(&zoom_in(ShortcutTextStyle::Compact)),
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
            Some(&zoom_out(ShortcutTextStyle::Compact)),
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
            Some(&fit_to_window(ShortcutTextStyle::Compact)),
            style,
            true,
        ) {
            self.handle_fit_to_window(ctx);
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "🔢",
            get_text("original_size", language),
            Some(&original_size(ShortcutTextStyle::Compact)),
            style,
            true,
        ) {
            self.handle_reset_zoom();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("clipboard", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let copy_image_shortcut = copy_image(ShortcutTextStyle::Compact);
        let copy_path_shortcut = copy_path(ShortcutTextStyle::Compact);
        let path = self.service.get_current_view_image_path().ok().flatten();
        let has_image = path.is_some();

        if self.render_menu_item(
            ui,
            "📋",
            get_text("copy_image", language),
            Some(&copy_image_shortcut),
            style,
            has_image,
        ) {
            if let Some(ref path) = path {
                let result = self.copy_image_to_clipboard(path);
                let success_msg = get_text("copy_image", language).to_string();
                self.handle_copy_result(result, &success_msg, language);
                clicked = true;
            }
        }

        if self.render_menu_item(
            ui,
            "📂",
            get_text("copy_path", language),
            Some(&copy_path_shortcut),
            style,
            has_image,
        ) {
            if let Some(ref path) = path {
                let result = ClipboardPort::copy_path(&self.clipboard_manager, path);
                let success_msg = get_text("copy_path", language).to_string();
                self.handle_copy_result(result, &success_msg, language);
                clicked = true;
            }
        }

        clicked
    }

    pub(super) fn render_modern_help_menu(
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

        ui.label(
            RichText::new(get_text("system_integration", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let integration = crate::adapters::platform::PlatformIntegration::new();
        let integration_enabled = !self.integration_task_running;
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

        #[cfg(target_os = "windows")]
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
