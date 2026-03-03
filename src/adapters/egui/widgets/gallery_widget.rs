//! Gallery Widget - 画廊 UI 组件

use crate::core::use_cases::GalleryState;
use egui::{Color32, Response, Ui, Vec2};

/// 画廊组件
#[derive(Default)]
pub struct GalleryWidget {
    items_per_row: usize,
}

impl GalleryWidget {
    /// 渲染画廊
    pub fn ui(&mut self, ui: &mut Ui, state: &GalleryState) -> Option<usize> {
        let available_width = ui.available_width();
        let mut clicked_index: Option<usize> = None;

        // 计算每行项目数
        self.items_per_row = state.layout.calculate_items_per_row(available_width);

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
                            let response = self.render_thumbnail(
                                ui,
                                image,
                                is_selected,
                                state.layout.thumbnail_size,
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
                painter.rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::WHITE));
            }

            // 文件名占位符
            if let Some(filename) = image.file_name() {
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
            }
        }

        response
    }
}
