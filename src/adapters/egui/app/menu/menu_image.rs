use super::style::MenuStyle;
use super::EguiApp;
use crate::adapters::egui::app::menu::menu_specs::{
    shortcut_copy_image, shortcut_copy_path, shortcut_fit_to_window, shortcut_flip_horizontal,
    shortcut_flip_vertical, shortcut_original_size, shortcut_rotate_clockwise,
    shortcut_rotate_counterclockwise, shortcut_zoom_in, shortcut_zoom_out,
};
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::{Language, NavigationDirection};
use crate::core::ports::ClipboardPort;
use egui::{Context, RichText};

impl EguiApp {
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
            Some(&shortcut_zoom_in()),
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
            Some(&shortcut_zoom_out()),
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
            Some(&shortcut_fit_to_window()),
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
            Some(&shortcut_original_size()),
            style,
            true,
        ) {
            self.handle_reset_zoom();
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("transform", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        if self.render_menu_item(
            ui,
            "↻",
            get_text("rotate_clockwise", language),
            Some(shortcut_rotate_clockwise()),
            style,
            true,
        ) {
            self.handle_rotate_clockwise(ctx);
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "↺",
            get_text("rotate_counterclockwise", language),
            Some(shortcut_rotate_counterclockwise()),
            style,
            true,
        ) {
            self.handle_rotate_counterclockwise(ctx);
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "⇋",
            get_text("flip_horizontal", language),
            Some(shortcut_flip_horizontal()),
            style,
            true,
        ) {
            self.handle_flip_horizontal(ctx);
            clicked = true;
        }

        if self.render_menu_item(
            ui,
            "⇅",
            get_text("flip_vertical", language),
            Some(shortcut_flip_vertical()),
            style,
            true,
        ) {
            self.handle_flip_vertical(ctx);
            clicked = true;
        }

        self.render_menu_separator(ui, style);

        ui.label(
            RichText::new(get_text("clipboard", language))
                .size(11.0)
                .color(style.shortcut_color),
        );
        ui.add_space(4.0);

        let copy_image_shortcut = shortcut_copy_image();
        let copy_path_shortcut = shortcut_copy_path();
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
}
