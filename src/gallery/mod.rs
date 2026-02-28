//! 图库模块 - 用于显示图像缩略图
//!
//! 以可配置大小和间距的网格显示图像缩略图。

use egui::{Color32, Rect, Response, Ui, Vec2};
use tracing::{debug, error, info};

use crate::config::GalleryConfig;

/// 图库状态和渲染
pub struct Gallery {
    config: GalleryConfig,
    images: Vec<GalleryImage>,
    selected_index: Option<usize>,
}

#[derive(Clone)]
pub struct GalleryImage {
    pub path: std::path::PathBuf,
    pub thumbnail: Option<egui::TextureHandle>,
}

impl Gallery {
    /// 使用给定配置创建新的图库
    pub fn new(config: GalleryConfig) -> Self {
        debug!("初始化图库，配置: {:?}", config);
        
        Self {
            config,
            images: Vec::new(),
            selected_index: None,
        }
    }

    /// 添加图像到图库
    pub fn add_image(&mut self, path: std::path::PathBuf) {
        self.images.push(GalleryImage {
            path,
            thumbnail: None,
        });
    }

    /// 从图库中移除指定索引的图像
    pub fn remove_image(&mut self, index: usize) -> Option<std::path::PathBuf> {
        if index < self.images.len() {
            let image = self.images.remove(index);
            // 更新选中索引
            if let Some(selected) = self.selected_index {
                if selected == index {
                    self.selected_index = None;
                } else if selected > index {
                    self.selected_index = Some(selected - 1);
                }
            }
            Some(image.path)
        } else {
            None
        }
    }

    /// 获取指定索引的图像路径
    pub fn get_image_path(&self, index: usize) -> Option<&std::path::Path> {
        self.images.get(index).map(|img| img.path.as_path())
    }

    /// 获取选中的图像路径
    pub fn get_selected_path(&self) -> Option<&std::path::Path> {
        self.selected_index.and_then(|idx| self.get_image_path(idx))
    }

    /// 选中指定索引的图像
    pub fn select_image(&mut self, index: usize) -> bool {
        if index < self.images.len() {
            self.selected_index = Some(index);
            debug!("选中图像，索引: {}", index);
            true
        } else {
            false
        }
    }

    /// 清除所有图像
    pub fn clear(&mut self) {
        self.images.clear();
        self.selected_index = None;
    }

    /// 加载缩略图
    fn load_thumbnail(
        &self,
        path: &std::path::Path,
        ctx: &egui::Context,
    ) -> Option<egui::TextureHandle> {
        let thumbnail_size = self.config.thumbnail_size as u32;
        
        // 首先尝试使用 image::open 加载（自动检测格式）
        let img_result = image::open(path);
        
        // 如果失败，尝试备用方法：读取字节并猜测格式
        let img = match img_result {
            Ok(img) => img,
            Err(e) => {
                debug!("缩略图自动格式检测失败 {:?}: {}，尝试备用方法...", path, e);
                
                // 备用方法：读取文件字节
                match std::fs::read(path) {
                    Ok(data) => {
                        match image::load_from_memory(&data) {
                            Ok(img) => {
                                info!("缩略图使用备用方法成功加载: {:?}", path);
                                img
                            }
                            Err(e2) => {
                                error!("缩略图备用解码也失败 {:?}: {}", path, e2);
                                return None;
                            }
                        }
                    }
                    Err(io_err) => {
                        error!("无法读取缩略图文件 {:?}: {}", path, io_err);
                        return None;
                    }
                }
            }
        };
        
        // 调整为缩略图大小，保持宽高比
        let resized = img.resize(
            thumbnail_size,
            thumbnail_size,
            image::imageops::FilterType::Lanczos3,
        );
        
        let rgba = resized.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.as_raw();
        
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
        let texture_name = format!("thumb_{}", path.file_name()?.to_string_lossy());
        
        Some(ctx.load_texture(
            texture_name,
            color_image,
            egui::TextureOptions::LINEAR,
        ))
    }

