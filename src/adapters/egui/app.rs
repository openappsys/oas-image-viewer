//! Egui 适配器 - 唯一的 egui 依赖处
//!
//! 将 egui 事件转换为 core 用例调用
//! 将 core 状态转换为 egui 显示

use eframe::Frame;
use egui::Context;
use std::path::PathBuf;
use std::sync::Arc;

use crate::adapters::clipboard::ClipboardManager;
use crate::adapters::egui::widgets::{GalleryWidget, ViewerWidget};
use crate::adapters::info_panel::InfoPanel;
use crate::adapters::shortcuts_help::ShortcutsHelpPanel;
use crate::core::domain::{is_image_file, NavigationDirection, ViewMode};
use crate::core::ports::AppConfig;
use crate::core::ports::{ClipboardPort, FileDialogPort, UiPort};
use crate::core::use_cases::{AppState, GalleryState, ImageViewerService, ViewState};

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
            state
                .config
                .viewer
                .about_window_pos
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
        style.spacing.window_margin = egui::Margin::same(10);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
        ctx.set_style(style);
    }

    /// 处理文件对话框打开
    fn handle_open_dialog(&mut self) {
        let dialog = crate::infrastructure::RfdFileDialog::new();
        if let Some(paths) = dialog.open_files() {
            for path in paths {
                // 添加到图库（与拖拽打开一致）
                let _ = self.service.update_state(|state| {
                    let image = crate::core::domain::Image::new(
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        path.clone(),
                    );
                    state.gallery.gallery.add_image(image);
                });
                self.pending_files.push(path);
            }
        }
    }

    /// 处理待处理文件
    fn process_pending_files(&mut self, ctx: &Context) {
        // 获取窗口尺寸用于计算适应窗口的缩放
        let rect = ctx.viewport_rect();
        let win_w = rect.width();
        let win_h = rect.height();

        while let Some(path) = self.pending_files.pop() {
            let path_str = path.to_string_lossy().to_string();

            // 尝试加载图像数据和纹理
            let load_result = self.load_image_with_data(ctx, &path);

            let fit_to_window = self
                .service
                .get_state()
                .map(|s| s.config.viewer.fit_to_window)
                .unwrap_or(true);

            let _ = self.service.update_state(|state| {
                // 打开图像获取元数据
                let _ = self.service.view_use_case.open_image(
                    &path,
                    &mut state.view,
                    Some(win_w),
                    Some(win_h),
                    fit_to_window,
                );
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
        let img = image::ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;

        // 转换为 RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let rgba_data = rgba.into_raw();

        // 创建 egui 图像数据
        let image_data =
            egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba_data);

        // 创建纹理
        let texture = ctx.load_texture(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
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
                    // 只添加用户拖放的文件到画廊（与 v0.2.0 一致）
                    // 不加载整个目录，只加载用户打开的文件
                    for path in &image_paths {
                        let _ = self.service.update_state(|state| {
                            // 添加单个文件到画廊（不加载整个目录）
                            let image = crate::core::domain::Image::new(
                                path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                path.clone(),
                            );
                            state.gallery.gallery.add_image(image);
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

    /// 导航到指定方向的图片并在查看器模式下打开
    fn navigate_and_open(&mut self, ctx: &Context, direction: NavigationDirection) {
        let mut new_index: Option<usize> = None;
        let _ = self.service.update_state(|state| {
            new_index = self
                .service
                .navigate_use_case
                .navigate(&mut state.gallery, direction);
        });

        // 只有在查看器模式下才打开图片
        if let Ok(state) = self.service.get_state() {
            if state.view.view_mode == ViewMode::Viewer {
                if let Some(index) = new_index {
                    if let Some(image) = state.gallery.gallery.get_image(index) {
                        let path = image.path().to_path_buf();
                        self.open_image(ctx, &path, state.config.viewer.fit_to_window);
                    }
                }
            }
        }
    }

    /// 打开图片（加载纹理并设置视图状态）
    fn open_image(&mut self, ctx: &Context, path: &std::path::Path, fit_to_window: bool) {
        // 加载纹理和数据
        self.load_and_set_image(ctx, path);

        // 获取窗口尺寸
        let rect = ctx.viewport_rect();
        let win_w = rect.width();
        let win_h = rect.height();

        // 打开图片
        let _ = self.service.update_state(|state| {
            let _ = self.service.view_use_case.open_image(
                path,
                &mut state.view,
                Some(win_w),
                Some(win_h),
                fit_to_window,
            );
        });
    }

    /// 应用缩放操作
    /// - `factor`: `Some(factor)` - 乘以指定系数（如 1.25 放大，1/1.25 缩小）；`Some(1.0)` - 100%；`None` - 适应窗口
    /// - `viewport_size`: 窗口尺寸 `(width, height)`，仅在 `factor` 为 `None` 时需要
    fn apply_zoom(&mut self, factor: Option<f32>, viewport_size: Option<(f32, f32)>) {
        let _ = self.service.update_state(|state| {
            match factor {
                None => {
                    // 适应窗口
                    if let (Some((win_w, win_h)), Some(ref image)) =
                        (viewport_size, &state.view.current_image)
                    {
                        let img_w = image.metadata().width;
                        let img_h = image.metadata().height;

                        let fit_scale =
                            crate::core::use_cases::ViewImageUseCase::calculate_fit_scale(
                                img_w, img_h, win_w, win_h,
                            );

                        state.view.scale = crate::core::domain::Scale::new(
                            fit_scale,
                            state.config.viewer.min_scale,
                            state.config.viewer.max_scale,
                        );
                        state.view.offset = crate::core::domain::Position::default();
                        state.view.user_zoomed = true;
                    }
                }
                Some(1.0) => {
                    // 100% 原始尺寸
                    state.view.scale = crate::core::domain::Scale::new(
                        1.0,
                        state.config.viewer.min_scale,
                        state.config.viewer.max_scale,
                    );
                    state.view.offset = crate::core::domain::Position::default();
                    state.view.user_zoomed = true;
                }
                Some(multiplier) => {
                    // 按比例缩放
                    if !state.view.user_zoomed {
                        state.view.scale = crate::core::domain::Scale::new(
                            1.0,
                            state.config.viewer.min_scale,
                            state.config.viewer.max_scale,
                        );
                        state.view.user_zoomed = true;
                    }
                    let current = state.view.scale.value();
                    let new_scale = if multiplier > 1.0 {
                        (current * multiplier).min(state.config.viewer.max_scale)
                    } else {
                        (current * multiplier).max(state.config.viewer.min_scale)
                    };
                    state.view.scale = crate::core::domain::Scale::new(
                        new_scale,
                        state.config.viewer.min_scale,
                        state.config.viewer.max_scale,
                    );
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
            let should_open_image = self
                .service
                .get_state()
                .map(|state| {
                    // 当前是画廊模式且即将切换到查看器
                    state.view.view_mode == ViewMode::Gallery
                        && state.gallery.gallery.selected_index().is_some()
                })
                .unwrap_or(false);

            if should_open_image {
                // 获取选中的图片路径和配置
                let (selected_path, fit_to_window): (Option<PathBuf>, bool) =
                    self.service.get_state().ok().map_or((None, true), |state| {
                        let path = state.gallery.gallery.selected_index().and_then(|index| {
                            state
                                .gallery
                                .gallery
                                .get_image(index)
                                .map(|img| img.path().to_path_buf())
                        });
                        (path, state.config.viewer.fit_to_window)
                    });

                if let Some(path) = selected_path {
                    // 切换到查看器模式
                    let _ = self.service.update_state(|state| {
                        state.view.view_mode = ViewMode::Viewer;
                    });
                    // 打开选中的图片
                    self.open_image(ctx, &path, fit_to_window);
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
            self.navigate_and_open(ctx, NavigationDirection::Previous);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.navigate_and_open(ctx, NavigationDirection::Next);
        }

        // F11 - 全屏（v0.2.0 兼容：也支持 Ctrl+Shift+F）
        if ctx.input(|i| {
            i.key_pressed(egui::Key::F11)
                || (i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift)
        }) {
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
        if ctx.input(|i| i.key_pressed(egui::Key::Plus) && i.modifiers.ctrl) {
            self.apply_zoom(Some(1.25), None);
        }

        // Ctrl+- 缩小 - 使用 v0.2.0 的 zoom_step (1.25)
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            self.apply_zoom(Some(1.0 / 1.25), None);
        }

        // Ctrl+0 - 适应窗口（根据窗口大小自动计算）
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            let rect = ctx.viewport_rect();
            self.apply_zoom(None, Some((rect.width(), rect.height())));
        }

        // Ctrl+1 - 1:1（原始尺寸，100%）
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            self.apply_zoom(Some(1.0), None);
        }

        // 回车键 - 图库模式下打开选中的图片
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            // 从 service 获取状态
            let state = self.service.get_state().ok();
            // 检查是否在图库模式且有选中的图片
            if let Some(s) = state {
                if s.view.view_mode == ViewMode::Gallery {
                    if let Some(selected_index) = s.gallery.gallery.selected_index() {
                        if let Some(selected_image) = s.gallery.gallery.get_image(selected_index) {
                            let image_path = selected_image.path().to_path_buf();
                            let fit_to_window = s.config.viewer.fit_to_window;
                            // 使用 open_image 辅助函数打开图片
                            self.open_image(ctx, &image_path, fit_to_window);
                        }
                    }
                }
            }
        }
    }

    /// 渲染拖拽覆盖层
    fn render_drag_overlay(&self, ctx: &Context) {
        if !self.drag_hovering {
            return;
        }

        let screen_rect = ctx.viewport_rect();

        // 获取拖拽预览文本
        let text = if let Some(preview) = Self::get_drag_preview_text(ctx) {
            format!("📂 {}", preview)
        } else {
            "📂 释放以打开图片".to_string()
        };

        egui::Area::new(egui::Id::new("drag_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let painter = ui.painter();

                // 半透明背景
                painter.rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(52, 152, 219, 30),
                );

                // 外边框（与 v0.2.0 一致）
                painter.rect_stroke(
                    screen_rect.shrink(2.0),
                    4.0,
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(52, 152, 219)),
                    egui::StrokeKind::Outside,
                );

                // 内边框（与 v0.2.0 一致）
                painter.rect_stroke(
                    screen_rect.shrink(8.0),
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 230)),
                    egui::StrokeKind::Outside,
                );

                let center = screen_rect.center();

                let font = egui::FontId::proportional(20.0);
                let text_size = painter
                    .layout(
                        text.clone(),
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

    /// 获取拖拽预览文本（显示正在拖拽的文件数量）
    fn get_drag_preview_text(ctx: &Context) -> Option<String> {
        ctx.input(|i| {
            let count = i.raw.hovered_files.len();
            if count > 1 {
                Some(format!("{} 个文件", count))
            } else if count == 1 {
                i.raw.hovered_files.first().and_then(|f| {
                    f.path
                        .as_ref()
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                })
            } else {
                None
            }
        })
    }

    /// 渲染关于窗口
    fn render_about_window(&mut self, ctx: &Context) {
        if !self.show_about {
            return;
        }

        let mut window = egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 200.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);

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
            let has_image = state.view.current_image.is_some();
            let is_viewer_mode = state.view.view_mode == ViewMode::Viewer;

            // 修复问题4: 切换到图库模式时隐藏面板
            // 只有在查看器模式下才显示信息面板
            let should_show = config_visible && has_image && is_viewer_mode;

            if should_show != self.info_panel.is_visible() {
                if should_show {
                    self.info_panel.show();
                } else {
                    self.info_panel.hide();
                }
            }

            // 修复问题5: 切换图片时更新信息
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
                // 修复问题1: 当没有图片时清除路径跟踪
                // 这样当打开新图片时能正确检测到路径变化
                if self.current_image_path.is_some() {
                    self.current_image_path = None;
                    self.info_panel.clear();
                }
            }
        }

        // 渲染信息面板（与 v0.2.0 一致）
        let closed_by_user = self.info_panel.ui(ctx);

        // 如果用户点击了信息面板右上角的关闭按钮，同步更新配置中的 show_info_panel
        if closed_by_user {
            let _ = self.service.update_state(|state| {
                state.config.viewer.show_info_panel = false;
            });
        }
    }

    /// 渲染菜单栏
    /// 渲染菜单栏（支持悬停切换）
    fn render_menu_bar(&mut self, ctx: &Context) {
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            return;
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            // 设置菜单栏样式：高亮颜色和紧凑间距
            ui.style_mut().visuals.widgets.active.weak_bg_fill =
                egui::Color32::from_rgb(52, 152, 219); // 蓝色高亮
            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                egui::Color32::from_rgb(100, 180, 230); // 浅蓝色悬停
            ui.style_mut().spacing.button_padding = egui::vec2(12.0, 6.0); // 紧凑按钮

            ui.horizontal(|ui| {
                // 状态管理：哪个菜单是打开的
                let open_menu_id = ui.id().with("open_menu");
                let mut open_menu: Option<usize> = ui.ctx().data(|d| d.get_temp(open_menu_id));

                let menus = ["文件", "视图", "图片", "帮助"];
                let mut responses: Vec<egui::Response> = Vec::new();

                // 第一遍：添加所有按钮（高亮当前打开的菜单）
                for (idx, title) in menus.iter().enumerate() {
                    let button = if open_menu == Some(idx) {
                        egui::Button::new(*title).selected(true) // 选项B：高亮当前选中
                    } else {
                        egui::Button::new(*title)
                    };
                    responses.push(ui.add(button));
                }

                // 第二遍：处理交互（点击和悬停）
                let mut new_open = open_menu;
                for (idx, response) in responses.iter().enumerate() {
                    // 点击：切换菜单
                    if response.clicked() {
                        new_open = if open_menu == Some(idx) {
                            None
                        } else {
                            Some(idx)
                        };
                    }
                    // 悬停：如果其他菜单已打开，切换到此菜单
                    if response.hovered() && open_menu.is_some() && open_menu != Some(idx) {
                        new_open = Some(idx);
                    }
                }

                // 更新状态
                if new_open != open_menu {
                    open_menu = new_open;
                    ui.ctx().data_mut(|d| {
                        if let Some(idx) = open_menu {
                            d.insert_temp(open_menu_id, idx);
                        } else {
                            d.remove_temp::<usize>(open_menu_id);
                        }
                    });
                }

                // 第三遍：显示打开的菜单
                if let Some(idx) = open_menu {
                    if let Some(button) = responses.get(idx) {
                        let popup_id = ui.id().with(format!("popup_{}", idx));
                        let anchor = egui::PopupAnchor::from(button.rect);
                        let layer_id =
                            egui::LayerId::new(egui::Order::Foreground, popup_id.with("layer"));

                        let mut should_close = false;

                        egui::Popup::new(popup_id, ui.ctx().clone(), anchor, layer_id)
                            .kind(egui::PopupKind::Menu)
                            .show(|ui| {
                                ui.set_min_width(160.0);
                                let clicked = match idx {
                                    0 => self.render_file_menu(ui, ctx),
                                    1 => self.render_view_menu(ui, ctx),
                                    2 => self.render_image_menu(ui, ctx),
                                    3 => self.render_help_menu(ui, ctx),
                                    _ => false,
                                };
                                if clicked {
                                    should_close = true;
                                }
                            });

                        // 检测 ESC 键关闭
                        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                            should_close = true;
                        }

                        // 检测点击菜单栏外部关闭
                        let pointer_pos = ui.ctx().input(|i| i.pointer.interact_pos());
                        let menu_bar_rect = ui.min_rect();
                        if ui.ctx().input(|i| i.pointer.any_click()) {
                            if let Some(pos) = pointer_pos {
                                let in_menu_bar = menu_bar_rect.contains(pos);
                                // 获取 popup 的 rect 来检测是否在菜单内点击
                                let popup_response = ui.ctx().read_response(popup_id);
                                let in_popup = popup_response.is_some_and(|r| r.rect.contains(pos));
                                if !in_menu_bar && !in_popup {
                                    should_close = true;
                                }
                            } else {
                                // 点击了但没有指针位置（可能是外部）
                                should_close = true;
                            }
                        }

                        if should_close {
                            ui.ctx().data_mut(|d| {
                                d.remove_temp::<usize>(open_menu_id);
                            });
                        }
                    }
                }
            });
        });
    }

    /// 文件菜单
    fn render_file_menu(&mut self, ui: &mut egui::Ui, ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("打开... (Ctrl+O)").clicked() {
            self.handle_open_dialog();
            clicked = true;
        }
        ui.separator();
        if ui.button("退出").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            clicked = true;
        }
        clicked
    }

    /// 视图菜单
    fn render_view_menu(&mut self, ui: &mut egui::Ui, ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("图库").clicked() {
            let _ = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Gallery);
            clicked = true;
        }
        if ui.button("查看器").clicked() {
            let _ = self
                .service
                .update_state(|s| s.view.view_mode = ViewMode::Viewer);
            clicked = true;
        }
        ui.separator();
        if ui.button("全屏切换 (F11)").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
            clicked = true;
        }
        clicked
    }

    /// 图片菜单
    fn render_image_menu(&mut self, ui: &mut egui::Ui, ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("上一张 (左箭头)").clicked() {
            let mut idx = None;
            let _ = self.service.update_state(|s| {
                idx = self.service.navigate_use_case.navigate(
                    &mut s.gallery,
                    crate::core::domain::NavigationDirection::Previous,
                )
            });
            if let Some(i) = idx {
                if let Ok(s) = self.service.get_state() {
                    if let Some(img) = s.gallery.gallery.get_image(i) {
                        let p = img.path().to_path_buf();
                        self.load_and_set_image(ctx, &p);
                    }
                }
            }
            clicked = true;
        }
        if ui.button("下一张 (右箭头)").clicked() {
            let mut idx = None;
            let _ = self.service.update_state(|s| {
                idx = self.service.navigate_use_case.navigate(
                    &mut s.gallery,
                    crate::core::domain::NavigationDirection::Next,
                )
            });
            if let Some(i) = idx {
                if let Ok(s) = self.service.get_state() {
                    if let Some(img) = s.gallery.gallery.get_image(i) {
                        let p = img.path().to_path_buf();
                        self.load_and_set_image(ctx, &p);
                    }
                }
            }
            clicked = true;
        }
        ui.separator();
        if ui.button("放大 (Ctrl++)").clicked() {
            let _ = self.service.update_state(|s| {
                let m = s.config.viewer.max_scale;
                self.service.view_use_case.zoom_in(&mut s.view, 1.25, m);
            });
            clicked = true;
        }
        if ui.button("缩小 (Ctrl+-)").clicked() {
            let _ = self.service.update_state(|s| {
                let m = s.config.viewer.min_scale;
                self.service.view_use_case.zoom_out(&mut s.view, 1.25, m);
            });
            clicked = true;
        }
        if ui.button("重置缩放 (Ctrl+0)").clicked() {
            let _ = self.service.update_state(|s| {
                let r = ctx.viewport_rect();
                if let Some(ref img) = s.view.current_image {
                    let fs = crate::core::use_cases::ViewImageUseCase::calculate_fit_scale(
                        img.metadata().width,
                        img.metadata().height,
                        r.width(),
                        r.height(),
                    );
                    s.view.scale = crate::core::domain::Scale::new(
                        fs,
                        s.config.viewer.min_scale,
                        s.config.viewer.max_scale,
                    );
                    s.view.offset = crate::core::domain::Position::default();
                    s.view.user_zoomed = true;
                }
            });
            clicked = true;
        }
        if ui.button("1:1 原始尺寸 (Ctrl+1)").clicked() {
            let _ = self.service.update_state(|s| {
                s.view.scale = crate::core::domain::Scale::new(
                    1.0,
                    s.config.viewer.min_scale,
                    s.config.viewer.max_scale,
                );
                s.view.offset = crate::core::domain::Position::default();
                s.view.user_zoomed = true;
            });
            clicked = true;
        }
        clicked
    }

    /// 帮助菜单
    fn render_help_menu(&mut self, ui: &mut egui::Ui, _ctx: &Context) -> bool {
        let mut clicked = false;
        if ui.button("快捷键帮助 (?)").clicked() {
            self.shortcuts_help_panel.toggle();
            clicked = true;
        }
        if ui.button("关于").clicked() {
            self.show_about = true;
            clicked = true;
        }
        clicked
    }
}

