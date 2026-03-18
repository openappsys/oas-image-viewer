//! 应用生命周期与主界面编排逻辑

use super::types::EguiApp;
use crate::adapters::egui::i18n::get_text;
use crate::core::domain::{Language, Theme, ViewMode};
use crate::core::ports::ClipboardPort;
use eframe::Frame;
use egui::Context;
use std::sync::mpsc::TryRecvError;

impl EguiApp {
    pub(super) fn apply_theme(&self, ctx: &Context) {
        let theme = self.service.get_theme().unwrap_or_default();

        ctx.set_visuals(match theme {
            Theme::System => {
                let is_dark = match dark_light::detect() {
                    Ok(dark_light::Mode::Dark) => true,
                    Ok(dark_light::Mode::Light) => false,
                    _ => true,
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

    pub(super) fn process_input(&mut self, ctx: &Context, language: Language) {
        self.save_window_position(ctx);
        self.gallery_widget.init(ctx);
        self.process_pending_files(ctx);
        self.handle_shortcuts(ctx);
        self.handle_drops(ctx);
        self.handle_gallery_scroll(ctx, language);
    }

    pub(super) fn poll_integration_task(&mut self, ctx: &Context, language: Language) {
        let mut message: Option<String> = None;
        let mut clear_receiver = false;

        if let Some(receiver) = &self.integration_task_receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    message = Some(result);
                    clear_receiver = true;
                }
                Err(TryRecvError::Disconnected) => {
                    message = Some(format!(
                        "{}: {}",
                        get_text("operation_failed", language),
                        get_text("integration_task_disconnected", language)
                    ));
                    clear_receiver = true;
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        if clear_receiver {
            self.integration_task_receiver = None;
            self.integration_task_running = false;
        }

        if let Some(msg) = message {
            self.last_context_menu_result = Some(msg);
            ctx.request_repaint();
        }
    }

    fn save_window_position(&mut self, ctx: &Context) {
        let outer_rect = ctx.input(|i| i.viewport().outer_rect);
        let current_pos = outer_rect.map(|rect| rect.left_top());

        if let Some(pos) = current_pos {
            if self.last_saved_window_pos != Some(pos) {
                self.last_saved_window_pos = Some(pos);
                if let Err(e) = self.set_window_position_and_save(pos.x, pos.y) {
                    tracing::error!(error = %e, "保存窗口位置失败");
                }
            }
        }
    }

    fn handle_gallery_scroll(&mut self, ctx: &Context, language: Language) {
        let Ok(Some(current_size)) = self.service.get_gallery_thumbnail_size_if_gallery_mode() else {
            return;
        };
        if let Some(new_size) = self.gallery_widget.handle_scroll(ctx, current_size, language) {
            if let Err(e) = self.set_thumbnail_size_and_save(new_size) {
                tracing::error!(error = %e, "更新缩略图大小失败");
            }
        }
    }

    pub(super) fn render_content(
        &mut self,
        ctx: &Context,
        frame: &mut Frame,
        language: Language,
    ) -> egui::InnerResponse<()> {
        self.render_menu_bar(ctx, frame, language);
        let texture_ref = self.current_texture.as_ref();

        egui::CentralPanel::default().show(ctx, |ui| {
            let view_mode = self.service.get_view_mode().unwrap_or(ViewMode::Gallery);
            match view_mode {
                ViewMode::Gallery => {
                    let gallery_state = match self.service.get_gallery_state_for_render() {
                        Ok(state) => state,
                        Err(e) => {
                            tracing::error!(error = %e, "读取图库状态失败");
                            return;
                        }
                    };
                    if let Some(index) = self.gallery_widget.ui(ui, &gallery_state, ctx, language) {
                        if let Some(image) = gallery_state.gallery.get_image(index) {
                            self.pending_clicked_image = Some(image.path().to_path_buf());
                        }
                    }
                }
                ViewMode::Viewer => {
                    let (mut view_state, viewer_settings) =
                        match self.service.get_view_state_and_settings() {
                            Ok(data) => data,
                            Err(e) => {
                                tracing::error!(error = %e, "读取查看状态失败");
                                return;
                            }
                        };
                    self.pending_double_click = self.viewer_widget.ui(
                        ui,
                        &mut view_state,
                        &viewer_settings,
                        texture_ref,
                        language,
                    );
                    if let Err(e) = self.service.set_view_state(view_state) {
                        tracing::error!(error = %e, "更新查看状态失败");
                    }
                }
            }
        })
    }

    pub(super) fn handle_interactions(&mut self, ctx: &Context) {
        if self.pending_double_click {
            self.pending_double_click = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }

        if let Some(ref path) = self.pending_clicked_image.take() {
            self.load_and_set_image(ctx, path);

            let rect = ctx.viewport_rect();
            let fit_to_window = self.service.is_fit_to_window_enabled().unwrap_or(true);

            let path = path.clone();
            if let Err(e) = self.service.open_image(
                &path,
                Some(rect.width()),
                Some(rect.height()),
                fit_to_window,
            ) {
                tracing::error!(path = %path.display(), error = %e, "从图库打开图片失败");
            }
        }
    }

    pub(super) fn render_context_menu(
        &mut self,
        _ctx: &Context,
        response: &egui::Response,
        language: Language,
    ) {
        let Ok(Some(path)) = self.service.get_current_view_image_path_if_viewer() else {
            return;
        };

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
        let clipboard_available = self.clipboard_manager.is_available();
        let label = format!("📋 {}", get_text("copy_image", language));

        ui.add_enabled_ui(clipboard_available, |ui| {
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
        let clipboard_available = self.clipboard_manager.is_available();
        let label = format!("📂 {}", get_text("copy_path", language));

        ui.add_enabled_ui(clipboard_available, |ui| {
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
            if let Err(e) = ClipboardPort::show_in_folder(&self.clipboard_manager, path) {
                tracing::warn!(path = %path.display(), error = %e, "在文件夹中显示失败");
            }
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

    pub(super) fn handle_copy_result(
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

    pub(super) fn copy_image_to_clipboard(
        &self,
        path: &std::path::Path,
    ) -> Result<(), crate::core::CoreError> {
        self.clipboard_manager
            .copy_image_from_file(path)
            .map_err(|e| crate::core::CoreError::technical("CLIPBOARD_ERROR", e.to_string()))
    }
}
