//! Egui 适配器 - 重构后的主模块
//!
//! 代码已拆分到以下子模块：
//! - types: EguiApp 结构体定义
//! - handlers: 事件处理函数
//! - menu: 菜单渲染
//! - render: 渲染逻辑
//! - utils: 工具函数

use eframe::Frame;
use egui::Context;
use std::path::PathBuf;





use crate::core::domain::{NavigationDirection, ViewMode};
use crate::core::ports::{ClipboardPort, UiPort};
use crate::core::use_cases::{AppState, GalleryState, ViewState};

mod handlers;
mod menu;
mod render;
mod types;
mod utils;

pub use types::EguiApp;

impl EguiApp {
    /// 处理快捷键
    fn handle_shortcuts(&mut self, ctx: &Context) {
        if self.shortcuts_help_panel.handle_input(ctx) {
            return;
        }

        self.handle_g_key(ctx);
        self.handle_ctrl_o(ctx);
        self.handle_navigation_keys(ctx);
        self.handle_f11(ctx);
        self.handle_f_key(ctx);
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
        } else {
            let _ = self.service.update_state(|state| {
                self.service.view_use_case.toggle_view_mode(&mut state.view);
            });
        }
    }

    fn should_open_from_gallery(&self) -> bool {
        self.service
            .get_state()
            .map(|state| {
                state.view.view_mode == ViewMode::Gallery
                    && state.gallery.gallery.selected_index().is_some()
            })
            .unwrap_or(false)
    }

    fn open_from_gallery(&mut self, ctx: &Context) {
        let (selected_path, fit_to_window) = self
            .service
            .get_state()
            .ok()
            .and_then(|state| {
                let path = state.gallery.gallery.selected_index().and_then(|index| {
                    state
                        .gallery
                        .gallery
                        .get_image(index)
                        .map(|img| img.path().to_path_buf())
                });
                path.map(|p| (p, state.config.viewer.fit_to_window))
            })
            .unwrap_or_else(|| (PathBuf::new(), true));

        if !selected_path.as_os_str().is_empty() {
            let _ = self.service.update_state(|state| {
                state.view.view_mode = ViewMode::Viewer;
            });
            self.open_image(ctx, &selected_path, fit_to_window);
        }
    }

    fn handle_ctrl_o(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
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
            let _ = self.service.update_state(|state| {
                state.config.viewer.show_info_panel = !state.config.viewer.show_info_panel;
            });
        }
    }

    fn handle_esc(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return;
        }

        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        } else {
            let _ = self.service.update_state(|state| {
                if state.view.view_mode == ViewMode::Viewer {
                    state.view.view_mode = ViewMode::Gallery;
                }
            });
        }
    }

    fn handle_zoom_keys(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Plus) && i.modifiers.ctrl) {
            self.handle_zoom_in();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            self.handle_zoom_out();
        }
    }

    fn handle_enter(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            return;
        }

        let state = self.service.get_state().ok();
        let Some(s) = state else { return };

        if s.view.view_mode != ViewMode::Gallery {
            return;
        }

        let Some(selected_index) = s.gallery.gallery.selected_index() else {
            return;
        };

        let Some(selected_image) = s.gallery.gallery.get_image(selected_index) else {
            return;
        };

        let image_path = selected_image.path().to_path_buf();
        let fit_to_window = s.config.viewer.fit_to_window;
        self.open_image(ctx, &image_path, fit_to_window);
    }
}

impl UiPort for EguiApp {
    fn request_repaint(&self) {
        // 重绘请求通过 egui context 处理
    }

    fn show_error(&self, message: &str) {
        tracing::error!("UI错误: {}", message);
    }

