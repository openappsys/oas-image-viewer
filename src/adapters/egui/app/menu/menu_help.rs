use super::integration::IntegrationAction;
use super::style::MenuStyle;
use super::EguiApp;
use crate::adapters::egui::app::types::UiTaskStatus;
use crate::adapters::egui::i18n::get_text;
use crate::adapters::platform::SystemIntegration;
use crate::core::domain::Language;
use egui::{Context, RichText};

impl EguiApp {
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
        let integration_enabled = !matches!(self.task_state.status, UiTaskStatus::Running);
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
