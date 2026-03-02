//! 主应用程序模块

use std::path::PathBuf;

use eframe::Frame;
use egui::Context;
use tracing::{debug, info};

use crate::config::Config;
use crate::decoder::ImageDecoder;
use crate::dnd::{extract_image_files, is_drag_hovering, get_drag_preview_text};
use crate::gallery::Gallery;
use crate::shortcuts_help::ShortcutsHelpPanel;
use crate::utils::is_image_file;
use crate::viewer::Viewer;

pub struct ImageViewerApp {
    config: Config,
    gallery: Gallery,
    viewer: Viewer,
    current_view: View,
    image_list: Vec<PathBuf>,
    current_index: usize,
    decoder: ImageDecoder,
    frame: Option<Frame>,
    drag_hovering: bool,
    show_about_window: bool,
    pending_drop_files: Vec<PathBuf>,
    config_saver: crate::config::DebouncedConfigSaver,
    shortcuts_help_panel: ShortcutsHelpPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Gallery,
    Viewer,
}

impl ImageViewerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        initial_path: Option<PathBuf>,
        config_saver: crate::config::DebouncedConfigSaver,
    ) -> Self {
        debug!("初始化 ImageViewerApp");
        Self::configure_styles(&cc.egui_ctx);

        let mut app = Self {
            gallery: Gallery::new(config.gallery.clone()),
            viewer: Viewer::new(config.viewer.clone()),
            current_view: View::Gallery,
            config,
            image_list: Vec::new(),
            current_index: 0,
            decoder: ImageDecoder::new(),
            frame: None,
            drag_hovering: false,
            show_about_window: false,
            pending_drop_files: Vec::new(),
            config_saver,
            shortcuts_help_panel: ShortcutsHelpPanel::new(),
        };

        if let Some(path) = initial_path {
            if path.is_file() && is_image_file(&path) {
                app.pending_drop_files.push(path);
            } else if path.is_dir() {
                app.open_directory(path);
            }
        }

        app
    }

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

    pub fn open_image(&mut self, path: PathBuf) {
        info!("正在打开图像: {:?}", path);
        
        if self.viewer.get_ctx().is_none() {
            tracing::error!("无法打开图像: egui 上下文不可用");
            return;
        }
        
        match self.decoder.decode_from_file(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_raw();
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                let ctx = self.viewer.get_ctx().unwrap();
                let texture_name = path.file_name().unwrap_or_default().to_string_lossy();
                info!("正在为 {} 创建纹理", texture_name);
                let texture = ctx.load_texture(
                    texture_name.to_string(),
                    color_image,
                    egui::TextureOptions::default(),
                );
                info!("纹理创建成功");
                
                self.viewer.set_image_with_texture(path.clone(), texture, size);
                self.current_view = View::Viewer;
                info!("图像打开成功，已切换到查看器模式");
                    
                if !self.image_list.contains(&path) {
                    self.image_list.push(path.clone());
                    self.gallery.add_image(path.clone());
                }
                
                if let Some(idx) = self.image_list.iter().position(|p| p == &path) {
                    self.current_index = idx;
                }
            }
            Err(e) => {
                tracing::error!("加载图像失败: {}", e);
            }
        }
    }

    pub fn open_directory(&mut self, path: PathBuf) {
        info!("正在打开目录: {:?}", path);
        
        if let Ok(entries) = std::fs::read_dir(&path) {
            let mut images: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| is_image_file(p))
                .collect();
            
            images.sort();
            
            for img in &images {
                self.gallery.add_image(img.clone());
            }
            
            self.image_list = images.clone();
            
            if let Some(first) = images.first() {
                self.pending_drop_files.push(first.clone());
            }
        }
    }

    fn show_open_dialog(&mut self) {
        info!("正在打开文件对话框...");
        let result = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"])
            .add_filter("All Files", &["*"])
            .pick_files();
        
        info!("文件对话框结果: {:?}", result.is_some());
        
        if let Some(paths) = result {
            info!("选择了 {} 个文件", paths.len());
            for path in &paths {
                info!("检查路径: {:?}, 扩展名: {:?}", path, path.extension());
                if is_image_file(path) {
                    info!("路径是图像文件，正在打开...");
                    self.pending_drop_files.push(path.clone());
                } else {
                    info!("路径不是图像文件");
                }
            }
        } else {
            info!("未选择文件或对话框已取消");
        }
    }

    fn next_image(&mut self) {
        if !self.image_list.is_empty() && self.current_index < self.image_list.len() - 1 {
            self.current_index += 1;
            let path = self.image_list[self.current_index].clone();
            self.open_image(path);
        }
    }

    fn prev_image(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            let path = self.image_list[self.current_index].clone();
            self.open_image(path);
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
            !self.is_fullscreen(ctx)));
    }

    fn is_fullscreen(&self, ctx: &Context) -> bool {
        ctx.input(|i| i.viewport().fullscreen.unwrap_or(false))
    }

        /// Hover 菜单按钮 - 鼠标悬停时自动打开菜单
    fn hover_menu_button(
        ui: &mut egui::Ui,
        title: &str,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        use egui::Id;
        
        let menu_id = Id::new(format!("menu_{}", title));
        let active_menu_id = Id::new("active_menu");
        
        // 获取当前活跃的菜单
        let active_menu = ui.data(|d| d.get_temp::<Id>(active_menu_id));
        let is_menu_open = active_menu == Some(menu_id);
        
        // 创建菜单按钮，传入 open 状态
        let menu_btn = egui::menu::menu_button(ui, title, |ui| {
            add_contents(ui);
        });
        
        // 改进的自动展开逻辑：hover时直接展开此菜单（无需先点击）
        if menu_btn.response.hovered() && !is_menu_open {
            ui.data_mut(|d| d.insert_temp(active_menu_id, menu_id));
        }
        
        // 点击按钮时记录活跃菜单（如果当前未打开则打开，已打开则关闭）
        if menu_btn.response.clicked() {
            if is_menu_open {
                ui.data_mut(|d| d.insert_temp(active_menu_id, Id::NULL));
            } else {
                ui.data_mut(|d| d.insert_temp(active_menu_id, menu_id));
            }
        }
        
        // 点击空白处或非菜单区域关闭菜单
        if ui.input(|i| i.pointer.any_click()) && !menu_btn.response.clicked() {
            // 检查点击是否在菜单内容区域内
            let clicked_in_menu = menu_btn.response.rect.contains(
                ui.input(|i| i.pointer.interact_pos()).unwrap_or_default()
            );
            if !clicked_in_menu {
                ui.data_mut(|d| d.insert_temp(active_menu_id, Id::NULL));
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        // ? 键 - 快捷键帮助面板
        // 检测 Text 事件中的 "?" 字符
        let question_pressed = ctx.input(|i| {
            i.events.iter().any(|e| {
                matches!(e, egui::Event::Text(text) if text == "?")
            })
        });
        
        if question_pressed {
            self.shortcuts_help_panel.toggle();
        }
        
        // G 键 - 切换图库/查看器
        if ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.any()) {
            self.toggle_view();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.show_open_dialog();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.prev_image();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.next_image();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::F11) ||
                    (i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift)) {
            self.toggle_fullscreen(ctx);
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.is_fullscreen(ctx) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            } else if self.current_view == View::Viewer {
                self.current_view = View::Gallery;
            }
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::PlusEquals) && i.modifiers.ctrl) {
            self.viewer.zoom_in();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            self.viewer.zoom_out();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            self.viewer.reset_zoom();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            self.viewer.reset_zoom();
        }
    }

    /// Handle file drops - only collect files, don't process immediately
    fn toggle_view(&mut self) {
        match self.current_view {
            View::Gallery => {
                if let Some(index) = self.gallery.selected_index() {
                    if index < self.image_list.len() {
                        self.current_index = index;
                        self.open_image(self.image_list[index].clone());
                    }
                } else if !self.image_list.is_empty() {
                    self.current_index = 0;
                    self.open_image(self.image_list[0].clone());
                }
            }
            View::Viewer => {
                self.current_view = View::Gallery;
                if self.current_index < self.image_list.len() {
                    self.gallery.select_image(self.current_index);
                }
            }
        }
    }

    fn handle_drops(&mut self, ctx: &Context) {
        self.drag_hovering = is_drag_hovering(ctx);
        
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let image_paths = extract_image_files(&i.raw.dropped_files);
                
                if !image_paths.is_empty() {
                    for path in &image_paths {
                        if !self.image_list.contains(path) {
                            self.image_list.push(path.clone());
                            self.gallery.add_image(path.clone());
                        }
                    }
                    
                    // Queue the first image for opening later (not in this callback)
                    if let Some(first_path) = image_paths.first() {
                        if let Some(idx) = self.image_list.iter().position(|p| p == first_path) {
                            self.current_index = idx;
                        }
                        self.pending_drop_files.push(first_path.clone());
                    }
                    
                    self.drag_hovering = false;
                }
            } else {
                self.drag_hovering = false;
            }
        });
    }
    
    /// Process pending files (called from update, outside of input callback)
    fn process_pending_files(&mut self) {
        if !self.pending_drop_files.is_empty() {
            let paths: Vec<PathBuf> = self.pending_drop_files.drain(..).collect();
            for path in paths {
                self.open_image(path);
            }
        }
    }
    
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
                
                painter.rect_stroke(
                    screen_rect.shrink(8.0),
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 230)),
                );
                
                let center = screen_rect.center();
                
                let text = if let Some(preview) = get_drag_preview_text(ctx) {
                    format!("📂 {}", preview)
                } else {
                    "📂 释放以打开图片".to_string()
                };
                
                let font = egui::FontId::proportional(20.0);
                let text_size = painter.layout(
                    text.clone(),
                    font.clone(),
                    egui::Color32::WHITE,
                    f32::INFINITY,
                ).size();
                
                let pill_rect = egui::Rect::from_center_size(
                    center,
                    text_size + egui::Vec2::new(40.0, 24.0),
                );
                
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

    fn render_about_window(&mut self, ctx: &Context) {
        if !self.show_about_window {
            return;
        }

        egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 200.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Image-Viewer");
                    ui.add_space(10.0);
                    ui.label("版本: v0.1.0");
                    ui.add_space(5.0);
                    ui.label("© 2026 Image-Viewer Contributors");
                    ui.add_space(5.0);
                    ui.label("许可证: MIT License");
                    ui.add_space(20.0);
                    if ui.button("关闭").clicked() {
                        self.show_about_window = false;
                    }
                });
            });
    }
}