    fn show_status(&self, message: &str) {
        tracing::info!("UI状态: {}", message);
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
        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
        });

        self.gallery_widget.init(ctx);
        self.process_pending_files(ctx);
        self.handle_shortcuts(ctx);
        self.handle_drops(ctx);

        let mut clicked_image: Option<PathBuf> = None;
        let mut double_clicked_viewer = false;

        self.render_menu_bar(ctx, _frame);

        let texture_ref = self.current_texture.as_ref();

        let central_response = egui::CentralPanel::default().show(ctx, |ui| {
            let mut state = self.service.get_state().unwrap_or_default();

            match state.view.view_mode {
                ViewMode::Gallery => {
                    if let Some(index) = self.gallery_widget.ui(ui, &state.gallery) {
                        if let Some(image) = state.gallery.gallery.get_image(index) {
                            clicked_image = Some(image.path().to_path_buf());
                        }
                    }
                }
                ViewMode::Viewer => {
                    double_clicked_viewer = self.viewer_widget.ui(
                        ui,
                        &mut state.view,
                        &state.config.viewer,
                        texture_ref,
                    );
                }
            }

            let _ = self.service.update_state(|s| *s = state);
        });

        if double_clicked_viewer {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }

        if let Some(ref path) = clicked_image {
            self.load_and_set_image(ctx, path);

            let rect = ctx.viewport_rect();
            let fit_to_window = self
                .service
                .get_state()
                .map(|s| s.config.viewer.fit_to_window)
                .unwrap_or(true);

            let path = path.clone();
            let _ = self.service.update_state(|state| {
                let _ = self.service.view_use_case.open_image(
                    &path,
                    &mut state.view,
                    Some(rect.width()),
                    Some(rect.height()),
                    fit_to_window,
                );
            });
        }

        self.render_info_panel(ctx);
        self.render_context_menu(ctx, &central_response.response);
        self.render_drag_overlay(ctx);
        self.render_about_window(ctx);
        self.render_shortcuts_help(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.service.update_state(|state| {
            if let Some(pos) = self.about_window_pos {
                state.config.viewer.about_window_pos =
                    Some(crate::core::domain::Position::new(pos.x, pos.y));
            }
            let _ = self.service.config_use_case.save_config(&state.config);
        });
    }
}

impl EguiApp {
    /// 渲染右键菜单
    fn render_context_menu(&mut self, _ctx: &Context, response: &egui::Response) {
        let Ok(state) = self.service.get_state() else {
            return;
        };

        if state.view.view_mode != ViewMode::Viewer {
            return;
        }

        let Some(ref image) = state.view.current_image else {
            return;
        };
        let path = image.path().to_path_buf();

        response.context_menu(|ui: &mut egui::Ui| {
            ui.set_min_width(150.0);
            self.render_copy_image_button(ui, &path);
            self.render_copy_path_button(ui, &path);
            ui.separator();
            self.render_show_in_folder_button(ui, &path);
            self.render_context_result(ui);
        });
    }

    fn render_copy_image_button(&mut self, ui: &mut egui::Ui, path: &std::path::Path) {
        let has_image = true;
        let clipboard_available = self.clipboard_manager.is_available();

        ui.add_enabled_ui(has_image && clipboard_available, |ui| {
            if ui.button("📋 复制图片").clicked() {
                let copy_result = self.copy_image_to_clipboard(path);
                self.handle_copy_result(copy_result, "图片已复制");
                ui.close();
            }
        });
    }

    fn copy_image_to_clipboard(
        &self,
        path: &std::path::Path,
    ) -> Result<(), crate::core::CoreError> {
        if let Some((width, height, ref data)) = self.current_texture_data {
            self.clipboard_manager.copy_image(width, height, data)
        } else {
            self.clipboard_manager
                .copy_image_from_file(path)
                .map_err(|e| crate::core::CoreError::technical("STORAGE_ERROR", e.to_string()))
        }
    }

    fn render_copy_path_button(&mut self, ui: &mut egui::Ui, path: &std::path::Path) {
        let has_image = true;
        let clipboard_available = self.clipboard_manager.is_available();

        ui.add_enabled_ui(has_image && clipboard_available, |ui| {
            if ui.button("📂 复制文件路径").clicked() {
                let result = ClipboardPort::copy_path(&self.clipboard_manager, path);
                self.handle_copy_result(result, "路径已复制");
                ui.close();
            }
        });
    }

    fn render_show_in_folder_button(&mut self, ui: &mut egui::Ui, path: &std::path::Path) {
        if ui.button("📁 在文件夹中显示").clicked() {
            let _ = ClipboardPort::show_in_folder(&self.clipboard_manager, path);
            ui.close();
        }
    }

    fn render_context_result(&self, ui: &mut egui::Ui) {
        let Some(ref result) = self.last_context_menu_result else {
            return;
        };
        ui.separator();
        ui.label(
            egui::RichText::new(result)
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
    }

    fn handle_copy_result(
        &mut self,
        result: Result<(), crate::core::CoreError>,
        success_msg: &str,
    ) {
        match result {
            Ok(_) => self.last_context_menu_result = Some(success_msg.to_string()),
            Err(e) => self.last_context_menu_result = Some(format!("复制失败: {}", e)),
        }
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
mod tests {}
