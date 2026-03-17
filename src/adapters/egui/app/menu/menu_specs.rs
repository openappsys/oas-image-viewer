use crate::adapters::egui::i18n::get_text;
use crate::adapters::egui::shortcut_labels::{open_file, open_folder, quit, ShortcutTextStyle};
use crate::core::domain::Language;

pub(super) fn popup_item_specs(idx: usize, language: Language) -> Vec<(String, Option<String>)> {
    match idx {
        0 => vec![
            (
                get_text("open", language).to_string(),
                Some(open_file(ShortcutTextStyle::Compact)),
            ),
            (
                get_text("open_folder", language).to_string(),
                Some(open_folder(ShortcutTextStyle::Compact)),
            ),
            (
                get_text("exit", language).to_string(),
                Some(quit(ShortcutTextStyle::Compact)),
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