    /// 渲染图库界面，返回点击的图像索引
    pub fn ui(
        &mut self,
        ui: &mut Ui,
    ) -> Option<usize> {
        let available_width = ui.available_width();
        let mut clicked_index: Option<usize> = None;
        
        // 基于配置计算每行项目数
        let items_per_row = if self.config.items_per_row > 0 {
            self.config.items_per_row
        } else {
            // 基于可用宽度自动计算
            let item_width = self.config.thumbnail_size as f32 + self.config.grid_spacing;
            (available_width / item_width).max(1.0) as usize
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 使用配置的网格间距
            let spacing = self.config.grid_spacing;
            ui.spacing_mut().item_spacing = Vec2::new(spacing, spacing);

            // 创建网格布局
            egui::Grid::new("gallery_grid")
                .num_columns(items_per_row)
                .spacing([spacing, spacing])
                .show(ui, |ui| {
                    for index in 0..self.images.len() {
                        let response = self.render_thumbnail(ui, index);
                        
                        if response.clicked() {
                            self.selected_index = Some(index);
                            clicked_index = Some(index);
                            debug!("选中图像，索引: {}", index);
                        }

                        // 每行items_per_row个项目后换行
                        if (index + 1) % items_per_row == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
        
        clicked_index
    }

    fn render_thumbnail(
        &mut self,
        ui: &mut Ui,
        index: usize,
    ) -> Response {
        let size = Vec2::splat(self.config.thumbnail_size as f32);
        let is_selected = self.selected_index == Some(index);
        
        // 确保图像存在
        if index >= self.images.len() {
            return ui.allocate_exact_size(size, egui::Sense::click()).1;
        }

        // 按需加载缩略图
        if self.images[index].thumbnail.is_none() {
            if let Some(texture) = self.load_thumbnail(&self.images[index].path, ui.ctx()) {
                self.images[index].thumbnail = Some(texture);
            }
        }

        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 带悬停/选中状态的背景
            let bg_color = if is_selected {
                Color32::from_rgb(52, 152, 219)  // 选中时的蓝色
            } else if response.hovered() {
                Color32::from_rgb(60, 60, 60)
            } else {
                Color32::from_rgb(40, 40, 40)
            };

            // 圆角（4px半径）
            painter.rect_filled(rect, 4.0, bg_color);

            // 选中边框
            if is_selected {
                painter.rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::WHITE));
                // 为选中项目添加微妙的阴影效果
                painter.rect_stroke(
                    rect.expand(2.0), 
                    4.0, 
                    egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(52, 152, 219, 100))
                );
            }

            // 缩略图或占位符
            if let Some(ref texture) = self.images[index].thumbnail {
                painter.image(
                    texture.id(), 
                    rect.shrink(4.0),  // 轻微内边距
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), 
                    Color32::WHITE
                );
            } else {
                // 带文件名的占位符
                let text = self.images[index]
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("未知");
                
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(12.0),
                    Color32::GRAY,
                );
            }

