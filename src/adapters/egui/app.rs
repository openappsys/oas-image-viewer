//! Egui 适配器 - 唯一的 egui 依赖处
//!
//! 将 egui 事件转换为 core 用例调用
//! 将 core 状态转换为 egui 显示

use eframe::Frame;
use egui::Context;
use std::path::PathBuf;
use std::sync::Arc;

use crate::adapters::egui::widgets::{GalleryWidget, ViewerWidget};
use crate::core::domain::ViewMode;
use crate::core::ports::AppConfig;
use crate::core::ports::FileDialogPort;
use crate::core::use_cases::{AppState, GalleryState, ImageViewerService, ViewState};

/// Egui 应用程序适配器
pub struct EguiApp {
    service: Arc<ImageViewerService>,
    viewer_widget: ViewerWidget,
    gallery_widget: GalleryWidget,
    show_about: bool,
    show_shortcuts: bool,
    pending_files: Vec<PathBuf>,
    drag_hovering: bool,
    /// 当前图像纹理缓存 (path, texture_handle)
    current_texture: Option<(String, egui::TextureHandle)>,
    /// 关于窗口位置
    about_window_pos: Option<egui::Pos2>,
    /// 快捷键帮助窗口位置
    shortcuts_window_pos: Option<egui::Pos2>,
}

impl EguiApp {
    /// 创建新的 Egui 应用程序
    pub fn new(cc: &eframe::CreationContext<'_>, service: Arc<ImageViewerService>) -> Self {
        Self::configure_styles(&cc.egui_ctx);

        // 加载配置中的窗口位置
        let (about_window_pos, shortcuts_window_pos) = if let Ok(state) = service.get_state() {
            let about_pos = state.config.viewer.about_window_pos
                .map(|p| egui::pos2(p.x, p.y));
            let shortcuts_pos = state.config.viewer.shortcuts_window_pos
                .map(|p| egui::pos2(p.x, p.y));
            (about_pos, shortcuts_pos)
        } else {
            (None, None)
        };

        Self {
            service,
            viewer_widget: ViewerWidget::default(),
            gallery_widget: GalleryWidget::default(),
            show_about: false,
            show_shortcuts: false,
            pending_files: Vec::new(),
            drag_hovering: false,
            current_texture: None,
            about_window_pos,
            shortcuts_window_pos,
        }
    }

    /// 配置 egui 样式
    fn configure_styles(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(10.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        ctx.set_style(style);
    }

    /// 处理文件对话框打开
    fn handle_open_dialog(&mut self) {
        let dialog = crate::infrastructure::RfdFileDialog::new();
        if let Some(paths) = dialog.open_files() {
            for path in paths {
                self.pending_files.push(path);
            }
        }
    }

    /// 处理待处理文件
    fn process_pending_files(&mut self, ctx: &Context) {
        while let Some(path) = self.pending_files.pop() {
            let path_str = path.to_string_lossy().to_string();
            
            // 尝试加载图像数据和纹理
            let texture_result = self.load_image_texture(ctx, &path);
            
            let _ = self.service.update_state(|state| {
                // 打开图像获取元数据
                let _ = self
                    .service
                    .view_use_case
                    .open_image(&path, &mut state.view);
            });
            
            // 更新纹理缓存
            if let Ok(texture) = texture_result {
                self.current_texture = Some((path_str, texture));
            } else {
                self.current_texture = None;
            }
        }
    }

    /// 加载图像纹理
    fn load_image_texture(&self, ctx: &Context, path: &std::path::Path) -> anyhow::Result<egui::TextureHandle> {
        use image::io::Reader as ImageReader;
        
        let img = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;
        
        // 转换为 RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        
        // 创建 egui 图像数据
        let image_data = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &rgba.into_raw(),
        );
        
        // 创建纹理
        let texture = ctx.load_texture(
            path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            image_data,
            egui::TextureOptions::LINEAR,
        );
        
        Ok(texture)
    }

