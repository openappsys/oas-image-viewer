use crate::adapters::egui::i18n::get_text;
use crate::adapters::platform::{PlatformIntegration, SystemIntegration};
use crate::core::domain::Language;

#[derive(Clone, Copy)]
pub(super) enum IntegrationAction {
    SetDefault,
    UnsetDefault,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    AddContextMenu,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    RemoveContextMenu,
    #[cfg(target_os = "macos")]
    RefreshOpenWith,
}

pub(super) fn integration_success_text(action: IntegrationAction, language: Language) -> String {
    match action {
        IntegrationAction::SetDefault => get_text("default_app_set", language).to_string(),
        IntegrationAction::UnsetDefault => get_text("default_app_unset", language).to_string(),
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        IntegrationAction::AddContextMenu => get_text("context_menu_added", language).to_string(),
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        IntegrationAction::RemoveContextMenu => {
            get_text("context_menu_removed", language).to_string()
        }
        #[cfg(target_os = "macos")]
        IntegrationAction::RefreshOpenWith => get_text("open_with_refreshed", language).to_string(),
    }
}

pub(super) fn perform_integration_action(
    action: IntegrationAction,
    language: Language,
) -> anyhow::Result<()> {
    let integration = PlatformIntegration::new();
    match action {
        IntegrationAction::SetDefault => integration.set_as_default(language),
        IntegrationAction::UnsetDefault => integration.unset_default(language),
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        IntegrationAction::AddContextMenu => integration.add_context_menu(language),
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        IntegrationAction::RemoveContextMenu => integration.remove_context_menu(language),
        #[cfg(target_os = "macos")]
        IntegrationAction::RefreshOpenWith => integration.refresh_open_with_registration(language),
    }
}
