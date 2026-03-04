//! Gallery Widget - 画廊 UI 组件

use crate::adapters::egui::thumbnail_loader::ThumbnailCache;
use crate::core::use_cases::GalleryState;
use egui::{Color32, Response, Ui, Vec2};

/// 画廊组件
pub struct GalleryWidget {
    items_per_row: usize,
    thumbnail_cache: ThumbnailCache,
}

impl Default for GalleryWidget {
    fn default() -> Self {
        Self {
            items_per_row: 4,
            thumbnail_cache: ThumbnailCache::default(),
        }
    }
}

impl GalleryWidget {
    /// 初始化缩略图加载器
    pub fn init(&mut self, ctx: &egui::Context) {
        self.thumbnail_cache.init(ctx);
    }

    /// 渲染画廊
    pub fn ui(&mut self, ui: &mut Ui, state: &GalleryState) -> Option<usize> {
        let available_width = ui.available_width();
        let mut clicked_index: Option<usize> = None;

        // 计算每行项目数
        self.items_per_row = state.layout.calculate_items_per_row(available_width);

        // 调整缩略图缓存大小
        let image_count = state.gallery.images().len();
        self.thumbnail_cache.resize(image_count);

        // 处理异步加载的缩略图结果
        self.thumbnail_cache.process_results();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let spacing = state.layout.grid_spacing;
                ui.spacing_mut().item_spacing = Vec2::new(spacing, spacing);

                egui::Grid::new("gallery_grid")
                    .num_columns(self.items_per_row)
                    .spacing([spacing, spacing])
                    .show(ui, |ui| {
                        for (index, image) in state.gallery.images().iter().enumerate() {
                            let is_selected = state.gallery.selected_index() == Some(index);

                            // 请求加载缩略图（如果还没加载）
                            self.thumbnail_cache.request_thumbnail(index, image.path());

                            let response = self.render_thumbnail(
                                ui,
                                image,
                                is_selected,
                                state.layout.thumbnail_size,
                                index,
                            );

                            if response.clicked() {
                                clicked_index = Some(index);
                            }

                            if (index + 1) % self.items_per_row == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });

        clicked_index
    }

    /// 渲染缩略图
    fn render_thumbnail(
        &self,
        ui: &mut Ui,
        image: &crate::core::domain::Image,
        is_selected: bool,
        size: u32,
        index: usize,
    ) -> Response {
        let size_vec = Vec2::splat(size as f32);
        let (rect, response) = ui.allocate_exact_size(size_vec, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 背景色
            let bg_color = if is_selected {
                Color32::from_rgb(52, 152, 219)
            } else if response.hovered() {
                Color32::from_rgb(60, 60, 60)
            } else {
                Color32::from_rgb(40, 40, 40)
            };

            painter.rect_filled(rect, 4.0, bg_color);

            // 选中边框
            if is_selected {
                painter.rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Outside);
            }

            // 尝试渲染缩略图
            if let Some(texture) = self.thumbnail_cache.get(index) {
                // 有缩略图，渲染它
                let texture_size = texture.size_vec2();
                let scale = (size as f32 / texture_size.x.max(texture_size.y)).min(1.0);
                let display_size = texture_size * scale * 0.9; // 留出边距

                let center = rect.center();
                let image_rect = Rect::from_center_size(center, display_size);

                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                // 显示文件名作为占位符
                let filename = image.file_name().unwrap_or("Unknown");
                let display_name = if filename.len() > 20 {
                    format!("{}...", &filename[..17])
                } else {
                    filename.to_string()
                };

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    display_name,
                    egui::FontId::proportional(12.0),
                    Color32::GRAY,
                );

                // 如果正在加载，显示加载提示
                if index < self.thumbnail_cache.loading.len() && self.thumbnail_cache.loading[index] {
                    painter.text(
                        rect.center() + Vec2::new(0.0, 20.0),
                        egui::Align2::CENTER_CENTER,
                        "加载中...",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(100, 100, 100),
                    );
                }
            }
        }

        response
    }
}

use egui::Rect;
