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

use crate::adapters::egui::i18n::get_text;
use crate::core::domain::{Language, NavigationDirection, ViewMode};
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
        } else if let Err(e) = self.service.update_state(|state| {
            self.service.view_use_case.toggle_view_mode(&mut state.view);
        }) {
            tracing::error!(error = %e, "切换视图模式失败");
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
            if let Err(e) = self.service.update_state(|state| {
                state.view.view_mode = ViewMode::Viewer;
            }) {
                tracing::error!(error = %e, "切换到查看器模式失败");
            }
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
            if let Err(e) = self.service.update_state(|state| {
                state.config.viewer.show_info_panel = !state.config.viewer.show_info_panel;
            }) {
                tracing::error!(error = %e, "切换信息面板失败");
            }
        }
    }

    fn handle_esc(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return;
        }

        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        } else if let Err(e) = self.service.update_state(|state| {
            if state.view.view_mode == ViewMode::Viewer {
                state.view.view_mode = ViewMode::Gallery;
            }
        }) {
            tracing::error!(error = %e, "切换到图库模式失败");
        }
    }

    fn handle_zoom_keys(&mut self, ctx: &Context) {
        // Ctrl++ 放大
        if ctx.input(|i| {
            (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)) && i.modifiers.ctrl
        }) {
            self.handle_zoom_in();
        }
        // Ctrl+- 缩小
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            self.handle_zoom_out();
        }
        // Ctrl+0 适应窗口
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            self.handle_fit_to_window(ctx);
        }
        // Ctrl+1 原始尺寸
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            self.handle_reset_zoom();
        }
    }

    fn save_window_position(&mut self, ctx: &Context) {
        // 获取窗口在屏幕上的绝对位置
        let outer_rect = ctx.input(|i| i.viewport().outer_rect);
        let current_pos = outer_rect.map(|rect| rect.left_top());

        if let Some(pos) = current_pos {
            // 只在窗口停止移动时保存（位置变化后）
            if self.last_saved_window_pos != Some(pos) {
                self.last_saved_window_pos = Some(pos);
                if let Err(e) = self.service.update_state(|state| {
                    state.config.window.x = Some(pos.x);
                    state.config.window.y = Some(pos.y);
                }) {
                    tracing::error!(error = %e, "保存窗口位置失败");
                }
                // 使用 request_save 启用防抖（500ms延迟）
                if let Ok(state) = self.service.get_state() {
                    if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                        tracing::error!(error = %e, "请求保存配置失败");
                    }
                }
            }
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

    /// 应用主题设置
    fn apply_theme(&self, ctx: &Context) {
        use crate::core::domain::Theme;

        let theme = self
            .service
            .get_state()
            .map(|s| s.config.theme)
            .unwrap_or_default();

        ctx.set_visuals(match theme {
            Theme::System => {
                // 使用 dark-light crate 检测真正的系统主题
                let is_dark = match dark_light::detect() {
                    Ok(dark_light::Mode::Dark) => true,
                    Ok(dark_light::Mode::Light) => false,
                    _ => true, // 默认或其他情况使用暗色
                };
                if is_dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                }
            }
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
            Theme::OLED => {
                // OLED 纯黑主题
                let mut visuals = egui::Visuals::dark();
                visuals.panel_fill = egui::Color32::from_rgb(0, 0, 0);
                visuals.window_fill = egui::Color32::from_rgb(0, 0, 0);
                visuals.extreme_bg_color = egui::Color32::from_rgb(0, 0, 0);
                visuals.code_bg_color = egui::Color32::from_rgb(15, 15, 15);
                visuals.faint_bg_color = egui::Color32::from_rgb(10, 10, 10);
                visuals
            }
        });
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
        // 延迟加载初始文件（确保 egui 上下文已准备好）
        if !self.initial_file_processed {
            if let Some(path) = self.initial_file.take() {
                tracing::info!("延迟加载初始文件: {:?}", path);
                let rect = ctx.viewport_rect();
                self.add_image_to_gallery(&path);
                self.process_single_file(ctx, &path, rect.width(), rect.height());
            }
            self.initial_file_processed = true;
        }

        // 应用主题设置
        self.apply_theme(ctx);

        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
        });

        // 获取当前语言
        let language = self
            .service
            .get_state()
            .map(|s| s.config.language)
            .unwrap_or_default();

        // 阶段1: 处理输入
        self.process_input(ctx, language);

        // 阶段2: 渲染内容
        let central_response = self.render_content(ctx, _frame, language);

        // 阶段3: 处理交互
        self.handle_interactions(ctx);

        // 阶段4: 渲染其他UI组件
        self.render_info_panel(ctx, language);
        self.render_context_menu(ctx, &central_response.response, language);
        self.render_drag_overlay(ctx, language);
        self.render_about_window(ctx, language);
        self.render_shortcuts_help(ctx, language);
        self.render_integration_result(ctx, language);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 使用最后一次保存的窗口位置
        if let Some(pos) = self.last_saved_window_pos {
            if let Err(e) = self.service.update_state(|state| {
                state.config.window.x = Some(pos.x);
                state.config.window.y = Some(pos.y);
            }) {
                tracing::error!(error = %e, "更新窗口位置失败");
            }
        }

        if let Err(e) = self.service.update_state(|state| {
            if let Some(pos) = self.about_window_pos {
                state.config.viewer.about_window_pos =
                    Some(crate::core::domain::Position::new(pos.x, pos.y));
            }
            if let Err(save_err) = self.service.config_use_case.save_config(&state.config) {
                tracing::error!(error = %save_err, "保存配置失败");
            } else {
                tracing::info!("配置已保存");
            }
        }) {
            tracing::error!(error = %e, "更新状态失败");
        }
    }
}

