//! egui 应用入口与模块装配

use eframe::Frame;
use egui::Context;

use crate::core::ports::UiPort;
use crate::core::use_cases::{AppState, GalleryState, ViewState};

mod copy_shortcuts;
mod handlers;
mod lifecycle;
mod menu;
mod render;
mod shortcuts;
mod slideshow;
mod state_sync;
mod transform;
mod types;
mod utils;

pub use types::EguiApp;

impl UiPort for EguiApp {
    fn request_repaint(&self) {
        // 重绘请求通过 egui context 处理
    }

    fn show_error(&self, message: &str) {
        tracing::error!("界面错误: {}", message);
    }

    fn show_status(&self, message: &str) {
        tracing::info!("界面状态: {}", message);
    }

    fn toggle_fullscreen(&self) {
        // 全屏切换在 update 中处理
    }

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn exit(&self) {
        // 退出在 update 中处理
    }

    fn window_size(&self) -> (f32, f32) {
        (1200.0, 800.0)
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if !self.initial_file_processed {
            if let Some(path) = self.initial_file.take() {
                tracing::info!("延迟加载初始文件: {:?}", path);
                let rect = ctx.viewport_rect();
                self.add_image_to_gallery(&path);
                self.process_single_file(ctx, &path, rect.width(), rect.height());
            }
            self.initial_file_processed = true;
        }

        self.apply_theme(ctx);

        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
        });

        let language = self.service.get_language().unwrap_or_default();

        self.poll_integration_task(ctx, language);
        self.process_input(ctx, language);
        self.tick_slideshow(ctx);
        let central_response = self.render_content(ctx, _frame, language);
        self.handle_interactions(ctx);
        self.render_info_panel(ctx, language);
        self.render_context_menu(ctx, &central_response.response, language);
        self.render_drag_overlay(ctx, language);
        self.render_about_window(ctx, language);
        self.render_shortcuts_help(ctx, language);
        self.handle_copy_shortcuts(ctx);
        self.render_integration_result(ctx, language);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(pos) = self.last_saved_window_pos {
            if let Err(e) = self.set_window_position_and_save(pos.x, pos.y) {
                tracing::error!(error = %e, "更新窗口位置失败");
            }
        }

        if let Err(e) = self.set_about_window_position(self.about_window_pos) {
            tracing::error!(error = %e, "更新状态失败");
        }
        self.save_config_now();
    }
}

/// 提供默认状态
impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::default(),
            gallery: GalleryState::default(),
            config: crate::core::ports::AppConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::copy_shortcuts::{resolve_copy_action, CopyAction, CopyShortcutState};

    fn state(
        wants_keyboard_input: bool,
        has_focused_widget: bool,
        has_copy_event: bool,
        key_copy_path: bool,
        key_copy_image: bool,
        active_shift: bool,
    ) -> CopyShortcutState {
        CopyShortcutState {
            wants_keyboard_input,
            has_focused_widget,
            has_copy_event,
            key_copy_path,
            key_copy_image,
            active_shift,
        }
    }

    #[test]
    fn matrix_ctrl_c_no_text_selected() {
        let decision = resolve_copy_action(state(false, false, false, false, true, false));
        assert_eq!(decision.action, Some(CopyAction::Image));
    }

    #[test]
    fn matrix_ctrl_shift_c_no_text_selected() {
        let decision = resolve_copy_action(state(false, false, false, true, false, true));
        assert_eq!(decision.action, Some(CopyAction::Path));
    }

    #[test]
    fn matrix_ctrl_c_with_text_selected() {
        let decision = resolve_copy_action(state(true, true, false, false, true, false));
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
    }

    #[test]
    fn matrix_ctrl_shift_c_with_text_selected() {
        let decision = resolve_copy_action(state(true, true, false, true, false, true));
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
    }

    #[test]
    fn matrix_cmd_c_no_text_selected() {
        let decision = resolve_copy_action(state(false, false, true, false, false, false));
        assert_eq!(decision.action, Some(CopyAction::Image));
    }

    #[test]
    fn matrix_cmd_shift_c_no_text_selected() {
        let decision = resolve_copy_action(state(false, false, true, false, false, true));
        assert_eq!(decision.action, Some(CopyAction::Path));
    }

    #[test]
    fn matrix_cmd_c_with_text_selected() {
        let decision = resolve_copy_action(state(true, true, true, false, false, false));
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(!decision.consume_copy_event);
        assert!(!decision.consume_shift_copy_key_event);
    }

    #[test]
    fn matrix_cmd_shift_c_with_text_selected() {
        let decision = resolve_copy_action(state(true, true, true, true, false, true));
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(decision.consume_copy_event);
        assert!(decision.consume_shift_copy_key_event);
    }

    #[test]
    fn matrix_cmd_shift_c_with_focused_widget_but_no_keyboard_wants_is_blocked() {
        let decision = resolve_copy_action(state(false, true, true, true, false, true));
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(decision.consume_copy_event);
        assert!(decision.consume_shift_copy_key_event);
    }
}
