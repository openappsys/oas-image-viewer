//! 键盘快捷键输入处理与动作分发

use super::types::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::{Color, NavigationDirection, ViewMode};
use crate::core::ports::ClipboardPort;
use egui::Context;

use super::copy_shortcuts::{
    collect_copy_shortcut_signals, resolve_copy_action, should_block_shift_copy_in_focused_context,
    CopyAction, CopyShortcutState,
};

impl EguiApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &Context) {
        if self.shortcuts_help_panel.handle_input(ctx) {
            return;
        }

        self.suppress_shift_copy_shortcut_before_widgets(ctx);
        self.handle_g_key(ctx);
        self.handle_ctrl_shift_o(ctx);
        self.handle_ctrl_o(ctx);
        self.handle_navigation_keys(ctx);
        self.handle_f11(ctx);
        self.handle_f_key(ctx);
        self.handle_b_key(ctx);
        self.handle_esc(ctx);
        self.handle_zoom_keys(ctx);
        self.handle_enter(ctx);
    }

    fn suppress_shift_copy_shortcut_before_widgets(&mut self, ctx: &Context) {
        let signals = ctx.input(|i| collect_copy_shortcut_signals(&i.events, i.modifiers.shift));
        let state = CopyShortcutState {
            wants_keyboard_input: ctx.wants_keyboard_input(),
            has_focused_widget: ctx.memory(|m| m.focused().is_some()),
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        };

        if !should_block_shift_copy_in_focused_context(state) {
            return;
        }

        ctx.input_mut(|i| {
            i.events.retain(|event| {
                !matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::C,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.shift && is_primary_copy_modifier(*modifiers)
                ) && !matches!(event, egui::Event::Copy)
            });
        });
    }

    fn handle_g_key(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.any()) {
            return;
        }

        let should_open = self.should_open_from_gallery();

        if should_open {
            self.open_from_gallery(ctx);
        } else if let Err(e) = self.service.toggle_view_mode() {
            tracing::error!(error = %e, "切换视图模式失败");
        }
    }

    fn should_open_from_gallery(&self) -> bool {
        self.service
            .get_selected_gallery_image_for_open()
            .map(|selection| selection.is_some())
            .unwrap_or(false)
    }

    fn open_from_gallery(&mut self, ctx: &Context) {
        let Some((selected_path, fit_to_window)) = self
            .service
            .get_selected_gallery_image_for_open()
            .ok()
            .flatten()
        else {
            return;
        };

        if let Err(e) = self.update_view_mode(ViewMode::Viewer) {
            tracing::error!(error = %e, "切换到查看器模式失败");
        }
        self.open_image(ctx, &selected_path, fit_to_window);
    }

    fn handle_ctrl_shift_o(&mut self, ctx: &Context) {
        if ctx.input(|i| {
            i.key_pressed(egui::Key::O) && Self::is_primary_modifier(i) && i.modifiers.shift
        }) {
            self.handle_open_directory_dialog();
        }
    }

    fn handle_ctrl_o(&mut self, ctx: &Context) {
        if ctx.input(|i| {
            i.key_pressed(egui::Key::O) && Self::is_primary_modifier(i) && !i.modifiers.shift
        }) {
            self.handle_open_dialog();
        }
    }

    fn handle_navigation_keys(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.navigate_and_open(ctx, NavigationDirection::Previous);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.navigate_and_open(ctx, NavigationDirection::Next);
        }
    }

    fn handle_f11(&mut self, ctx: &Context) {
        if ctx.input(|i| {
            i.key_pressed(egui::Key::F11)
                || (i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift)
        }) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }
    }

    fn handle_f_key(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::F) && !i.modifiers.any()) {
            if let Err(e) = self.toggle_info_panel_visible() {
                tracing::error!(error = %e, "切换信息面板失败");
            }
        }
    }

    fn handle_b_key(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::B) && !i.modifiers.any()) {
            return;
        }
        if ctx.wants_keyboard_input() || ctx.memory(|m| m.focused().is_some()) {
            return;
        }

        let current = match self.service.get_viewer_settings() {
            Ok(settings) => settings.background_color,
            Err(e) => {
                tracing::error!(error = %e, "读取查看器配置失败");
                return;
            }
        };
        let next = next_background_color(current);
        if let Err(e) = self.set_viewer_background_color_and_save(next) {
            tracing::error!(error = %e, "切换查看器背景色失败");
        }
    }

    pub(super) fn handle_copy_shortcuts(&mut self, ctx: &Context) {
        let signals = ctx.input(|i| collect_copy_shortcut_signals(&i.events, i.modifiers.shift));

        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: ctx.wants_keyboard_input(),
            has_focused_widget: ctx.memory(|m| m.focused().is_some()),
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });

        if decision.consume_copy_event {
            ctx.input_mut(|i| i.events.retain(|event| !matches!(event, egui::Event::Copy)));
        }
        if decision.consume_shift_copy_key_event {
            ctx.input_mut(|i| {
                i.events.retain(|event| {
                    !matches!(
                        event,
                        egui::Event::Key {
                            key: egui::Key::C,
                            pressed: true,
                            modifiers,
                            ..
                        } if modifiers.shift && is_primary_copy_modifier(*modifiers)
                    )
                });
            });
        }
        if decision.clear_hint {
            self.last_context_menu_result = None;
        }
        let Some(action) = decision.action else {
            return;
        };

        let Some((path, language)) = self
            .service
            .get_current_view_image_path_and_language()
            .ok()
            .flatten()
        else {
            return;
        };

        match action {
            CopyAction::Path => {
                let result = ClipboardPort::copy_path(&self.clipboard_manager, &path);
                let success_msg = get_text("copy_path", language).to_string();
                self.handle_copy_result(result, &success_msg, language);
            }
            CopyAction::Image => {
                let result = self.copy_image_to_clipboard(&path);
                let success_msg = get_text("copy_image", language).to_string();
                self.handle_copy_result(result, &success_msg, language);
            }
        }
    }

    fn is_primary_modifier(input: &egui::InputState) -> bool {
        input.modifiers.command || input.modifiers.ctrl
    }

    fn handle_esc(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return;
        }

        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        } else if self
            .service
            .get_view_mode()
            .map(|mode| mode == ViewMode::Viewer)
            .unwrap_or(false)
        {
            if let Err(e) = self.update_view_mode(ViewMode::Gallery) {
                tracing::error!(error = %e, "切换到图库模式失败");
            }
        }
    }

    fn handle_zoom_keys(&mut self, ctx: &Context) {
        if ctx.input(|i| {
            (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals))
                && Self::is_primary_modifier(i)
        }) {
            self.handle_zoom_in();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && Self::is_primary_modifier(i)) {
            self.handle_zoom_out();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && Self::is_primary_modifier(i)) {
            self.handle_fit_to_window(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && Self::is_primary_modifier(i)) {
            self.handle_reset_zoom();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2) && Self::is_primary_modifier(i)) {
            self.handle_fit_to_width(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3) && Self::is_primary_modifier(i)) {
            self.handle_fit_to_height(ctx);
        }
    }

    fn handle_enter(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            return;
        }

        let Some((image_path, fit_to_window)) = self
            .service
            .get_selected_gallery_image_for_open()
            .ok()
            .flatten()
        else {
            return;
        };
        self.open_image(ctx, &image_path, fit_to_window);
    }
}

fn is_primary_copy_modifier(modifiers: egui::Modifiers) -> bool {
    #[cfg(target_os = "macos")]
    {
        modifiers.mac_cmd
    }
    #[cfg(not(target_os = "macos"))]
    {
        modifiers.ctrl
    }
}

fn next_background_color(current: Color) -> Color {
    let black = Color::rgb(0, 0, 0);
    let gray = Color::rgb(30, 30, 30);
    let white = Color::rgb(255, 255, 255);
    if current == black {
        gray
    } else if current == gray {
        white
    } else {
        black
    }
}

#[cfg(test)]
mod tests {
    use super::next_background_color;
    use crate::core::domain::Color;

    #[test]
    fn background_color_cycle_black_gray_white() {
        let black = Color::rgb(0, 0, 0);
        let gray = Color::rgb(30, 30, 30);
        let white = Color::rgb(255, 255, 255);
        assert_eq!(next_background_color(black), gray);
        assert_eq!(next_background_color(gray), white);
        assert_eq!(next_background_color(white), black);
    }

    #[test]
    fn background_color_cycle_from_custom_falls_back_to_black() {
        let custom = Color::rgb(64, 64, 64);
        assert_eq!(next_background_color(custom), Color::rgb(0, 0, 0));
    }
}
