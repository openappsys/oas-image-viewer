use super::style::MenuStyle;
use super::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::adapters::egui::shortcut_labels::{open_file, open_folder, quit, ShortcutTextStyle};
use crate::core::domain::Language;
use egui::{Context, RichText};

impl EguiApp {
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
}
