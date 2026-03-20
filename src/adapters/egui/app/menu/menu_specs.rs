//! 菜单弹窗条目与快捷键文案规格

use crate::adapters::egui::i18n::get_text;
use crate::adapters::egui::shortcut_labels::{
    copy_image, copy_path, fit_to_window, open_file, open_folder, original_size, quit, zoom_in,
    zoom_out, ShortcutTextStyle,
};
use crate::core::domain::Language;

pub(super) fn shortcut_open_file() -> String {
    open_file(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_open_folder() -> String {
    open_folder(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_exit() -> String {
    quit(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_gallery_toggle() -> &'static str {
    "G"
}

pub(super) fn shortcut_viewer_toggle() -> &'static str {
    "G"
}

pub(super) fn shortcut_fullscreen() -> &'static str {
    "F11"
}

pub(super) fn shortcut_slideshow_toggle() -> &'static str {
    "S"
}

pub(super) fn shortcut_zoom_in() -> String {
    zoom_in(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_zoom_out() -> String {
    zoom_out(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_fit_to_window() -> String {
    fit_to_window(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_original_size() -> String {
    original_size(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_rotate_clockwise() -> &'static str {
    "R"
}

pub(super) fn shortcut_rotate_counterclockwise() -> &'static str {
    "Shift+R"
}

pub(super) fn shortcut_flip_horizontal() -> &'static str {
    "H"
}

pub(super) fn shortcut_flip_vertical() -> &'static str {
    "V"
}

pub(super) fn shortcut_copy_image() -> String {
    copy_image(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_copy_path() -> String {
    copy_path(ShortcutTextStyle::Compact)
}

pub(super) fn shortcut_shortcuts_help() -> &'static str {
    "?"
}

pub(super) fn popup_item_specs(idx: usize, language: Language) -> Vec<(String, Option<String>)> {
    match idx {
        0 => vec![
            (
                get_text("open", language).to_string(),
                Some(shortcut_open_file()),
            ),
            (
                get_text("open_folder", language).to_string(),
                Some(shortcut_open_folder()),
            ),
            (
                get_text("exit", language).to_string(),
                Some(shortcut_exit()),
            ),
        ],
        1 => vec![
            (
                get_text("gallery", language).to_string(),
                Some(shortcut_gallery_toggle().to_string()),
            ),
            (
                get_text("viewer", language).to_string(),
                Some(shortcut_viewer_toggle().to_string()),
            ),
            (
                get_text("fullscreen", language).to_string(),
                Some(shortcut_fullscreen().to_string()),
            ),
            (
                get_text("slideshow_play", language).to_string(),
                Some(shortcut_slideshow_toggle().to_string()),
            ),
            (
                get_text("slideshow_pause", language).to_string(),
                Some(shortcut_slideshow_toggle().to_string()),
            ),
            ("1s".to_string(), None),
            ("2s".to_string(), None),
            ("3s".to_string(), None),
            ("5s".to_string(), None),
            (get_text("slideshow_end_loop", language).to_string(), None),
            (get_text("slideshow_end_stop", language).to_string(), None),
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
                Some(shortcut_zoom_in()),
            ),
            (
                get_text("zoom_out", language).to_string(),
                Some(shortcut_zoom_out()),
            ),
            (
                get_text("fit_to_window", language).to_string(),
                Some(shortcut_fit_to_window()),
            ),
            (
                get_text("original_size", language).to_string(),
                Some(shortcut_original_size()),
            ),
            (
                get_text("rotate_clockwise", language).to_string(),
                Some(shortcut_rotate_clockwise().to_string()),
            ),
            (
                get_text("rotate_counterclockwise", language).to_string(),
                Some(shortcut_rotate_counterclockwise().to_string()),
            ),
            (
                get_text("flip_horizontal", language).to_string(),
                Some(shortcut_flip_horizontal().to_string()),
            ),
            (
                get_text("flip_vertical", language).to_string(),
                Some(shortcut_flip_vertical().to_string()),
            ),
            (
                get_text("copy_image", language).to_string(),
                Some(shortcut_copy_image()),
            ),
            (
                get_text("copy_path", language).to_string(),
                Some(shortcut_copy_path()),
            ),
        ],
        3 => {
            let mut items = vec![
                (
                    get_text("shortcuts_title", language).to_string(),
                    Some(shortcut_shortcuts_help().to_string()),
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
