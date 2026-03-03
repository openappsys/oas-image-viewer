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
use crate::info_panel::InfoPanel;
use crate::clipboard::ClipboardManager;
use crate::shortcuts_help::ShortcutsHelpPanel;

/// Egui 应用程序适配器
pub struct EguiApp {
    service: Arc<ImageViewerService>,
    viewer_widget: ViewerWidget,
    gallery_widget: GalleryWidget,
    info_panel: InfoPanel,
    shortcuts_help_panel: ShortcutsHelpPanel,
    clipboard_manager: ClipboardManager,
    show_about: bool,
    pending_files: Vec<PathBuf>,
    drag_hovering: bool,
    /// 当前图像纹理缓存 (path, texture_handle)
    current_texture: Option<(String, egui::TextureHandle)>,
    /// 当前纹理的 RGBA 数据，用于复制到剪贴板 (width, height, data)
    current_texture_data: Option<(usize, usize, Vec<u8>)>,
    /// 当前显示的图片路径，用于检测图片变化
    current_image_path: Option<PathBuf>,
    /// 关于窗口位置
    about_window_pos: Option<egui::Pos2>,
    /// 右键菜单最后一次操作结果
    last_context_menu_result: Option<String>,
}

impl EguiApp {
    /// 创建新的 Egui 应用程序
    pub fn new(cc: &eframe::CreationContext<'_>, service: Arc<ImageViewerService>) -> Self {
        Self::configure_styles(&cc.egui_ctx);

        // 加载配置中的窗口位置
        let about_window_pos = if let Ok(state) = service.get_state() {
            state.config.viewer.about_window_pos
                .map(|p| egui::pos2(p.x, p.y))
        } else {
            None
        };

        Self {
            service,
            viewer_widget: ViewerWidget::default(),
            gallery_widget: GalleryWidget::default(),
            info_panel: InfoPanel::new(),
            shortcuts_help_panel: ShortcutsHelpPanel::new(),
            show_about: false,
            pending_files: Vec::new(),
            drag_hovering: false,
            current_texture: None,
            current_texture_data: None,
            current_image_path: None,
            about_window_pos,
            clipboard_manager: ClipboardManager::new(),
            last_context_menu_result: None,
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
            let load_result = self.load_image_with_data(ctx, &path);
            
            let _ = self.service.update_state(|state| {
                // 打开图像获取元数据
                let _ = self
                    .service
                    .view_use_case
                    .open_image(&path, &mut state.view);
            });
            
            // 更新纹理缓存和数据
            match load_result {
                Ok((texture, width, height, rgba_data)) => {
                    self.current_texture = Some((path_str, texture));
                    self.current_texture_data = Some((width, height, rgba_data));
                }
                Err(_) => {
                    self.current_texture = None;
                    self.current_texture_data = None;
                }
            }
        }
    }

    /// 加载图像纹理和原始 RGBA 数据（用于复制到剪贴板）
    fn load_image_with_data(
        &self,
        ctx: &Context,
        path: &std::path::Path,
    ) -> anyhow::Result<(egui::TextureHandle, usize, usize, Vec<u8>)> {
        use image::io::Reader as ImageReader;
        
        let img = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;
        
        // 转换为 RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let rgba_data = rgba.into_raw();
        
        // 创建 egui 图像数据
        let image_data = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &rgba_data,
        );
        
        // 创建纹理
        let texture = ctx.load_texture(
            path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            image_data,
            egui::TextureOptions::LINEAR,
        );
        
