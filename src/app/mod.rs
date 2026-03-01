//! Main application module

use crate::config::{Config, DebouncedConfigSaver};
use crate::decoder::ImageDecoder;
use crate::gallery::{Gallery, NavAction};
use crate::shortcuts_help::ShortcutsHelpPanel;
use crate::utils::is_image_file;
use crate::viewer::Viewer;
use eframe::Frame;
use egui::Context;
use std::path::PathBuf;
use tracing::{debug, info};

pub struct ImageViewerApp {
    config: Config,
    config_saver: DebouncedConfigSaver,
    gallery: Gallery,
    viewer: Viewer,
    current_view: View,
    image_list: Vec<PathBuf>,
    current_index: usize,
    decoder: ImageDecoder,
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
        config_saver: DebouncedConfigSaver,
    ) -> Self {
        Self::configure_styles(&cc.egui_ctx);
        let mut app = Self {
            gallery: Gallery::new(config.gallery.clone()),
            viewer: Viewer::new(config.viewer.clone()),
            current_view: View::Gallery,
            config,
            config_saver,
            image_list: Vec::new(),
            current_index: 0,
            decoder: ImageDecoder::new(),
            shortcuts_help_panel: ShortcutsHelpPanel::new(),
        };
        app.gallery.init_thumbnail_loader(&cc.egui_ctx);
        if let Some(path) = initial_path {
            if path.is_file() && is_image_file(&path) {
                app.open_image(path);
            } else if path.is_dir() {
                app.open_directory(path);
            }
        } else if let Some(last_dir) = app.config.last_opened_directory().map(|p| p.to_path_buf()) {
            // 如果有上次打开的目录，尝试恢复
            info!("恢复上次打开的目录: {:?}", last_dir);
            if last_dir.is_dir() {
                app.open_directory(last_dir);
            }
        }
        app
    }

    fn configure_styles(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        ctx.set_style(style);
    }

    pub fn open_image(&mut self, path: PathBuf) {
        if let Some(ctx) = self.viewer.get_ctx() {
            if let Ok(img) = self.decoder.decode_from_file(&path) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let texture = ctx.load_texture(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    color_image,
                    egui::TextureOptions::default(),
                );
                let rgba_data = rgba.as_raw().to_vec();
                self.viewer
                    .set_image_with_texture_and_data(path.clone(), texture, size, rgba_data);
                self.current_view = View::Viewer;
                if !self.image_list.contains(&path) {
                    self.image_list.push(path.clone());
                    self.gallery.add_image(path.clone());
                }
                if let Some(idx) = self.image_list.iter().position(|p| p == &path) {
                    self.current_index = idx;
                }
                self.gallery.select_image(self.current_index);
            }
        }
    }

    pub fn open_directory(&mut self, path: PathBuf) {
        info!("打开目录: {:?}", path);

        if let Ok(entries) = std::fs::read_dir(&path) {
            let mut images: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| is_image_file(p))
                .collect();
            images.sort();
            self.gallery.clear();
            self.image_list.clear();
            for img in &images {
                self.gallery.add_image(img.clone());
            }
            self.image_list = images;
            if !self.image_list.is_empty() {
                let first = self.image_list[0].clone();
                self.open_image(first);
            }

            // 记录上次打开的目录
            self.config.set_last_opened_directory(&path);
            // 防抖保存配置
            self.config_saver.request_save(&self.config);
            info!("已记录并保存目录: {:?}", path);
        }
    }

    fn toggle_view(&mut self) {
        match self.current_view {
            View::Gallery => {
                if let Some(selected_idx) = self.gallery.selected_index() {
                    if selected_idx < self.image_list.len() {
                        self.current_index = selected_idx;
                        let path = self.image_list[self.current_index].clone();
                        self.open_image(path);
                    }
                } else if !self.image_list.is_empty() {
                    let path = self.image_list[self.current_index].clone();
                    self.open_image(path);
                }
            }
            View::Viewer => {
                self.current_view = View::Gallery;
                self.gallery.select_image(self.current_index);
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        // 如果查看器正在处理输入（如F键切换信息面板），不要处理其他快捷键
        if self.viewer.handle_input(ctx) {
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            self.toggle_view();
        }
    }

    fn handle_gallery_navigation(&mut self, ctx: &Context) {
        if self.current_view == View::Gallery {
            match self.gallery.handle_keyboard(ctx) {
                NavAction::None => {}
                NavAction::SelectAndOpen(index) => {
                    if index < self.image_list.len() {
                        self.current_index = index;
                        let path = self.image_list[index].clone();
                        self.open_image(path);
                    }
                }
            }
        }
    }
}

impl eframe::App for ImageViewerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.viewer.set_ctx(ctx.clone());
        self.gallery.init_thumbnail_loader(ctx);
        self.handle_shortcuts(ctx);
        self.handle_gallery_navigation(ctx);
        // 处理快捷键帮助面板输入
        self.shortcuts_help_panel.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| match self.current_view {
            View::Gallery => {
                if let Some(index) = self.gallery.ui(ui) {
                    if let Some(path) = self.gallery.get_image_path(index) {
                        self.open_image(path.to_path_buf());
                    }
                }
            }
            View::Viewer => {
                self.viewer.ui(ui);
            }
        });
        // 渲染快捷键帮助面板
        self.shortcuts_help_panel.ui(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        debug!("Application exiting, saving config...");
        // 立即保存确保配置被写入
        self.config_saver.save_now(&self.config);
    }
}