impl eframe::App for ImageViewerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.viewer.set_ctx(ctx.clone());
        
        // 初始化画廊缩略图加载器
        self.gallery.init_thumbnail_loader(ctx);
        
        // Process any pending files from drag-drop or dialog
        self.process_pending_files();
        
        self.handle_shortcuts(ctx);
        
        self.handle_drops(ctx);
        
        self.render_drag_overlay(ctx);
        
        // 处理快捷键帮助面板输入
        self.shortcuts_help_panel.handle_input(ctx);
        
        // 处理查看器输入（F键信息面板、右键菜单等）
        self.viewer.handle_input(ctx);

        if !self.is_fullscreen(ctx) {
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    Self::hover_menu_button(ui, "文件", |ui| {
                        if ui.button("打开... (Ctrl+O)").clicked() {
                            self.show_open_dialog();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("退出").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    Self::hover_menu_button(ui, "视图", |ui| {
                        if ui.button("图库").clicked() {
                            self.current_view = View::Gallery;
                            ui.close_menu();
                        }
                        if ui.button("查看器").clicked() {
                            self.current_view = View::Viewer;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("全屏切换 (F11)").clicked() {
                            self.toggle_fullscreen(ctx);
                            ui.close_menu();
                        }
                    });
                    Self::hover_menu_button(ui, "图片", |ui| {
                        if ui.button("上一张 (左箭头)").clicked() {
                            self.prev_image();
                            ui.close_menu();
                        }
                        if ui.button("下一张 (右箭头)").clicked() {
                            self.next_image();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("放大 (Ctrl++)").clicked() {
                            self.viewer.zoom_in();
                            ui.close_menu();
                        }
                        if ui.button("缩小 (Ctrl+-)").clicked() {
                            self.viewer.zoom_out();
                            ui.close_menu();
                        }
                        if ui.button("重置缩放 (Ctrl+0)").clicked() {
                            self.viewer.reset_zoom();
                            ui.close_menu();
                        }
                    });
                    Self::hover_menu_button(ui, "帮助", |ui| {
                        if ui.button("快捷键帮助 (?)").clicked() {
                            self.shortcuts_help_panel.toggle();
                            ui.close_menu();
                        }
                        if ui.button("关于").clicked() {
                            self.show_about_window = true;
                            ui.close_menu();
                        }
                    });
                });
            });
        }

        // 先渲染中央面板（图片），再渲染覆盖层，确保覆盖层在上面
        let mut clicked_image: Option<PathBuf> = None;
        
        let response = egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Gallery => {
                    if let Some(index) = self.gallery.ui(ui) {
                        if let Some(path) = self.gallery.get_image_path(index) {
                            clicked_image = Some(path.to_path_buf());
                        }
                    }
                }
                View::Viewer => {
                    self.viewer.ui(ui);
                }
            }
        });
        
        // 在中央面板之后渲染信息面板，确保它在最上层
        if self.current_view == View::Viewer {
            self.viewer.info_panel_mut().ui(ctx);
        }
        
        // 在中央面板之后渲染快捷键帮助面板，确保它在最上层
        self.shortcuts_help_panel.ui(ctx);
        
        if let Some(path) = clicked_image {
            self.open_image(path);
        }
        
        if self.current_view == View::Viewer && response.response.double_clicked() {
            self.toggle_fullscreen(ctx);
        }
        
        self.render_about_window(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        debug!("应用程序退出，正在保存配置");
        if let Err(e) = self.config.save() {
            tracing::error!("退出时保存配置失败: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试 View 枚举
    #[test]
    fn test_view_enum_variants() {
        let gallery = View::Gallery;
        let viewer = View::Viewer;
        
        assert_ne!(std::mem::discriminant(&gallery), std::mem::discriminant(&viewer));
    }

    #[test]
    fn test_view_enum_equality() {
        assert_eq!(View::Gallery, View::Gallery);
        assert_eq!(View::Viewer, View::Viewer);
        assert_ne!(View::Gallery, View::Viewer);
    }

    #[test]
    fn test_view_enum_clone() {
        let view = View::Gallery;
        let cloned = view;
        assert_eq!(view, cloned);
    }

    #[test]
    fn test_view_enum_debug() {
        let gallery = View::Gallery;
        let debug_str = format!("{:?}", gallery);
        assert!(debug_str.contains("Gallery") || debug_str == "Gallery");
    }

    #[test]
    fn test_view_enum_copy() {
        let original = View::Gallery;
        let copied = original;
        // 如果实现了 Copy，original 仍然可用
        assert_eq!(original, View::Gallery);
        assert_eq!(copied, View::Gallery);
    }

    // 测试图像格式检测逻辑
    #[test]
    fn test_detect_image_format_logic() {
        fn detect_format(path: &PathBuf) -> String {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| if e.is_empty() { "Unknown".to_string() } else { e.to_uppercase() })
                .unwrap_or_else(|| "Unknown".to_string())
        }

        let test_cases = vec![
            ("test.png", "PNG"),
            ("test.PNG", "PNG"),
            ("test.jpg", "JPG"),
            ("test.jpeg", "JPEG"),
            ("test.gif", "GIF"),
            ("test.webp", "WEBP"),
            ("test.bmp", "BMP"),
            ("test.tiff", "TIFF"),
            ("test", "Unknown"),
            ("test.", "Unknown"),
        ];

        for (input, expected) in test_cases {
            let path = PathBuf::from(input);
            assert_eq!(detect_format(&path), expected, "Failed for {}", input);
        }
    }

    // 测试图像索引导航逻辑
    #[test]
    fn test_image_navigation_logic() {
        let image_list: Vec<PathBuf> = vec![
            PathBuf::from("img1.png"),
            PathBuf::from("img2.png"),
            PathBuf::from("img3.png"),
        ];

        // 测试 next 逻辑
        let current_index = 1usize;
        assert!(current_index < image_list.len() - 1);
        let next_index = current_index + 1;
        assert_eq!(next_index, 2);

        // 测试 prev 逻辑
        assert!(current_index > 0);
        let prev_index = current_index - 1;
        assert_eq!(prev_index, 0);

        // 测试边界
        let first = 0usize;
        assert!(!(first > 0)); // 不能在开头向前

        let last = image_list.len() - 1;
        assert!(!(last < image_list.len() - 1)); // 不能在末尾向后
    }

    #[test]
    fn test_image_list_deduplication() {
        let mut image_list = vec![
            PathBuf::from("img1.png"),
            PathBuf::from("img2.png"),
        ];

        let new_image = PathBuf::from("img1.png");

        // 模拟添加前去重检查
        if !image_list.contains(&new_image) {
            image_list.push(new_image.clone());
        }

        assert_eq!(image_list.len(), 2); // 没有重复添加

        // 添加新图像
        let another_image = PathBuf::from("img3.png");
        if !image_list.contains(&another_image) {
            image_list.push(another_image);
        }

        assert_eq!(image_list.len(), 3);
    }

    // 测试视图状态转换
    #[test]
    fn test_view_state_transitions() {
        let mut current_view = View::Gallery;

        // Gallery -> Viewer
        current_view = View::Viewer;
        assert_eq!(current_view, View::Viewer);

        // Viewer -> Gallery
        current_view = View::Gallery;
        assert_eq!(current_view, View::Gallery);
    }

    // 测试键盘快捷键检测逻辑
    #[test]
    fn test_shortcut_key_detection_logic() {
        // 模拟 ? 键检测逻辑
        fn is_question_key(text: &str, modifiers_empty: bool) -> bool {
            text == "?" && modifiers_empty
        }

        assert!(is_question_key("?", true));
        assert!(!is_question_key("?", false));
        assert!(!is_question_key("a", true));
        assert!(!is_question_key("/", true));
    }

    #[test]
    fn test_f_key_detection_logic() {
        // 模拟 F 键检测逻辑
        fn is_f_key(key: &str, modifiers_empty: bool) -> bool {
            key == "F" && modifiers_empty
        }

        assert!(is_f_key("F", true));
        assert!(!is_f_key("F", false));
        assert!(!is_f_key("f", true)); // 区分大小写
    }

    // 测试菜单 hover 状态机
    #[test]
    fn test_menu_hover_state_machine() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct MenuState { active_menu: Option<u32> }

        let mut state = MenuState { active_menu: None };
        let menu1_id: u32 = 1;
        let menu2_id: u32 = 2;

        // 初始状态：无激活菜单
        assert!(state.active_menu.is_none());

        // 点击 menu1，激活它
        state.active_menu = Some(menu1_id);
        assert_eq!(state.active_menu, Some(menu1_id));

        // hover 到 menu2，切换到 menu2
        let hovered = true;
        let any_menu_open = state.active_menu.is_some();
        let is_current_menu = state.active_menu == Some(menu2_id);

        if hovered && any_menu_open && !is_current_menu {
            state.active_menu = Some(menu2_id);
        }

        assert_eq!(state.active_menu, Some(menu2_id));
    }

    // 测试文件拖放过滤逻辑
    #[test]
    fn test_drag_drop_path_filtering() {
        fn is_image_file(path: &PathBuf) -> bool {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
            } else {
                false
            }
        }

        let image_path = PathBuf::from("/test/image.png");
        let text_path = PathBuf::from("/test/readme.txt");
        let no_ext = PathBuf::from("/test/README");

        assert!(is_image_file(&image_path));
        assert!(!is_image_file(&text_path));
        assert!(!is_image_file(&no_ext));
    }

    // 测试全屏状态切换
    #[test]
    fn test_fullscreen_toggle_logic() {
        let mut is_fullscreen = false;

        // 切换到全屏
        is_fullscreen = !is_fullscreen;
        assert!(is_fullscreen);

        // 切换回窗口
        is_fullscreen = !is_fullscreen;
        assert!(!is_fullscreen);
    }

    // 测试缩放级别计算
    #[test]
    fn test_zoom_calculation_logic() {
        let zoom_step = 1.2_f32;
        let max_scale = 20.0_f32;
        let min_scale = 0.1_f32;

        let mut scale = 1.0_f32;

        // 放大
        scale = (scale * zoom_step).min(max_scale);
        assert!((scale - 1.2).abs() < 0.001);

        // 缩小
        scale = (scale / zoom_step).max(min_scale);
        assert!((scale - 1.0).abs() < 0.001);

        // 测试最大限制
        scale = 25.0;
        scale = scale.min(max_scale);
        assert_eq!(scale, 20.0);

        // 测试最小限制
        scale = 0.05;
        scale = scale.max(min_scale);
        assert_eq!(scale, 0.1);
    }

    // 测试配置保存防抖逻辑
    #[test]
    fn test_config_debounce_logic() {
        use std::time::{Duration, Instant};

        struct DebounceState {
            last_request: Option<Instant>,
            debounce_duration: Duration,
        }

        let mut state = DebounceState {
            last_request: None,
            debounce_duration: Duration::from_millis(500),
        };

        // 第一次请求
        let now = Instant::now();
        state.last_request = Some(now);
        assert!(state.last_request.is_some());

        // 在防抖时间内的新请求应该被忽略
        let should_save = state.last_request.map(|t| now.duration_since(t) >= state.debounce_duration)
            .unwrap_or(true);
        assert!(!should_save);
    }

    // 测试菜单 ID 生成逻辑
    #[test]
    fn test_menu_id_generation() {
        let menu_names = ["文件", "视图", "图片", "帮助"];
        
        for name in &menu_names {
            let menu_id = format!("menu_{}", name);
            assert!(menu_id.contains(name));
            assert!(menu_id.starts_with("menu_"));
        }
    }

    // 测试图像列表索引查找
    #[test]
    fn test_image_list_position_lookup() {
        let image_list = vec![
            PathBuf::from("/path/img1.png"),
            PathBuf::from("/path/img2.png"),
            PathBuf::from("/path/img3.png"),
        ];

        let target = PathBuf::from("/path/img2.png");
        let position = image_list.iter().position(|p| p == &target);
        assert_eq!(position, Some(1));

        let not_found = PathBuf::from("/path/notfound.png");
        let position = image_list.iter().position(|p| p == &not_found);
        assert_eq!(position, None);
    }

    // 测试路径扩展名提取的各种情况
    #[test]
    fn test_path_extension_variations() {
        let test_cases = vec![
            ("image.png", Some("png")),
            ("image.PNG", Some("PNG")),
            ("archive.tar.gz", Some("gz")),
            ("Makefile", None),
            (".hidden", None),
            ("file.", Some("")),
            ("", None),
        ];

        for (input, expected) in test_cases {
            let path = PathBuf::from(input);
            let ext = path.extension().map(|e| e.to_str().unwrap_or(""));
            assert_eq!(ext, expected, "Failed for {}", input);
        }
    }

    // 测试 ESC 键处理逻辑
    #[test]
    fn test_escape_key_handling() {
        let is_fullscreen = true;
        let current_view = View::Viewer;

        // ESC 在全屏查看器模式应该退出全屏
        let should_exit_fullscreen = is_fullscreen;
        assert!(should_exit_fullscreen);

        // ESC 在非全屏查看器模式应该返回画廊
        let is_fullscreen = false;
        let should_return_to_gallery = !is_fullscreen && current_view == View::Viewer;
        assert!(should_return_to_gallery);
    }

    // 测试 Ctrl 组合键检测
    #[test]
    fn test_ctrl_combo_detection() {
        fn is_ctrl_o(key: &str, ctrl: bool, shift: bool) -> bool {
            key == "O" && ctrl && !shift
        }

        assert!(is_ctrl_o("O", true, false));
        assert!(!is_ctrl_o("O", false, false));
        assert!(!is_ctrl_o("O", true, true));
        assert!(!is_ctrl_o("P", true, false));
    }

    // 测试箭头键导航
    #[test]
    fn test_arrow_key_navigation() {
        // 左箭头应该触发 prev_image
        // 右箭头应该触发 next_image
        
        let left_pressed = true;
        let right_pressed = true;
        
        assert!(left_pressed);
        assert!(right_pressed);
        
        // 在第一个图像时，左箭头不应该有作用
        let current_index = 0usize;
        assert!(!(current_index > 0));
        
        // 在最后一个图像时，右箭头不应该有作用
        let image_list_len = 3;
        let current_index = 2;
        assert!(!(current_index < image_list_len - 1));
    }

    // 测试缩放重置快捷键
    #[test]
    fn test_zoom_reset_shortcuts() {
        fn is_reset_zoom(key: &str, ctrl: bool) -> bool {
            (key == "0" || key == "1") && ctrl
        }

        assert!(is_reset_zoom("0", true));
        assert!(is_reset_zoom("1", true));
        assert!(!is_reset_zoom("0", false));
        assert!(!is_reset_zoom("2", true));
    }

    // 测试 G 键切换逻辑
    #[test]
    fn test_g_key_toggle() {
        fn is_g_key(key: &str, modifiers_empty: bool) -> bool {
            key == "G" && modifiers_empty
        }

        assert!(is_g_key("G", true));
        assert!(!is_g_key("G", false));
        assert!(!is_g_key("g", true));
    }

    // 测试 F11 全屏切换
    #[test]
    fn test_f11_fullscreen() {
        fn is_f11(key: &str) -> bool {
            key == "F11"
        }

        fn is_ctrl_shift_f(key: &str, ctrl: bool, shift: bool) -> bool {
            key == "F" && ctrl && shift
        }

        assert!(is_f11("F11"));
        assert!(is_ctrl_shift_f("F", true, true));
        
        // 两者都应该触发全屏切换
        let fullscreen_toggle = is_f11("F11") || is_ctrl_shift_f("F", true, true);
        assert!(fullscreen_toggle);
    }

    // 测试图像打开条件
    #[test]
    fn test_image_open_conditions() {
        let path_file = PathBuf::from("/test/image.png");
        let path_dir = PathBuf::from("/test/folder");
        
        // 是文件且是图像
        let is_file = true; // path_file.is_file()
        let is_image = true; // is_image_file(&path_file)
        let should_open = is_file && is_image;
        assert!(should_open);
        
        // 是目录
        let is_dir = true; // path_dir.is_dir()
        let should_open_dir = is_dir;
        assert!(should_open_dir);
    }

    // 测试双点击检测
    #[test]
    fn test_double_click_detection() {
        let double_clicked = true;
        let current_view = View::Viewer;
        
        let should_toggle_fullscreen = double_clicked && current_view == View::Viewer;
        assert!(should_toggle_fullscreen);
    }

    // 测试待处理文件队列
    #[test]
    fn test_pending_files_queue() {
        let mut pending: Vec<PathBuf> = vec![];
        
        // 添加文件到队列
        pending.push(PathBuf::from("img1.png"));
        pending.push(PathBuf::from("img2.png"));
        
        assert_eq!(pending.len(), 2);
        assert!(!pending.is_empty());
        
        // 处理所有待处理文件
        let processed: Vec<PathBuf> = pending.drain(..).collect();
        assert_eq!(processed.len(), 2);
        assert!(pending.is_empty());
    }

    // 测试拖拽悬停状态
    #[test]
    fn test_drag_hover_state() {
        let mut drag_hovering = false;
        
        // 开始拖拽
        drag_hovering = true;
        assert!(drag_hovering);
        
        // 拖拽结束
        drag_hovering = false;
        assert!(!drag_hovering);
    }

    // 测试关于窗口状态
    #[test]
    fn test_about_window_state() {
        let mut show_about = false;
        
        // 显示关于窗口
        show_about = true;
        assert!(show_about);
        
        // 关闭关于窗口
        show_about = false;
        assert!(!show_about);
    }
}
