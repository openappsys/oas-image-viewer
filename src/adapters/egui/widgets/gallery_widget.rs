//! 画廊组件 - 画廊界面组件

use crate::adapters::egui::i18n::{format_thumbnail_hint, get_text};
use crate::adapters::egui::thumbnail_loader::ThumbnailCache;
use crate::core::domain::Language;
use crate::core::use_cases::GalleryState;
use egui::{Color32, Response, Sense, Ui, Vec2};
use std::time::{Duration, Instant};

/// 缩略图大小提示显示时间
const SIZE_HINT_DURATION: Duration = Duration::from_millis(800);

/// 画廊组件
pub struct GalleryWidget {
    items_per_row: usize,
    thumbnail_cache: ThumbnailCache,
    /// 大小提示显示结束时间
    size_hint_until: Option<Instant>,
    /// 当前显示的缩略图大小
    last_thumbnail_size: u32,
}

impl Default for GalleryWidget {
    fn default() -> Self {
        Self {
            items_per_row: 4,
            thumbnail_cache: ThumbnailCache::default(),
            size_hint_until: None,
            last_thumbnail_size: 100,
        }
    }
}

impl GalleryWidget {
    /// 初始化缩略图加载器
    pub fn init(&mut self, ctx: &egui::Context) {
        self.thumbnail_cache.init(ctx);
    }

    /// 处理滚轮事件来调整缩略图大小
    /// 返回 true 如果缩略图大小发生了变化
    pub fn handle_scroll(
        &mut self,
        ctx: &egui::Context,
        current_size: u32,
        _language: Language,
    ) -> Option<u32> {
        let ctrl_pressed = ctx.input(|i| i.modifiers.ctrl);
        let scroll_delta = ctx.input(|i| i.raw_scroll_delta.y);
        let smooth_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        let total_delta = if scroll_delta != 0.0 {
            scroll_delta
        } else {
            smooth_delta
        };

        if !ctrl_pressed {
            return None;
        }

        if total_delta == 0.0 {
            return None;
        }

        let new_size = if total_delta > 0.0 {
            // 向上滚动 - 放大
            (current_size + 10).min(200)
        } else {
            // 向下滚动 - 缩小
            current_size.saturating_sub(10).max(60)
        };

        if new_size != current_size {
            // 显示大小提示
            self.size_hint_until = Some(Instant::now() + SIZE_HINT_DURATION);
            self.last_thumbnail_size = new_size;
            Some(new_size)
        } else {
            None
        }
    }

    /// 渲染缩略图大小提示
    fn render_size_hint(&self, ctx: &egui::Context, language: Language) {
        let Some(until) = self.size_hint_until else {
            return;
        };

        if Instant::now() > until {
            return;
        }

        let text = format_thumbnail_hint(self.last_thumbnail_size, language);
        let screen_rect = ctx.viewport_rect();

        // 在右下角显示提示
        let pos = egui::pos2(screen_rect.right() - 20.0, screen_rect.bottom() - 20.0);

        egui::Area::new(egui::Id::new("thumbnail_size_hint"))
            .fixed_pos(pos)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let bg_color = Color32::from_rgba_premultiplied(0, 0, 0, 180);
                let text_color = Color32::WHITE;

                let text_style = egui::FontId::proportional(14.0);
                let text_size = ui
                    .painter()
                    .layout(text.clone(), text_style.clone(), text_color, f32::INFINITY)
                    .size();

                let padding = egui::vec2(12.0, 6.0);
                let rect = egui::Rect::from_min_size(ui.min_rect().min, text_size + padding * 2.0);

                ui.painter().rect_filled(rect, 6.0, bg_color);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    text_style,
                    text_color,
                );
            });
    }

    /// 渲染画廊
    ///
    /// # 返回值
    /// 返回被点击图片的索引，如果没有则返回 None
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        state: &GalleryState,
        ctx: &egui::Context,
        language: Language,
    ) -> Option<usize> {
        let available_width = ui.available_width();
        let mut clicked_index: Option<usize> = None;

        // 如果没有图片，显示空状态提示
        let image_count = state.gallery.images().len();
        if image_count == 0 {
            let available_size = ui.available_size();
            let (rect, _) = ui.allocate_exact_size(available_size, Sense::click());
            let empty_text = get_text("empty_gallery", language);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                empty_text,
                egui::FontId::proportional(18.0),
                Color32::GRAY,
            );
            return None;
        }

        // 计算每行项目数
        self.items_per_row = state.layout.calculate_items_per_row(available_width);

        // 调整缩略图缓存大小
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
                                language,
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

        // 渲染大小提示（在画廊 UI 渲染后）
        self.render_size_hint(ctx, language);

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
        language: Language,
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
                painter.rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0, Color32::WHITE),
                    egui::StrokeKind::Outside,
                );
            }

            // 尝试渲染缩略图
            if let Some(texture) = self.thumbnail_cache.get(index) {
                // 有缩略图，渲染它
                let texture_size = texture.size_vec2();
                let max_side = texture_size.x.max(texture_size.y).max(1.0);
                let scale = size as f32 / max_side;
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
                let filename = image
                    .file_name()
                    .unwrap_or_else(|| get_text("unknown", language));
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
                if index < self.thumbnail_cache.loading.len() && self.thumbnail_cache.loading[index]
                {
                    let loading_text = get_text("loading", language);
                    painter.text(
                        rect.center() + Vec2::new(0.0, 20.0),
                        egui::Align2::CENTER_CENTER,
                        loading_text,
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
