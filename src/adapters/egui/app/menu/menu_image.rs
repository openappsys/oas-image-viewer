use super::style::MenuStyle;
use super::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::adapters::egui::shortcut_labels::{
    copy_image, copy_path, fit_to_window, original_size, zoom_in, zoom_out, ShortcutTextStyle,
};
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
}