impl UiPort for EguiApp {
    fn request_repaint(&self) {
        // 重绘请求通过 egui context 处理
    }

    fn show_error(&self, message: &str) {
        tracing::error!("UI错误: {}", message);
        // 错误显示在 update 循环中通过 last_context_menu_result 处理
    }

    fn show_status(&self, message: &str) {
        tracing::info!("UI状态: {}", message);
        // 状态显示在 update 循环中处理
    }

    fn toggle_fullscreen(&self) {
        // 全屏切换在 update 中通过 frame 处理
    }

    fn is_fullscreen(&self) -> bool {
        false // 状态在 update 中维护
    }

    fn exit(&self) {
        // 退出在 update 中通过 frame 处理
    }

    fn window_size(&self) -> (f32, f32) {
        // 从配置获取默认大小
        (1200.0, 800.0)
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // 强制统一所有布局参数，解决浅色/暗色间距不一致问题
        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
        });

        // 注意：不要每帧调用 set_pixels_per_point()，这会导致菜单抖动

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
        let mut double_clicked_viewer = false;

        // 渲染菜单栏（与 v0.2.0 一致：在 CentralPanel 之前）
        self.render_menu_bar(ctx);

        // 获取当前纹理引用（在菜单栏之后）
        let texture_ref = self.current_texture.as_ref();

        // 渲染 CentralPanel（图片区域）
        let central_response = egui::CentralPanel::default().show(ctx, |ui| {
            let mut state = self.service.get_state().unwrap_or_default();
            // 日志函数
            fn log_panel(msg: &str) {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("debug.log")
                {
                    let _ = writeln!(file, "{}", msg);
                }
            }
            log_panel(&format!(
                "CentralPanel: view_mode={:?}",
                state.view.view_mode
            ));

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

            // 同步所有状态更改
            let _ = self.service.update_state(|s| *s = state);
        });

        // 处理双击全屏（拖拽和滚轮已在 viewer_widget 内处理）
        if double_clicked_viewer {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
            ));
        }

        // 处理画廊点击
        if let Some(ref path) = clicked_image {
            // 加载纹理和数据
            self.load_and_set_image(ctx, path);

            // 获取窗口尺寸
            let rect = ctx.viewport_rect();
            let win_w = rect.width();
            let win_h = rect.height();
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
                    Some(win_w),
                    Some(win_h),
                    fit_to_window,
                );
            });
        }

        // 渲染信息面板（在 CentralPanel 之后，确保在图片上层）
        self.render_info_panel(ctx);

        // 右键菜单（仅在查看器模式下，在 CentralPanel 上触发，不再覆盖整个窗口）
        if let Ok(state) = self.service.get_state() {
            if state.view.view_mode == ViewMode::Viewer {
                if let Some(ref image) = state.view.current_image {
                    let path = image.path().to_path_buf();
                    central_response.response.context_menu(|ui: &mut egui::Ui| {
                        ui.set_min_width(150.0);

                        let has_image = true;
                        let clipboard_available = self.clipboard_manager.is_available();

                        // 复制图片（优先使用当前已加载的 RGBA 数据）
                        ui.add_enabled_ui(has_image && clipboard_available, |ui| {
                            if ui.button("📋 复制图片").clicked() {
                                let copy_result = if let Some((width, height, ref data)) =
                                    self.current_texture_data
                                {
                                    self.clipboard_manager.copy_image(width, height, data)
                                } else {
                                    self.clipboard_manager.copy_image_from_file(&path).map_err(
                                        |e| {
                                            crate::core::CoreError::technical(
                                                "STORAGE_ERROR",
                                                e.to_string(),
                                            )
                                        },
                                    )
                                };

                                match copy_result {
                                    Ok(_) => {
                                        self.last_context_menu_result =
                                            Some("图片已复制".to_string());
                                    }
                                    Err(e) => {
                                        self.last_context_menu_result =
                                            Some(format!("复制失败: {}", e));
                                    }
                                }
                                ui.close();
                            }
                        });

                        // 复制文件路径
                        ui.add_enabled_ui(has_image && clipboard_available, |ui| {
                            if ui.button("📂 复制文件路径").clicked() {
                                match ClipboardPort::copy_path(&self.clipboard_manager, &path) {
                                    Ok(_) => {
                                        self.last_context_menu_result =
                                            Some("路径已复制".to_string());
                                    }
                                    Err(e) => {
                                        self.last_context_menu_result =
                                            Some(format!("复制失败: {}", e));
                                    }
                                }
                                ui.close();
                            }
                        });

                        ui.separator();

                        // 在文件夹中显示
                        if ui.button("📁 在文件夹中显示").clicked() {
                            let _ = ClipboardPort::show_in_folder(&self.clipboard_manager, &path);
                            ui.close();
                        }

                        // 显示上次操作结果
                        if let Some(ref result) = self.last_context_menu_result {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(result)
                                    .size(11.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                        }
                    });
                }
            }
        }

        // 渲染拖拽覆盖层
        self.render_drag_overlay(ctx);

        // 渲染关于窗口
        self.render_about_window(ctx);

        // 渲染快捷键帮助
        self.render_shortcuts_help(ctx);
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
mod tests {}
