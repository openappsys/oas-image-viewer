//! 事件处理模块

use super::types::EguiApp;
use crate::core::domain::{is_image_file, NavigationDirection};

use egui::Context;
use std::path::{Path, PathBuf};

impl EguiApp {
    /// 处理文件对话框打开
    pub(crate) fn handle_open_dialog(&mut self) {
        if let Some(paths) = self.file_dialog.open_files() {
            tracing::debug!(count = paths.len(), "打开文件对话框选择了文件");
            for path in paths {
                if path.exists() {
                    tracing::debug!(path = ?path, "添加图片");
                    self.add_image_to_gallery(&path);
                    self.session_state.pending_files.push(path);
                } else {
                    tracing::warn!(path = ?path, "文件路径无效");
                }
            }
        } else {
            tracing::debug!("文件对话框被取消或未选择文件");
        }
    }

    pub(crate) fn handle_open_directory_dialog(&mut self) {
        let Some(path) = self.file_dialog.open_directory() else {
            tracing::debug!("目录对话框被取消或未选择目录");
            return;
        };

        if let Err(e) = self
            .service
            .load_directory(self.image_source.as_ref(), &path)
        {
            tracing::error!(path = ?path, error = %e, "扫描目录失败");
            return;
        }

        if let Err(e) = self.set_last_opened_directory_and_save(path) {
            tracing::error!(error = %e, "更新最近目录失败");
        }
    }

    pub(crate) fn add_image_to_gallery(&mut self, path: &Path) {
        if let Err(e) = self.service.add_image_to_gallery(path) {
            tracing::error!(error = %e, "添加图片到图库失败");
        }
    }

    /// 处理待处理文件
    pub(crate) fn process_pending_files(&mut self, ctx: &Context) {
        let rect = ctx.viewport_rect();
        let win_w = rect.width();
        let win_h = rect.height();

        // 处理通过界面加入的文件（拖放、文件对话框）
        while let Some(path) = self.session_state.pending_files.pop() {
            self.process_single_file(ctx, &path, win_w, win_h);
        }

        // 在 macOS 上，还要检查通过 Apple Event 收到的文件
        // （例如“打开方式”菜单或双击已关联文件）
        #[cfg(target_os = "macos")]
        {
            for path in crate::adapters::macos_file_open::get_pending_files() {
                tracing::info!("处理 macOS Apple Event 文件: {:?}", path);
                self.add_image_to_gallery(&path);
                self.process_single_file(ctx, &path, win_w, win_h);
            }
        }
    }

    pub(crate) fn process_single_file(
        &mut self,
        ctx: &Context,
        path: &Path,
        win_w: f32,
        win_h: f32,
    ) {
        let path_str = path.to_string_lossy().to_string();
        let load_result = self.load_image_with_data(ctx, path);

        let fit_to_window = self.service.is_fit_to_window_enabled().unwrap_or(true);

        if let Err(e) =
            self.service
                .open_image(path, Some(win_w), Some(win_h), fit_to_window)
        {
            tracing::error!(path = %path.display(), error = %e, "更新查看状态失败");
        }

        self.update_texture_cache(load_result, path_str);
    }

    fn update_texture_cache(
        &mut self,
        load_result: anyhow::Result<(egui::TextureHandle, usize, usize, Vec<u8>)>,
        path_str: String,
    ) {
        match load_result {
            Ok((texture, width, height, rgba_data)) => {
                self.reset_readonly_transform();
                self.current_texture = Some((path_str.clone(), texture));
                self.current_texture_data = Some((width, height, rgba_data));
            }
            Err(e) => {
                tracing::error!(path = %path_str, error = %e, "加载图片纹理失败");
                self.reset_readonly_transform();
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
                self.reset_readonly_transform();
                self.current_texture = Some((path.to_string_lossy().to_string(), texture));
                self.current_texture_data = Some((width, height, rgba_data));
            }
            Err(e) => {
                tracing::error!(path = %path.display(), error = %e, "加载图片失败");
                self.reset_readonly_transform();
                self.current_texture = None;
                self.current_texture_data = None;
            }
        }
    }

    /// 处理拖放
    pub(crate) fn handle_drops(&mut self, ctx: &Context) {
        self.ui_state.drag_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

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
            self.session_state.pending_files.push(first_path.to_path_buf());
            tracing::debug!(path = %first_path.display(), "拖放添加图片");
        }

        self.ui_state.drag_hovering = false;
    }

    /// 导航并打开图片
    pub(crate) fn navigate_and_open(&mut self, ctx: &Context, direction: NavigationDirection) {
        self.bump_slideshow_timer();
        match self.service.navigate_gallery(direction) {
            Ok(index) => self.open_navigated_image(ctx, index),
            Err(e) => {
                tracing::error!(error = %e, "导航失败");
            }
        }
    }

    fn open_navigated_image(&mut self, ctx: &Context, index: Option<usize>) {
        let Some(idx) = index else { return };

        let Ok(Some((path, fit_to_window))) =
            self.service.get_gallery_image_path_and_fit_if_viewer(idx)
        else {
            return;
        };

        self.open_image(ctx, &path, fit_to_window);
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
        if let Err(e) = self.service.open_image(
            path,
            Some(rect.width()),
            Some(rect.height()),
            fit_to_window,
        ) {
            tracing::error!(path = %path.display(), error = %e, "打开图片失败");
        }
        self.bump_slideshow_timer();
    }

    /// 处理放大
    pub(crate) fn handle_zoom_in(&mut self) {
        if let Err(e) = self.service.zoom_in(1.25) {
            tracing::error!(error = %e, "放大失败");
        }
    }

    /// 处理缩小
    pub(crate) fn handle_zoom_out(&mut self) {
        if let Err(e) = self.service.zoom_out(1.25) {
            tracing::error!(error = %e, "缩小失败");
        }
    }

    /// 重置缩放（100%原始尺寸）
    pub(crate) fn handle_reset_zoom(&mut self) {
        if let Err(e) = self.service.reset_zoom() {
            tracing::error!(error = %e, "重置缩放失败");
        }
    }

    /// 适应窗口
    pub(crate) fn handle_fit_to_window(&mut self, ctx: &Context) {
        let rect = ctx.viewport_rect();
        if let Err(e) = self.service.fit_to_window(rect.width(), rect.height()) {
            tracing::error!(error = %e, "适应窗口失败");
        }
    }

    pub(crate) fn handle_fit_to_width(&mut self, ctx: &Context) {
        let rect = ctx.viewport_rect();
        if let Err(e) = self.service.fit_to_width(rect.width()) {
            tracing::error!(error = %e, "适应宽度失败");
        }
    }

    pub(crate) fn handle_fit_to_height(&mut self, ctx: &Context) {
        let rect = ctx.viewport_rect();
        if let Err(e) = self.service.fit_to_height(rect.height()) {
            tracing::error!(error = %e, "适应高度失败");
        }
    }
}
