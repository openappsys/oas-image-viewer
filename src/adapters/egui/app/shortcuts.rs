//! 键盘快捷键输入处理与动作分发

use super::types::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::{NavigationDirection, ViewMode};
use crate::core::ports::ClipboardPort;
use egui::Context;

use super::copy_shortcuts::{
    collect_copy_shortcut_signals, resolve_copy_action, CopyAction, CopyShortcutState,
};

impl EguiApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &Context) {
        if self.shortcuts_help_panel.handle_input(ctx) {
            return;
        }

        self.handle_g_key(ctx);
        self.handle_ctrl_shift_o(ctx);
        self.handle_ctrl_o(ctx);
        self.handle_navigation_keys(ctx);
        self.handle_f11(ctx);
        self.handle_f_key(ctx);
        self.handle_copy_shortcuts(ctx);
        self.handle_esc(ctx);
        self.handle_zoom_keys(ctx);
        self.handle_enter(ctx);
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

    fn handle_copy_shortcuts(&mut self, ctx: &Context) {
        let signals = ctx.input(|i| collect_copy_shortcut_signals(&i.events, i.modifiers.shift));

        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: ctx.wants_keyboard_input(),
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });

        if decision.consume_copy_event {
            ctx.input_mut(|i| i.events.retain(|event| !matches!(event, egui::Event::Copy)));
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
            (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)) && i.modifiers.ctrl
        }) {
            self.handle_zoom_in();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            self.handle_zoom_out();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            self.handle_fit_to_window(ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            self.handle_reset_zoom();
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