impl EguiApp {
    /// 阶段1: 处理输入 - 处理所有输入相关逻辑
    fn process_input(&mut self, ctx: &Context, language: Language) {
        self.save_window_position(ctx);
        self.gallery_widget.init(ctx);
        self.process_pending_files(ctx);
        self.handle_shortcuts(ctx);
        self.handle_drops(ctx);
        self.handle_gallery_scroll(ctx, language);
    }

    /// 处理画廊滚轮调整缩略图大小
    fn handle_gallery_scroll(&mut self, ctx: &Context, language: Language) {
        let Ok(state) = self.service.get_state() else {
            return;
        };

        // 只在画廊模式下处理
        if state.view.view_mode != ViewMode::Gallery {
            return;
        }

        let current_size = state.config.gallery.thumbnail_size;
        if let Some(new_size) = self
            .gallery_widget
            .handle_scroll(ctx, current_size, language)
        {
            // 更新配置中的缩略图大小
            if let Err(e) = self.service.update_state(|s| {
                s.config.gallery.thumbnail_size = new_size;
            }) {
                tracing::error!(error = %e, "更新缩略图大小失败");
            }
            // 请求保存配置
            if let Ok(state) = self.service.get_state() {
                if let Err(e) = self.service.config_use_case.request_save(&state.config) {
                    tracing::error!(error = %e, "请求保存配置失败");
                }
            }
        }
    }