            // 如果启用则显示文件名标签
            if self.config.show_filenames {
                let filename = self.images[index]
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("未知");
                
                // 截断长文件名
                let display_name = if filename.len() > 20 {
                    format!("{}...", &filename[..17])
                } else {
                    filename.to_string()
                };

                painter.text(
                    rect.center_bottom() + Vec2::new(0.0, -2.0),
                    egui::Align2::CENTER_BOTTOM,
                    display_name,
                    egui::FontId::proportional(10.0),
                    Color32::LIGHT_GRAY,
                );
            }
        }

        response
    }

    /// 获取当前选中的图像索引
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// 获取图库中的图像数量
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// 检查图库是否为空
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// 获取图像列表（只读）
    pub fn images(&self) -> &[GalleryImage] {
        &self.images
    }

    /// 获取配置
    pub fn config(&self) -> &GalleryConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: GalleryConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 基础初始化测试
    // =========================================================================

    #[test]
    fn test_gallery_new() {
        let config = GalleryConfig::default();
        let gallery = Gallery::new(config);
        
        assert!(gallery.is_empty());
        assert_eq!(gallery.len(), 0);
        assert_eq!(gallery.selected_index(), None);
    }

    #[test]
    fn test_gallery_with_custom_config() {
        let config = GalleryConfig {
            thumbnail_size: 150,
            items_per_row: 6,
            grid_spacing: 16.0,
            show_filenames: true,
        };
        let gallery = Gallery::new(config);
        assert!(gallery.is_empty());
        assert_eq!(gallery.config().thumbnail_size, 150);
    }

    // =========================================================================
    // 添加图像测试
    // =========================================================================

    #[test]
    fn test_gallery_add_image() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test.png"));
        
        assert_eq!(gallery.len(), 1);
        assert!(!gallery.is_empty());
    }

    #[test]
    fn test_gallery_add_multiple_images() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.add_image(std::path::PathBuf::from("test3.png"));
        
        assert_eq!(gallery.len(), 3);
    }

    #[test]
    fn test_gallery_add_many_images() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        for i in 0..100 {
            gallery.add_image(std::path::PathBuf::from(format!("test{}.png", i)));
        }
        
        assert_eq!(gallery.len(), 100);
    }

    // =========================================================================
    // 清除测试
    // =========================================================================

    #[test]
    fn test_gallery_clear() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.clear();
        
        assert!(gallery.is_empty());
        assert_eq!(gallery.selected_index(), None);
    }

    #[test]
    fn test_gallery_clear_empty() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.clear();
        
        assert!(gallery.is_empty());
        assert_eq!(gallery.len(), 0);
    }

    #[test]
    fn test_gallery_clear_with_selection() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.select_image(0);
        assert_eq!(gallery.selected_index(), Some(0));
        
        gallery.clear();
        assert_eq!(gallery.selected_index(), None);
    }

    // =========================================================================
    // 选中测试
    // =========================================================================

    #[test]
    fn test_gallery_select_image() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        
        assert!(gallery.select_image(0));
        assert_eq!(gallery.selected_index(), Some(0));
        
        assert!(gallery.select_image(1));
        assert_eq!(gallery.selected_index(), Some(1));
    }

    #[test]
    fn test_gallery_select_invalid_index() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        
        assert!(!gallery.select_image(5));
        assert_eq!(gallery.selected_index(), None);
    }

    #[test]
    fn test_gallery_select_empty() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        assert!(!gallery.select_image(0));
        assert_eq!(gallery.selected_index(), None);
    }

    #[test]
    fn test_gallery_select_then_add() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.select_image(0);
        
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        // 选中状态应保持
        assert_eq!(gallery.selected_index(), Some(0));
    }

    // =========================================================================
    // 获取路径测试
    // =========================================================================

    #[test]
    fn test_gallery_get_image_path() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("/path/to/test.png"));
        
        let path = gallery.get_image_path(0);
        assert!(path.is_some());
        assert_eq!(path.unwrap().to_str().unwrap(), "/path/to/test.png");
    }

    #[test]
    fn test_gallery_get_image_path_invalid() {
        let config = GalleryConfig::default();
        let gallery = Gallery::new(config);
        
        assert!(gallery.get_image_path(0).is_none());
    }

    #[test]
    fn test_gallery_get_selected_path() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("/path/to/selected.png"));
        gallery.add_image(std::path::PathBuf::from("/path/to/other.png"));
        gallery.select_image(0);
        
        let path = gallery.get_selected_path();
        assert!(path.is_some());
        assert!(path.unwrap().to_str().unwrap().contains("selected"));
    }

    #[test]
    fn test_gallery_get_selected_path_none() {
        let config = GalleryConfig::default();
        let gallery = Gallery::new(config);
        
        assert!(gallery.get_selected_path().is_none());
    }

    // =========================================================================
    // 移除图像测试
    // =========================================================================

    #[test]
    fn test_gallery_remove_image() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        
        let removed = gallery.remove_image(0);
        assert!(removed.is_some());
        assert_eq!(gallery.len(), 1);
    }

    #[test]
    fn test_gallery_remove_invalid_index() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        
        let removed = gallery.remove_image(5);
        assert!(removed.is_none());
        assert_eq!(gallery.len(), 1);
    }

    #[test]
    fn test_gallery_remove_selected() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.select_image(0);
        
        gallery.remove_image(0);
        // 选中应被清除
        assert_eq!(gallery.selected_index(), None);
    }

    #[test]
    fn test_gallery_remove_after_selected() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.add_image(std::path::PathBuf::from("test3.png"));
        gallery.select_image(2);
        
        gallery.remove_image(0);
        // 选中索引应减1
        assert_eq!(gallery.selected_index(), Some(1));
    }

    // =========================================================================
    // 更新配置测试
    // =========================================================================

    #[test]
    fn test_gallery_update_config() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        let new_config = GalleryConfig {
            thumbnail_size: 200,
            items_per_row: 8,
            grid_spacing: 20.0,
            show_filenames: false,
        };
        
        gallery.update_config(new_config);
        assert_eq!(gallery.config().thumbnail_size, 200);
        assert_eq!(gallery.config().items_per_row, 8);
    }

    #[test]
    fn test_gallery_update_config_preserves_images() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.select_image(0);
        
        let new_config = GalleryConfig {
            thumbnail_size: 200,
            ..Default::default()
        };
        gallery.update_config(new_config);
        
        assert_eq!(gallery.len(), 2);
        assert_eq!(gallery.selected_index(), Some(0));
    }

    // =========================================================================
    // 边界条件测试
    // =========================================================================

    #[test]
    fn test_gallery_add_duplicate_paths() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        // 允许重复路径
        gallery.add_image(std::path::PathBuf::from("test.png"));
        gallery.add_image(std::path::PathBuf::from("test.png"));
        
        assert_eq!(gallery.len(), 2);
    }

    #[test]
    fn test_gallery_images_accessor() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        
        let images = gallery.images();
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_gallery_select_last() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        gallery.add_image(std::path::PathBuf::from("test3.png"));
        
        assert!(gallery.select_image(2));
        assert_eq!(gallery.selected_index(), Some(2));
    }

    #[test]
    fn test_gallery_remove_all() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        gallery.add_image(std::path::PathBuf::from("test1.png"));
        gallery.add_image(std::path::PathBuf::from("test2.png"));
        
        gallery.remove_image(0);
        gallery.remove_image(0);
        
        assert!(gallery.is_empty());
        assert_eq!(gallery.selected_index(), None);
    }

    #[test]
    fn test_gallery_path_types() {
        let config = GalleryConfig::default();
        let mut gallery = Gallery::new(config);
        
        // 测试各种路径类型
        gallery.add_image(std::path::PathBuf::from("/absolute/path.png"));
        gallery.add_image(std::path::PathBuf::from("relative/path.png"));
        gallery.add_image(std::path::PathBuf::from("file.with.dots.png"));
        gallery.add_image(std::path::PathBuf::from("unicode_文件.png"));
        
        assert_eq!(gallery.len(), 4);
    }
}
