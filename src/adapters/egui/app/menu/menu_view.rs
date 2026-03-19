use super::style::MenuStyle;
use super::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::adapters::egui::app::types::SlideshowEndBehavior;
use crate::core::domain::{Language, Theme, ViewMode};
use egui::{Context, RichText};

impl EguiApp {
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

        let slideshow_label = if self.slideshow.playing {
            get_text("slideshow_pause", language)
        } else {
            get_text("slideshow_play", language)
        };
        if self.render_menu_item(ui, "▶", slideshow_label, Some("S"), style, true) {
            self.toggle_slideshow();
            clicked = true;
        }

        ui.label(
            RichText::new(get_text("slideshow_interval", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        for interval in [1_u64, 2, 3, 5] {
            let label = format!("{}s", interval);
            if self.render_menu_item(
                ui,
                "⏱",
                &label,
                None,
                style,
                self.slideshow.interval_seconds != interval,
            ) {
                self.set_slideshow_interval(interval);
                clicked = true;
            }
        }

        ui.label(
            RichText::new(get_text("slideshow_end_behavior", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "🔁",
            get_text("slideshow_end_loop", language),
            None,
            style,
            self.slideshow.end_behavior != SlideshowEndBehavior::Loop,
        ) {
            self.set_slideshow_end_behavior(SlideshowEndBehavior::Loop);
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "⏹",
            get_text("slideshow_end_stop", language),
            None,
            style,
            self.slideshow.end_behavior != SlideshowEndBehavior::Stop,
        ) {
            self.set_slideshow_end_behavior(SlideshowEndBehavior::Stop);
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
}