    /// 阶段2: 渲染内容 - 渲染中央面板（图库或查看器）
    fn render_content(
        &mut self,
        ctx: &Context,
        _frame: &mut Frame,
        language: Language,
    ) -> egui::InnerResponse<()> {
        self.render_menu_bar(ctx, _frame, language);

        let texture_ref = self.current_texture.as_ref();

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut state = self.service.get_state().unwrap_or_default();

            // 同步配置中的缩略图大小到布局
            state.gallery.layout.thumbnail_size = state.config.gallery.thumbnail_size;

            match state.view.view_mode {
                ViewMode::Gallery => {
                    if let Some(index) = self.gallery_widget.ui(ui, &state.gallery, ctx, language) {
                        if let Some(image) = state.gallery.gallery.get_image(index) {
                            self.pending_clicked_image = Some(image.path().to_path_buf());
                        }
                    }
                }
                ViewMode::Viewer => {
                    self.pending_double_click = self.viewer_widget.ui(
                        ui,
                        &mut state.view,
                        &state.config.viewer,
                        texture_ref,
                        language,
                    );
                }
            }

            if let Err(e) = self.service.update_state(|s| *s = state) {
                tracing::error!(error = %e, "更新状态失败");
            }
        })
    }

    /// 阶段3: 处理交互 - 处理用户交互结果
    fn handle_interactions(&mut self, ctx: &Context) {
        // 处理双击全屏
        if self.pending_double_click {
            self.pending_double_click = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }

        // 处理图库点击图片
        if let Some(ref path) = self.pending_clicked_image.take() {
            self.load_and_set_image(ctx, path);

            let rect = ctx.viewport_rect();
            let fit_to_window = self
                .service
                .get_state()
                .map(|s| s.config.viewer.fit_to_window)
                .unwrap_or(true);

            let path = path.clone();
            if let Err(e) = self.service.update_state(|state| {
                let _ = self.service.view_use_case.open_image(
                    &path,
                    &mut state.view,
                    Some(rect.width()),
                    Some(rect.height()),
                    fit_to_window,
                );
            }) {
                tracing::error!(path = %path.display(), error = %e, "从图库打开图片失败");
            }
        }
    }

    /// 渲染右键菜单
    fn render_context_menu(
        &mut self,
        _ctx: &Context,
        response: &egui::Response,
        language: Language,
    ) {
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
            self.render_copy_image_button(ui, &path, language);
            self.render_copy_path_button(ui, &path, language);
            ui.separator();
            self.render_show_in_folder_button(ui, &path, language);
            self.render_context_result(ui);
        });
    }

    fn render_copy_image_button(
        &mut self,
        ui: &mut egui::Ui,
        path: &std::path::Path,
        language: Language,
    ) {
        let has_image = true;
        let clipboard_available = self.clipboard_manager.is_available();
        let label = format!("📋 {}", get_text("copy_image", language));

        ui.add_enabled_ui(has_image && clipboard_available, |ui| {
            if ui.button(label).clicked() {
                let copy_result = self.copy_image_to_clipboard(path);
                let success_msg = get_text("copy_image", language).to_string();
                self.handle_copy_result(copy_result, &success_msg, language);
                ui.close();
            }
        });
    }

    fn render_copy_path_button(
        &mut self,
        ui: &mut egui::Ui,
        path: &std::path::Path,
        language: Language,
    ) {
        let has_image = true;
        let clipboard_available = self.clipboard_manager.is_available();
        let label = format!("📂 {}", get_text("copy_path", language));

        ui.add_enabled_ui(has_image && clipboard_available, |ui| {
            if ui.button(label).clicked() {
                let result = ClipboardPort::copy_path(&self.clipboard_manager, path);
                let success_msg = get_text("copy_path", language).to_string();
                self.handle_copy_result(result, &success_msg, language);
                ui.close();
            }
        });
    }

    fn render_show_in_folder_button(
        &mut self,
        ui: &mut egui::Ui,
        path: &std::path::Path,
        language: Language,
    ) {
        let label = format!("📁 {}", get_text("show_in_folder", language));
        if ui.button(label).clicked() {
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
        language: Language,
    ) {
        match result {
            Ok(_) => self.last_context_menu_result = Some(success_msg.to_string()),
            Err(_) => {
                let error_msg = get_text("copy_failed", language);
                self.last_context_menu_result = Some(error_msg.to_string());
            }
        }
    }

    fn copy_image_to_clipboard(
        &self,
        path: &std::path::Path,
    ) -> Result<(), crate::core::CoreError> {
        self.clipboard_manager
            .copy_image_from_file(path)
            .map_err(|e| crate::core::CoreError::technical("CLIPBOARD_ERROR", e.to_string()))
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
