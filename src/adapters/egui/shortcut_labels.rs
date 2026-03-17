#[derive(Clone, Copy)]
pub enum ShortcutTextStyle {
    Compact,
    Spaced,
}

fn plus(style: ShortcutTextStyle) -> &'static str {
    match style {
        ShortcutTextStyle::Compact => "+",
        ShortcutTextStyle::Spaced => " + ",
    }
}

fn cmd_or_ctrl() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    }
}

pub fn open_file(style: ShortcutTextStyle) -> String {
    format!("{}{}O", cmd_or_ctrl(), plus(style))
}

pub fn open_folder(style: ShortcutTextStyle) -> String {
    format!("{}{}Shift{}O", cmd_or_ctrl(), plus(style), plus(style))
}

pub fn copy_image(style: ShortcutTextStyle) -> String {
    format!("{}{}C", cmd_or_ctrl(), plus(style))
}

pub fn copy_path(style: ShortcutTextStyle) -> String {
    format!("{}{}Shift{}C", cmd_or_ctrl(), plus(style), plus(style))
}

pub fn quit(style: ShortcutTextStyle) -> String {
    if cfg!(target_os = "macos") {
        format!("Cmd{}Q", plus(style))
    } else {
        "Alt+F4".to_string()
    }
}

pub fn zoom_in(style: ShortcutTextStyle) -> String {
    format!("Ctrl{}+", plus(style))
}

pub fn zoom_out(style: ShortcutTextStyle) -> String {
    format!("Ctrl{}-", plus(style))
}

pub fn fit_to_window(style: ShortcutTextStyle) -> String {
    format!("Ctrl{}0", plus(style))
}

pub fn original_size(style: ShortcutTextStyle) -> String {
    format!("Ctrl{}1", plus(style))
}
