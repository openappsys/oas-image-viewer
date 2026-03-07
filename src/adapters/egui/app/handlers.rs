//! 事件处理模块

use super::types::EguiApp;
use crate::core::domain::{is_image_file, NavigationDirection, ViewMode};
use crate::core::ports::FileDialogPort;

use egui::Context;
use std::path::{Path, PathBuf};

impl EguiApp {
    /// 处理文件对话框打开
    pub(crate) fn handle_open_dialog(&mut self) {
        let dialog = crate::infrastructure::RfdFileDialog::new();
        if let Some(paths) = dialog.open_files() {
            for path in paths {
                self.add_image_to_gallery(&path);
                self.pending_files.push(path);
            }
        }
    }

    fn add_image_to_gallery(&mut self, path: &Path) {
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let _ = self.service.update_state(|state| {
            let image = crate::core::domain::Image::new(file_name, path.to_path_buf());
            state.gallery.gallery.add_image(image);
        });
    }

    /// 处理待处理文件
    pub(crate) fn process_pending_files(&mut self, ctx: &Context) {
        let rect = ctx.viewport_rect();
        let win_w = rect.width();
        let win_h = rect.height();

        while let Some(path) = self.pending_files.pop() {
            self.process_single_file(ctx, &path, win_w, win_h);
        }
    }

    fn process_single_file(&mut self, ctx: &Context, path: &Path, win_w: f32, win_h: f32) {
        let path_str = path.to_string_lossy().to_string();
        let load_result = self.load_image_with_data(ctx, path);

        let fit_to_window = self
            .service
            .get_state()
            .map(|s| s.config.viewer.fit_to_window)
            .unwrap_or(true);

        let _ = self.service.update_state(|state| {
            let _ = self.service.view_use_case.open_image(
                path,
                &mut state.view,
                Some(win_w),
                Some(win_h),
                fit_to_window,
            );
        });

        self.update_texture_cache(load_result, path_str);
    }

    fn update_texture_cache(
        &mut self,
        load_result: anyhow::Result<(egui::TextureHandle, usize, usize, Vec<u8>)>,
        path_str: String,
    ) {
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

    /// 加载图像纹理和数据
    pub(crate) fn load_image_with_data(
        &self,
        ctx: &Context,
        path: &std::path::Path,
    ) -> anyhow::Result<(egui::TextureHandle, usize, usize, Vec<u8>)> {
        let img = image::ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let rgba_data = rgba.into_raw();

        let image_data =
            egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba_data);

        let texture_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let texture = ctx.load_texture(texture_name, image_data, egui::TextureOptions::LINEAR);

        Ok((texture, width as usize, height as usize, rgba_data))
    }

    /// 加载并设置当前图像
    pub(crate) fn load_and_set_image(&mut self, ctx: &Context, path: &std::path::Path) {
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
    pub(crate) fn handle_drops(&mut self, ctx: &Context) {
        self.drag_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        ctx.input(|i| {
            if i.raw.dropped_files.is_empty() {
                return;
            }

            let image_paths: Vec<PathBuf> = i
                .raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| is_image_file(p))
                .collect();

            if !image_paths.is_empty() {
                self.process_dropped_images(&image_paths);
            }
        });
    }

    fn process_dropped_images(&mut self, image_paths: &[PathBuf]) {
        for path in image_paths {
            self.add_image_to_gallery(path);
        }

        if let Some(first_path) = image_paths.first() {
            self.pending_files.push(first_path.to_path_buf());
        }

        self.drag_hovering = false;
    }

    /// 导航并打开图片
    pub(crate) fn navigate_and_open(&mut self, ctx: &Context, direction: NavigationDirection) {
        let index = self.service.get_state().ok().and_then(|mut state| {
            self.service
                .navigate_use_case
                .navigate(&mut state.gallery, direction)
        });

        if let Some(idx) = index {
            self.open_navigated_image(ctx, Some(idx));
        }
    }

    fn open_navigated_image(&mut self, ctx: &Context, index: Option<usize>) {
        let Some(idx) = index else { return };

        let Ok(state) = self.service.get_state() else {
            return;
        };
        if state.view.view_mode != ViewMode::Viewer {
            return;
        }

        let Some(image) = state.gallery.gallery.get_image(idx) else {
            return;
        };

        let path = image.path().to_path_buf();
        self.open_image(ctx, &path, state.config.viewer.fit_to_window);
    }

    /// 打开图片
    pub(crate) fn open_image(
        &mut self,
        ctx: &Context,
        path: &std::path::Path,
        fit_to_window: bool,
    ) {
        self.load_and_set_image(ctx, path);

        let rect = ctx.viewport_rect();

        let _ = self.service.update_state(|state| {
            let _ = self.service.view_use_case.open_image(
                path,
                &mut state.view,
                Some(rect.width()),
                Some(rect.height()),
                fit_to_window,
            );
        });
    }

    /// 处理放大
    pub(crate) fn handle_zoom_in(&mut self) {
        let _ = self.service.update_state(|state| {
            let max = state.config.viewer.max_scale;
            self.service
                .view_use_case
                .zoom_in(&mut state.view, 1.25, max);
        });
    }

    /// 处理缩小
    pub(crate) fn handle_zoom_out(&mut self) {
        let _ = self.service.update_state(|state| {
            let min = state.config.viewer.min_scale;
            self.service
                .view_use_case
                .zoom_out(&mut state.view, 1.25, min);
        });
    }
}