        Ok((texture, width as usize, height as usize, rgba_data))
    }

    /// 加载并设置当前图像（纹理 + RGBA数据）
    fn load_and_set_image(&mut self, ctx: &Context, path: &std::path::Path) {
        match self.load_image_with_data(ctx, path) {
            Ok((texture, width, height, rgba_data)) => {
                self.current_texture = Some((path.to_string_lossy().to_string(), texture));
                self.current_texture_data = Some((width, height, rgba_data));
                self.current_image_path = Some(path.to_path_buf());
            }
            Err(_) => {
                self.current_texture = None;
                self.current_texture_data = None;
            }
        }
    }

    /// 处理拖放
    fn handle_drops(&mut self, ctx: &Context) {
        // 检查是否有拖拽悬停
        self.drag_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        // 处理释放的文件 - 与 v0.2.0 一致：添加所有文件到画廊，打开第一张
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let mut image_paths: Vec<PathBuf> = Vec::new();
                
                for file in &i.raw.dropped_files {
                    if let Some(path) = file.path.clone() {
                        if is_image_file(&path) {
                            image_paths.push(path);
                        }
                    }
                }
                
                if !image_paths.is_empty() {
                    // 添加所有文件到画廊（与 v0.2.0 一致）
                    for path in &image_paths {
                        let _ = self.service.update_state(|state| {
                            self.service.navigate_use_case.load_directory(
                                &mut state.gallery,
                                &crate::infrastructure::FsImageSource::new(),
                                path.parent().unwrap_or(path),
                            );
                        });
                    }
                    
                    // 打开第一张图片
                    if let Some(first_path) = image_paths.first() {
                        self.pending_files.push(first_path.clone());
                    }
                    
                    self.drag_hovering = false;
                }
            }
        });
    }

    /// 处理快捷键
    fn handle_shortcuts(&mut self, ctx: &Context) {
        // 让快捷键帮助面板处理 ? 键和 Esc 键
        if self.shortcuts_help_panel.handle_input(ctx) {
            return;
        }

        // G 键 - 切换视图（v0.2.0 兼容：画廊→查看器时打开选中图片）
        if ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.any()) {
            let should_open_image = self.service.get_state().map(|state| {
                // 当前是画廊模式且即将切换到查看器
                state.view.view_mode == ViewMode::Gallery && 
                state.gallery.gallery.selected_index().is_some()
            }).unwrap_or(false);
            
            if should_open_image {
                // 获取选中的图片路径
                let selected_path: Option<PathBuf> = self.service.get_state().ok().and_then(|state| {
                    state.gallery.gallery.selected_index().and_then(|index| {
                        state.gallery.gallery.get_image(index).map(|img| img.path().to_path_buf())
                    })
                });
                
                if let Some(path) = selected_path {
                    // 切换到查看器模式
                    let _ = self.service.update_state(|state| {
                        state.view.view_mode = ViewMode::Viewer;
                    });
                    // 打开选中的图片（加载纹理和数据）
                    self.load_and_set_image(ctx, &path);
                    let _ = self.service.update_state(|state| {
                        let _ = self.service.view_use_case.open_image(&path, &mut state.view);
                    });
                }
            } else {
                // 普通切换（查看器→画廊 或 画廊无选中）
                let _ = self.service.update_state(|state| {
                    self.service.view_use_case.toggle_view_mode(&mut state.view);
                });
            }
        }

        // Ctrl+O - 打开文件
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.handle_open_dialog();
        }

        // 箭头键导航
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            let mut new_index: Option<usize> = None;
            let _ = self.service.update_state(|state| {
                if state.view.view_mode == ViewMode::Viewer {
                    new_index = self.service.navigate_use_case.navigate(
                        &mut state.gallery,
                        crate::core::domain::NavigationDirection::Previous,
                    );
                }
            });
            
            // 导航后加载选中的图片
            if let Some(index) = new_index {
                if let Ok(state) = self.service.get_state() {
                    if let Some(image) = state.gallery.gallery.get_image(index) {
                        let path = image.path().to_path_buf();
                        // 加载纹理和数据
                        self.load_and_set_image(ctx, &path);
                        // 打开图片
                        let _ = self.service.update_state(|state| {
                            let _ = self.service.view_use_case.open_image(&path, &mut state.view);
                        });
                    }
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            let mut new_index: Option<usize> = None;
            let _ = self.service.update_state(|state| {
                if state.view.view_mode == ViewMode::Viewer {
                    new_index = self.service.navigate_use_case.navigate(
                        &mut state.gallery,
                        crate::core::domain::NavigationDirection::Next,
                    );
                }
            });
            
            // 导航后加载选中的图片
            if let Some(index) = new_index {
                if let Ok(state) = self.service.get_state() {
                    if let Some(image) = state.gallery.gallery.get_image(index) {
                        let path = image.path().to_path_buf();
                        // 加载纹理和数据
                        self.load_and_set_image(ctx, &path);
                        // 打开图片
                        let _ = self.service.update_state(|state| {
                            let _ = self.service.view_use_case.open_image(&path, &mut state.view);
                        });
                    }
                }
            }
        }

        // F11 - 全屏（v0.2.0 兼容：也支持 Ctrl+Shift+F）
        if ctx.input(|i| i.key_pressed(egui::Key::F11) ||
                    (i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift)) {
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

        // Ctrl++ 放大 - 使用 v0.2.0 的 zoom_step (1.25)
        if ctx.input(|i| i.key_pressed(egui::Key::PlusEquals) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                if !state.view.user_zoomed {
                    state.view.scale = crate::core::domain::Scale::new(
                        1.0,
                        state.config.viewer.min_scale,
                        state.config.viewer.max_scale
                    );
                    state.view.user_zoomed = true;
                }
                let current = state.view.scale.value();
                let new_scale = (current * 1.25).min(state.config.viewer.max_scale);
                state.view.scale = crate::core::domain::Scale::new(
                    new_scale,
                    state.config.viewer.min_scale,
                    state.config.viewer.max_scale
                );
            });
        }

        // Ctrl+- 缩小 - 使用 v0.2.0 的 zoom_step (1.25)
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                if !state.view.user_zoomed {
                    state.view.scale = crate::core::domain::Scale::new(
                        1.0,
                        state.config.viewer.min_scale,
                        state.config.viewer.max_scale
                    );
                    state.view.user_zoomed = true;
                }
                let current = state.view.scale.value();
                let new_scale = (current / 1.25).max(state.config.viewer.min_scale);
                state.view.scale = crate::core::domain::Scale::new(
                    new_scale,
                    state.config.viewer.min_scale,
                    state.config.viewer.max_scale
                );
            });
        }

        // Ctrl+0 - 重置缩放（与 v0.2.0 一致）
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                state.view.scale = crate::core::domain::Scale::new(
                    1.0,
                    state.config.viewer.min_scale,
                    state.config.viewer.max_scale
                );
                state.view.offset = crate::core::domain::Position::default();
                state.view.user_zoomed = true;
            });
        }

        // Ctrl+0 重置缩放
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                self.service.view_use_case.reset_zoom(&mut state.view);
            });
        }
        
        // Ctrl+1 - 重置缩放（与 v0.2.0 一致）
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            let _ = self.service.update_state(|state| {
                state.view.scale = crate::core::domain::Scale::new(
                    1.0,
                    state.config.viewer.min_scale,
                    state.config.viewer.max_scale
                );
                state.view.offset = crate::core::domain::Position::default();
                state.view.user_zoomed = true;
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
        self.shortcuts_help_panel.ui(ctx);
    }

    /// 渲染信息面板 (F-104)
    /// 渲染信息面板
    fn render_info_panel(&mut self, ctx: &Context) {
        // 同步配置中的 show_info_panel 状态
        if let Ok(state) = self.service.get_state() {
            let config_visible = state.config.viewer.show_info_panel;
            if config_visible != self.info_panel.is_visible() {
                if config_visible {
                    self.info_panel.show();
                } else {
                    self.info_panel.hide();
                }
            }
            
            // 只在图片路径改变时才更新信息面板（避免每帧重复加载EXIF）
            if let Some(ref image) = state.view.current_image {
                let new_path = image.path().to_path_buf();
                if self.current_image_path.as_ref() != Some(&new_path) {
                    self.current_image_path = Some(new_path.clone());
                    self.info_panel.set_image_info(
                        &new_path,
                        (image.metadata().width, image.metadata().height),
                        &format!("{:?}", image.metadata().format),
                    );
                }
            } else {
                self.current_image_path = None;
            }
        }
        
        // 渲染信息面板并检查是否需要关闭
        let should_close = self.info_panel.ui(ctx);
        
        // 如果用户点击了关闭按钮，同步更新配置
        if should_close {
            let _ = self.service.update_state(|state| {
                state.config.viewer.show_info_panel = false;
            });
        }
    }

    /// 渲染右键菜单 (F-106, F-107)
    fn render_context_menu(&mut self, ctx: &Context, path: &std::path::Path) {
        // 使用 Area 创建右键点击检测区域（避开顶部菜单栏）
        let menu_bar_height = 30.0;
        let available_rect = ctx.screen_rect();
        let response = egui::Area::new(egui::Id::new("viewer_context_menu_area"))
            .fixed_pos(egui::pos2(available_rect.min.x, available_rect.min.y + menu_bar_height))
            .interactable(true)
            .show(ctx, |ui| {
                let size = egui::vec2(available_rect.width(), available_rect.height() - menu_bar_height);
                ui.allocate_response(size, egui::Sense::click())
            })
            .response;
        
        response.context_menu(|ui: &mut egui::Ui| {
            ui.set_min_width(150.0);
            
            let has_image = true; // 有图片
            let clipboard_available = self.clipboard_manager.is_available();
            
            // 复制图片（使用已加载的 texture_data，与 v0.2.0 一致）
            ui.add_enabled_ui(has_image && clipboard_available, |ui| {
                if ui.button("📋 复制图片").clicked() {
                    // 优先使用已加载的 RGBA 数据
                    let copy_result = if let Some((width, height, ref data)) = self.current_texture_data {
                        self.clipboard_manager.copy_image(data, width, height)
                    } else {
                        // 回退到从文件加载
                        self.clipboard_manager.copy_image_from_file(path)
                    };
                    
                    match copy_result {
                        Ok(_) => {
                            self.last_context_menu_result = Some("图片已复制".to_string());
                        }
                        Err(e) => {
                            self.last_context_menu_result = Some(format!("复制失败: {}", e));
                        }
                    }
                    ui.close_menu();
                }
            });
            
            // 复制文件路径
            ui.add_enabled_ui(has_image && clipboard_available, |ui| {
                if ui.button("📂 复制文件路径").clicked() {
                    match self.clipboard_manager.copy_image_path(path) {
                        Ok(_) => {
                            self.last_context_menu_result = Some("路径已复制".to_string());
                        }
                        Err(e) => {
                            self.last_context_menu_result = Some(format!("复制失败: {}", e));
                        }
                    }
                    ui.close_menu();
                }
            });
            
            ui.separator();
            
            // 在文件夹中显示
            if ui.button("📁 在文件夹中显示").clicked() {
                let _ = ClipboardManager::show_in_folder(path);
                ui.close_menu();
            }
            
            // 显示上次操作结果
            if let Some(ref result) = self.last_context_menu_result {
                ui.separator();
                ui.label(egui::RichText::new(result).size(11.0).color(ui.visuals().weak_text_color()));
            }
        });
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
                        self.shortcuts_help_panel.toggle();
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
        // 每帧禁用 UI 缩放，确保 Ctrl++ 只缩放图片而不是整个界面
        ctx.set_pixels_per_point(1.0);

        // 初始化画廊缩略图加载器（与 v0.2.0 一致）
        self.gallery_widget.init(ctx);
        
        // 处理待处理文件
        self.process_pending_files(ctx);

        // 处理快捷键
        self.handle_shortcuts(ctx);

        // 处理拖放
        self.handle_drops(ctx);

        // 渲染主内容（先于菜单栏，确保菜单在顶层）
        let mut clicked_image: Option<PathBuf> = None;
        let mut viewer_actions: (bool, f32, Option<egui::Pos2>, Option<egui::Vec2>) = 
            (false, 1.0, None, None);

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
                    viewer_actions = self.viewer_widget.ui(
                        ui,
                        &state.view,
                        &state.config.viewer,
                        texture_ref,
                    );
                }
            }
        });
        
        // 处理查看器动作（双击全屏、滚轮缩放、拖拽平移）
        let (double_clicked, zoom_factor, mouse_pos, drag_offset) = viewer_actions;
        
        // 处理双击全屏
        if double_clicked {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }
        
        // 处理滚轮缩放（v0.2.0 方式：以鼠标为中心，调整 offset）
        if zoom_factor != 1.0 {
            let _ = self.service.update_state(|state| {
                let current_scale = state.view.scale.value();
                let min_scale = state.config.viewer.min_scale;
                let max_scale = state.config.viewer.max_scale;
                
                // 计算新缩放值并限制范围
                let new_scale = (current_scale * zoom_factor).clamp(min_scale, max_scale);
                
                // v0.2.0 关键：以鼠标位置为中心缩放，调整 offset
                if let Some(mouse) = mouse_pos {
                    // 获取窗口中心（用于计算相对位置）
                    let rect = ctx.screen_rect();
                    let center = rect.center();
                    
                    // 计算鼠标相对于中心的偏移（包含当前的 offset）
                    let zoom_center = mouse - center;
                    let current_offset = egui::Vec2::new(state.view.offset.x, state.view.offset.y);
                    let zoom_center_relative = zoom_center - current_offset;
                    
                    // 根据缩放比例调整 offset，使鼠标指向的位置保持不动
                    let scale_ratio = new_scale / current_scale;
                    let new_offset = current_offset - zoom_center_relative * (scale_ratio - 1.0);
                    
                    state.view.offset.x = new_offset.x;
                    state.view.offset.y = new_offset.y;
                }
                
                // 更新缩放值
                state.view.scale = crate::core::domain::Scale::new(new_scale, min_scale, max_scale);
                state.view.user_zoomed = true;
            });
        }
        
        // 处理拖拽平移
        if let Some(offset) = drag_offset {
            let _ = self.service.update_state(|state| {
                state.view.offset.x += offset.x;
                state.view.offset.y += offset.y;
            });
        }

        // 处理画廊点击
        if let Some(ref path) = clicked_image {
            // 加载纹理和数据
            self.load_and_set_image(ctx, path);
            
            let path = path.clone();
            let _ = self.service.update_state(|state| {
                let _ = self
                    .service
                    .view_use_case
                    .open_image(&path, &mut state.view);
            });
        }
        
        // 渲染右键菜单（仅在查看器模式下）
        if let Ok(state) = self.service.get_state() {
            if state.view.view_mode == ViewMode::Viewer {
                if let Some(ref image) = state.view.current_image {
                    self.render_context_menu(ctx, image.path());
                }
            }
        }
        
        // 渲染拖拽覆盖层（在内容之后，但在菜单之前）
        self.render_drag_overlay(ctx);

        // 渲染菜单栏（最后渲染，确保在顶层）
        self.render_menu_bar(ctx);

        // 渲染关于窗口
        self.render_about_window(ctx);

        // 渲染快捷键帮助
        self.render_shortcuts_help(ctx);
        
        // 渲染信息面板 (F-104)
        self.render_info_panel(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 保存主窗口位置到配置
        let _ = self.service.update_state(|state| {
            // 从当前保存的位置更新配置
            if let Some(pos) = self.about_window_pos {
                state.config.viewer.about_window_pos = 
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
