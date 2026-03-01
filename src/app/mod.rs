//! Main application module

use std::path::PathBuf;

use eframe::Frame;
use egui::Context;
use tracing::{debug, info};

use crate::config::Config;
use crate::decoder::ImageDecoder;
use crate::dnd::{extract_image_files, is_drag_hovering, get_drag_preview_text};
use crate::gallery::Gallery;
use crate::utils::is_image_file;
use crate::viewer::Viewer;

/// Main application state
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Gallery,
    Viewer,
}

impl ImageViewerApp {
    /// Create a new application instance with the given configuration.
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config, initial_path: Option<PathBuf>) -> Self {
        debug!("Initializing ImageViewerApp");
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
        };

        // Load initial path if provided
        if let Some(path) = initial_path {
            if path.is_file() && is_image_file(&path) {
                app.open_image(path);
            } else if path.is_dir() {
                app.open_directory(path);
            }
        }

        app
    }

    /// Configure visual styles for the application.
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

    /// Open a single image file
    pub fn open_image(&mut self, path: PathBuf) {
        info!("Opening image: {:?}", path);
        
        // Check if ctx is available
        if self.viewer.get_ctx().is_none() {
            tracing::error!("Cannot open image: egui context not available");
            return;
        }
        
        // Try to load and decode the image
        match self.decoder.decode_from_file(&path) {
            Ok(img) => {
                // Convert to egui texture
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_raw();
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                let ctx = self.viewer.get_ctx().unwrap();
                let texture_name = path.file_name().unwrap_or_default().to_string_lossy();
                info!("Creating texture for: {}", texture_name);
                let texture = ctx.load_texture(
                    texture_name.to_string(),
                    color_image,
                    egui::TextureOptions::default(),
                );
                info!("Texture created successfully");
                
                
                self.viewer.set_image_with_texture(path.clone(), texture, size);
                self.current_view = View::Viewer;
                info!("Image opened successfully, switched to Viewer mode");
                    
                    // Update image list if this is a new file
                    if !self.image_list.contains(&path) {
                        self.image_list.push(path.clone());
                        self.gallery.add_image(path.clone());
                    }
                    
                    // Set current index
                    if let Some(idx) = self.image_list.iter().position(|p| p == &path) {
                        self.current_index = idx;
                    }
            }
            Err(e) => {
                tracing::error!("Failed to load image: {}", e);
            }
        }
    }

    /// Open a directory and load all images
    pub fn open_directory(&mut self, path: PathBuf) {
        info!("Opening directory: {:?}", path);
        
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
            
            // Open first image if any
            if let Some(first) = images.first() {
                self.open_image(first.clone());
            }
        }
    }

    /// Show file open dialog
    fn show_open_dialog(&mut self) {
        info!("Opening file dialog...");
        let result = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"])
            .add_filter("All Files", &["*"])
            .pick_files();
        
        info!("File dialog result: {:?}", result.is_some());
        
        if let Some(paths) = result {
            info!("Selected {} files", paths.len());
            for path in &paths {
                info!("Checking path: {:?}, extension: {:?}", path, path.extension());
                if is_image_file(path) {
                    info!("Path is image file, opening...");
                    self.open_image(path.clone());
                } else {
                    info!("Path is NOT an image file");
                }
            }
        } else {
            info!("No files selected or dialog cancelled");
        }
    }

    /// Navigate to next image
    fn next_image(&mut self) {
        if !self.image_list.is_empty() && self.current_index < self.image_list.len() - 1 {
            self.current_index += 1;
            let path = self.image_list[self.current_index].clone();
            self.open_image(path);
        }
    }

    /// Navigate to previous image
    fn prev_image(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            let path = self.image_list[self.current_index].clone();
            self.open_image(path);
        }
    }

    /// Toggle fullscreen mode
    fn toggle_fullscreen(&mut self, ctx: &Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
            !self.is_fullscreen(ctx)));
    }

    /// Check if currently fullscreen
    fn is_fullscreen(&self, ctx: &Context) -> bool {
        ctx.input(|i| i.viewport().fullscreen.unwrap_or(false))
    }

    /// Handle keyboard shortcuts
    fn handle_shortcuts(&mut self, ctx: &Context) {
        // Ctrl+O - Open file
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.show_open_dialog();
        }
        
        // Left arrow - Previous image
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.prev_image();
        }
        
        // Right arrow - Next image
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.next_image();
        }
        
        // F11 or Ctrl+Shift+F - Fullscreen toggle
        if ctx.input(|i| i.key_pressed(egui::Key::F11) ||
                    (i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift)) {
            self.toggle_fullscreen(ctx);
        }
        
        // Esc - Exit fullscreen or go back to gallery
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.is_fullscreen(ctx) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            } else if self.current_view == View::Viewer {
                self.current_view = View::Gallery;
            }
        }
        
        // Ctrl++ / Ctrl+= - Zoom in
        if ctx.input(|i| i.key_pressed(egui::Key::PlusEquals) && i.modifiers.ctrl) {
            self.viewer.zoom_in();
        }
        
        // Ctrl+- - Zoom out
        if ctx.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
            self.viewer.zoom_out();
        }
        
        // Ctrl+0 - Reset zoom
        if ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl) {
            self.viewer.reset_zoom();
        }
        
        // Ctrl+1 - 1:1 view (actual size)
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            self.viewer.reset_zoom();
        }
    }

    /// Handle file drops with support for multiple files
    fn handle_drops(&mut self, ctx: &Context) {
        // Update drag hover state for visual feedback
        self.drag_hovering = is_drag_hovering(ctx);
        
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let image_paths = extract_image_files(&i.raw.dropped_files);
                
                if !image_paths.is_empty() {
                    // Store current list length to check if we had images before
                    let had_images = !self.image_list.is_empty();
                    
                    // Add all dropped images to gallery
                    for path in &image_paths {
                        if !self.image_list.contains(path) {
                            self.image_list.push(path.clone());
                            self.gallery.add_image(path.clone());
                        }
                    }
                    
                    // Open the first dropped image
                    if let Some(first_path) = image_paths.first() {
                        // Find the index of the first image in our list
                        if let Some(idx) = self.image_list.iter().position(|p| p == first_path) {
                            self.current_index = idx;
                        }
                        
                        let path = first_path.clone();
                        let _ = i;
                        self.open_image(path);
                    }
                    
                    // Clear drag hover state
                    self.drag_hovering = false;
                }
            } else {
                // Reset hover state when not dropping
                self.drag_hovering = false;
            }
        });
    }
    
    /// Render drag overlay when files are being dragged over the window
    fn render_drag_overlay(&self, ctx: &Context) {
        if !self.drag_hovering {
            return;
        }
        
        // Create a semi-transparent overlay
        let screen_rect = ctx.screen_rect();
        
        egui::Area::new(egui::Id::new("drag_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                // Draw a highlighted border around the window
                let painter = ui.painter();
                
                // Fill with semi-transparent color
                painter.rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(52, 152, 219, 30),
                );
                
                // Draw a prominent border
                painter.rect_stroke(
                    screen_rect.shrink(2.0),
                    4.0,
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(52, 152, 219)),
                );
                
                // Draw another inner border for emphasis
                painter.rect_stroke(
                    screen_rect.shrink(8.0),
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 230)),
                );
                
                // Show drop hint in the center
                let center = screen_rect.center();
                
                // Background pill for text
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
}

impl eframe::App for ImageViewerApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Set context FIRST so it's available for file dialogs
        self.viewer.set_ctx(ctx.clone());
        
        // Handle shortcuts
        self.handle_shortcuts(ctx);
        
        // Handle file drops (updates drag_hovering state)
        self.handle_drops(ctx);
        
        // Render drag overlay if hovering
        self.render_drag_overlay(ctx);

        // Menu bar (hidden in fullscreen)
        if !self.is_fullscreen(ctx) {
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("文件", |ui| {
                        if ui.button("打开... (Ctrl+O)").clicked() {
                            self.show_open_dialog();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("退出").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("视图", |ui| {
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
                    ui.menu_button("图片", |ui| {
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
                });
            });
        }

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
        
        // 处理图库点击事件
        if let Some(path) = clicked_image {
            self.open_image(path);
        }
        
        // Handle double-click for fullscreen in viewer mode
        if self.current_view == View::Viewer && response.response.double_clicked() {
            self.toggle_fullscreen(ctx);
        }
        
        // Update viewer with frame reference for texture loading
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        debug!("Application exiting, saving configuration");
        if let Err(e) = self.config.save() {
            tracing::error!("Failed to save config on exit: {}", e);
        }
    }
}