    /// 处理拖放
    fn handle_drops(&mut self, ctx: &Context) {
        // 检查是否有拖拽悬停
        self.drag_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        // 处理释放的文件
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = file.path.clone() {
                    if is_image_file(&path) {
                        self.pending_files.push(path);
                    }
                }
            }
        });
    }

    /// 处理快捷键
    fn handle_shortcuts(&mut self, ctx: &Context) {
        // ? 键 - 快捷键帮助
        let question_pressed = ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(text) if text == "?"))
        });
        if question_pressed {
            self.show_shortcuts = !self.show_shortcuts;
        }

        // G 键 - 切换视图
        if ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.any()) {
            let _ = self.service.update_state(|state| {
                self.service.view_use_case.toggle_view_mode(&mut state.view);
            });
        }

        // Ctrl+O - 打开文件
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.handle_open_dialog();
        }

        // 箭头键导航
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            let _ = self.service.update_state(|state| {
                if state.view.view_mode == ViewMode::Viewer {
                    self.service.navigate_use_case.navigate(
                        &mut state.gallery,
                        crate::core::domain::NavigationDirection::Previous,
                    );
                }
            });
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            let _ = self.service.update_state(|state| {
                if state.view.view_mode == ViewMode::Viewer {
                    self.service.navigate_use_case.navigate(
                        &mut state.gallery,
                        crate::core::domain::NavigationDirection::Next,
                    );
                }
            });
        }

        // F11 - 全屏
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }

        // F 键 - 切换文件信息面板
        if ctx.input(|i| i.key_pressed(egui::Key::F) && !i.modifiers.any()) {
            let _ = self.service.update_state(|state| {
                state.config.viewer.show_info_panel = !state.config.viewer.show_info_panel;
            });
        }

        // ESC - 退出全屏或返回画廊
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
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

        // Ctrl++ 放大
        if ctx.input(|i| i.key_pressed(egui::Key::PlusEquals) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                let max = state.config.viewer.max_scale;
                self.service
                    .view_use_case
                    .zoom_in(&mut state.view, 1.25, max);
            });
        }

        // Ctrl+- 缩小
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                let min = state.config.viewer.min_scale;
                self.service
                    .view_use_case
                    .zoom_out(&mut state.view, 1.25, min);
            });
        }

        // Ctrl+0 重置缩放
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                self.service.view_use_case.reset_zoom(&mut state.view);
            });
        }
    }

    /// 渲染拖拽覆盖层
    fn render_drag_overlay(&self, ctx: &Context) {
        if !self.drag_hovering {
            return;
        }

        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("drag_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let painter = ui.painter();

                painter.rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(52, 152, 219, 30),
                );

                painter.rect_stroke(
                    screen_rect.shrink(2.0),
                    4.0,
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(52, 152, 219)),
                );

                let center = screen_rect.center();
                let text = "📂 释放以打开图片";

                let font = egui::FontId::proportional(20.0);
                let text_size = painter
                    .layout(
                        text.to_string(),
                        font.clone(),
                        egui::Color32::WHITE,
                        f32::INFINITY,
                    )
                    .size();

                let pill_rect =
                    egui::Rect::from_center_size(center, text_size + egui::Vec2::new(40.0, 24.0));

                painter.rect_filled(
                    pill_rect,
                    8.0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
                );

                painter.text(
                    pill_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    font,
                    egui::Color32::WHITE,
                );
            });
    }

    /// 渲染关于窗口
    fn render_about_window(&mut self, ctx: &Context) {
        if !self.show_about {
            return;
        }

        let mut window = egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 200.0]);
        
        // 如果有保存的位置，使用它
        if let Some(pos) = self.about_window_pos {
            window = window.current_pos(pos);
        }
        
        let response = window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Image-Viewer");
                ui.add_space(10.0);
                ui.label("版本: v0.3.0");
                ui.add_space(5.0);
                ui.label("© 2026 Image-Viewer Contributors");
                ui.add_space(5.0);
                ui.label("许可证: MIT License");
                ui.add_space(20.0);
                if ui.button("关闭").clicked() {
                    self.show_about = false;
                }
            });
        });

        // 保存窗口位置
        if let Some(inner) = response {
            self.about_window_pos = Some(inner.response.rect.left_top());
        }
    }

    /// 渲染快捷键帮助
    fn render_shortcuts_help(&mut self, ctx: &Context) {
        if !self.show_shortcuts {
            return;
        }

        let mut window = egui::Window::new("快捷键帮助")
            .collapsible(false)
            .resizable(false)
            .default_size([300.0, 400.0]);
        
        // 如果有保存的位置，使用它
        if let Some(pos) = self.shortcuts_window_pos {
            window = window.current_pos(pos);
        }
        
        let response = window.show(ctx, |ui| {
            ui.label("文件操作:");
            ui.label("  Ctrl+O - 打开文件");
            ui.separator();

            ui.label("导航:");
            ui.label("  ← → - 上一张/下一张");
            ui.label("  G - 切换画廊/查看器");
            ui.separator();

            ui.label("缩放:");
            ui.label("  Ctrl++ - 放大");
            ui.label("  Ctrl+- - 缩小");
            ui.label("  Ctrl+0 - 重置缩放");
            ui.label("  鼠标滚轮 - 缩放");
            ui.separator();

            ui.label("其他:");
            ui.label("  F - 显示/隐藏文件信息面板");
            ui.label("  F11 - 全屏");
            ui.label("  ? - 显示此帮助");
            ui.label("  Esc - 退出全屏/返回");
            ui.separator();

            if ui.button("关闭").clicked() {
                self.show_shortcuts = false;
            }
        });

        // 保存窗口位置
        if let Some(inner) = response {
            self.shortcuts_window_pos = Some(inner.response.rect.left_top());
        }
    }

    /// 悬停菜单按钮
    fn hover_menu_button(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        use egui::Id;

        let menu_id = Id::new(format!("menu_{}", title));
        let active_menu_id = Id::new("active_menu");

        let active_menu = ui.data(|d| d.get_temp::<Id>(active_menu_id));
        let is_menu_open = active_menu == Some(menu_id);

        let menu_btn = egui::menu::menu_button(ui, title, |ui| {
            add_contents(ui);
        });

        if menu_btn.response.hovered() && !is_menu_open {
            ui.data_mut(|d| d.insert_temp(active_menu_id, menu_id));
        }

        if menu_btn.response.clicked() {
            if is_menu_open {
                ui.data_mut(|d| d.insert_temp(active_menu_id, Id::NULL));
            } else {
                ui.data_mut(|d| d.insert_temp(active_menu_id, menu_id));
            }
        }

        if ui.input(|i| i.pointer.any_click()) && !menu_btn.response.clicked() {
            let clicked_in_menu = menu_btn
                .response
                .rect
                .contains(ui.input(|i| i.pointer.interact_pos()).unwrap_or_default());
            if !clicked_in_menu {
                ui.data_mut(|d| d.insert_temp(active_menu_id, Id::NULL));
            }
        }
    }

    /// 渲染菜单栏
    fn render_menu_bar(&mut self, ctx: &Context) {
        // 全屏时不显示菜单
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            return;
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                Self::hover_menu_button(ui, "文件", |ui| {
                    if ui.button("打开... (Ctrl+O)").clicked() {
                        self.handle_open_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                Self::hover_menu_button(ui, "视图", |ui| {
                    if ui.button("图库").clicked() {
                        let _ = self.service.update_state(|state| {
                            state.view.view_mode = ViewMode::Gallery;
                        });
                        ui.close_menu();
                    }
                    if ui.button("查看器").clicked() {
                        let _ = self.service.update_state(|state| {
                            state.view.view_mode = ViewMode::Viewer;
                        });
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("全屏切换 (F11)").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                            !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
                        ));
                        ui.close_menu();
                    }
                });

                Self::hover_menu_button(ui, "图片", |ui| {
                    if ui.button("上一张 (左箭头)").clicked() {
                        let _ = self.service.update_state(|state| {
                            self.service.navigate_use_case.navigate(
                                &mut state.gallery,
                                crate::core::domain::NavigationDirection::Previous,
                            );
                        });
                        ui.close_menu();
                    }
                    if ui.button("下一张 (右箭头)").clicked() {
                        let _ = self.service.update_state(|state| {
                            self.service.navigate_use_case.navigate(
                                &mut state.gallery,
                                crate::core::domain::NavigationDirection::Next,
                            );
                        });
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("放大 (Ctrl++)").clicked() {
                        let _ = self.service.update_state(|state| {
                            let max = state.config.viewer.max_scale;
                            self.service
                                .view_use_case
                                .zoom_in(&mut state.view, 1.25, max);
                        });
                        ui.close_menu();
                    }
                    if ui.button("缩小 (Ctrl+-)").clicked() {
                        let _ = self.service.update_state(|state| {
                            let min = state.config.viewer.min_scale;
                            self.service
                                .view_use_case
                                .zoom_out(&mut state.view, 1.25, min);
                        });
                        ui.close_menu();
                    }
                    if ui.button("重置缩放 (Ctrl+0)").clicked() {
                        let _ = self.service.update_state(|state| {
                            self.service.view_use_case.reset_zoom(&mut state.view);
                        });
                        ui.close_menu();
                    }
                });

                Self::hover_menu_button(ui, "帮助", |ui| {
                    if ui.button("快捷键帮助 (?)").clicked() {
                        self.show_shortcuts = !self.show_shortcuts;
                        ui.close_menu();
                    }
                    if ui.button("关于").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // 处理待处理文件
        self.process_pending_files(ctx);

        // 处理快捷键
        self.handle_shortcuts(ctx);

        // 处理拖放
        self.handle_drops(ctx);

        // 渲染拖拽覆盖层
        self.render_drag_overlay(ctx);

        // 渲染菜单栏
        self.render_menu_bar(ctx);

        // 渲染主内容
        let mut clicked_image: Option<PathBuf> = None;

        // 获取当前纹理引用
        let texture_ref = self.current_texture.as_ref();

        egui::CentralPanel::default().show(ctx, |ui| {
            let state = self.service.get_state().unwrap_or_default();

            match state.view.view_mode {
                ViewMode::Gallery => {
                    if let Some(index) = self.gallery_widget.ui(ui, &state.gallery) {
                        if let Some(image) = state.gallery.gallery.get_image(index) {
                            clicked_image = Some(image.path().to_path_buf());
                        }
                    }
                }
                ViewMode::Viewer => {
                    self.viewer_widget.ui(
                        ui,
                        &state.view,
                        &state.config.viewer,
                        texture_ref,
                    );
                }
            }
        });

        // 处理画廊点击
        if let Some(ref path) = clicked_image {
            // 加载纹理
            if let Ok(texture) = self.load_image_texture(ctx, path) {
                self.current_texture = Some((path.to_string_lossy().to_string(), texture));
            } else {
                self.current_texture = None;
            }
            
            let path = path.clone();
            let _ = self.service.update_state(|state| {
                let _ = self
                    .service
                    .view_use_case
                    .open_image(&path, &mut state.view);
            });
        }

        // 渲染关于窗口
        self.render_about_window(ctx);

        // 渲染快捷键帮助
        self.render_shortcuts_help(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 保存主窗口位置和子窗口位置到配置
        let _ = self.service.update_state(|state| {
            // 从当前保存的位置更新配置
            if let Some(pos) = self.about_window_pos {
                state.config.viewer.about_window_pos = 
                    Some(crate::core::domain::Position::new(pos.x, pos.y));
            }
            if let Some(pos) = self.shortcuts_window_pos {
                state.config.viewer.shortcuts_window_pos = 
                    Some(crate::core::domain::Position::new(pos.x, pos.y));
            }
            let _ = self.service.config_use_case.save_config(&state.config);
        });
    }
}

/// 检查文件是否为图像
fn is_image_file(path: &std::path::Path) -> bool {
    use crate::core::domain::Image;
    Image::detect_format(path).is_supported()
}

/// 提供默认状态
impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::default(),
            gallery: GalleryState::default(),
            config: AppConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(std::path::Path::new("test.png")));
        assert!(is_image_file(std::path::Path::new("test.jpg")));
        assert!(!is_image_file(std::path::Path::new("test.txt")));
    }
}
