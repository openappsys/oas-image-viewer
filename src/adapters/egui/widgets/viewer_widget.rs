//! Viewer Widget - 查看器 UI 组件

use crate::adapters::clipboard::ClipboardManager;
use crate::core::domain::Scale;
use crate::core::domain::ViewerSettings;
use crate::core::ports::ClipboardPort;
use crate::core::use_cases::ViewState;
use egui::{Color32, Rect, Sense, Ui, Vec2};

/// 查看器组件
#[derive(Default)]
pub struct ViewerWidget {
    dragging: bool,
    clipboard: ClipboardManager,
}

impl ViewerWidget {
    /// 渲染查看器
    /// 返回 是否双击全屏
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        state: &mut ViewState,
        settings: &ViewerSettings,
        texture: Option<&(String, egui::TextureHandle)>,
    ) -> bool {
        let available_size = ui.available_size();
        let bg_color = Color32::from_rgb(
            settings.background_color.r,
            settings.background_color.g,
            settings.background_color.b,
        );

        let (rect, response) = ui.allocate_exact_size(available_size, Sense::click_and_drag());
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // 处理双击全屏
        let double_clicked = ui.input(|i| {
            i.pointer
                .button_double_clicked(egui::PointerButton::Primary)
        });

        // 处理拖拽平移（左键拖拽）- 直接修改 state
        let hovered = response.hovered();
        if response.dragged() {
            self.dragging = true;
            state.offset.x += response.drag_delta().x;
            state.offset.y += response.drag_delta().y;
        } else {
            self.dragging = false;
        }

        // 处理滚轮缩放 - 直接修改 state
        if hovered && !self.dragging {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = 1.0 + scroll_delta * 0.001;
                let current_scale = state.scale.value();
                let new_scale =
                    (current_scale * zoom_factor).clamp(settings.min_scale, settings.max_scale);

                if new_scale != current_scale {
                    // 以鼠标为中心缩放，调整 offset
                    if let Some(mouse) = ui.input(|i| i.pointer.hover_pos()) {
                        let center = rect.center();
                        let current_offset = Vec2::new(state.offset.x, state.offset.y);
                        let zoom_center = mouse - center - current_offset;
                        let new_offset =
                            current_offset - zoom_center * (new_scale / current_scale - 1.0);
                        state.offset.x = new_offset.x;
                        state.offset.y = new_offset.y;
                    }
                    state.scale = Scale::new(new_scale, settings.min_scale, settings.max_scale);
                    state.user_zoomed = true;
                }
            }
        }

        // 渲染图像或占位符
        // 右键菜单
        response.context_menu(|ui| {
            ui.set_min_width(150.0);

            let has_image = state.current_image.is_some();
            let clipboard_available = self.clipboard.is_available();

            // 复制图片
            let copy_image_btn = ui.add_enabled(
                has_image && clipboard_available,
                egui::Button::new("📋 复制图片"),
            );
            if copy_image_btn.clicked() {
                if let Some(ref image) = state.current_image {
                    let path = image.path();
                    let _ = self.clipboard.copy_image_from_file(path);
                }
                ui.close();
            }

            // 复制文件路径
            let copy_path_btn = ui.add_enabled(
                has_image && clipboard_available,
                egui::Button::new("📋 复制文件路径"),
            );
            if copy_path_btn.clicked() {
                if let Some(ref image) = state.current_image {
                    let path = image.path();
                    let _ = self.clipboard.copy_path(path);
                }
                ui.close();
            }

            ui.separator();

            // 在文件夹中显示
            let show_in_folder_btn =
                ui.add_enabled(has_image, egui::Button::new("📁 在文件夹中显示"));
            if show_in_folder_btn.clicked() {
                if let Some(ref image) = state.current_image {
                    let path = image.path();
                    let _ = ClipboardPort::show_in_folder(&self.clipboard, path);
                }
                ui.close();
            }
        });

        if let Some(ref image) = state.current_image {
            self.render_image(ui, image, state, rect, settings, texture);
        } else {
            // 无图像占位符
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "🖼 未选择图像\n按 Ctrl+O 打开图像或从图库中选择\n也可以直接拖拽图像到窗口",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
        }

        // 渲染缩放控制面板
        self.render_zoom_controls(ui, rect, state, settings);

        // 渲染尺寸指示器
        self.render_dimensions_indicator(ui, rect, state);

        double_clicked
    }

    /// 渲染图像
    fn render_image(
        &self,
        ui: &mut Ui,
        image: &crate::core::domain::Image,
        state: &ViewState,
        rect: Rect,
        _settings: &ViewerSettings,
        texture: Option<&(String, egui::TextureHandle)>,
    ) {
        // 如果有纹理，渲染实际图像
        if let Some((_, texture_handle)) = texture {
            // 计算缩放后的图像尺寸
            let img_size = texture_handle.size_vec2();
            let scale = state.scale.value();
            let scaled_size = img_size * scale;

            // 计算居中位置（考虑偏移）
            let center = rect.center() + Vec2::new(state.offset.x, state.offset.y);
            let image_rect = Rect::from_center_size(center, scaled_size);

            // 设置裁剪区域
            ui.set_clip_rect(rect);

            // 渲染图像纹理
            ui.painter().image(
                texture_handle.id(),
                image_rect,
                Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // 纹理加载中或失败，显示文件名作为占位
            let center = rect.center();
            let text = format!(
                "{}\n{}x{}",
                image.file_name().unwrap_or("Unknown"),
                image.metadata().width,
                image.metadata().height,
            );

            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        }

        // 缩放指示
        if state.user_zoomed {
            let zoom_text = format!("{:.0}%", state.scale.percentage());
            ui.painter().text(
                rect.center() + Vec2::new(0.0, 30.0),
                egui::Align2::CENTER_CENTER,
                zoom_text,
                egui::FontId::proportional(12.0),
                Color32::GRAY,
            );
        }
    }

    /// 渲染缩放控制面板
    fn render_zoom_controls(
        &self,
        ui: &mut Ui,
        rect: Rect,
        state: &mut ViewState,
        settings: &ViewerSettings,
    ) {
        let scale = state.scale.value();
        let is_100_percent = (scale - 1.0).abs() < 0.001;

        // 按钮大小定义
        let small_btn_size = Vec2::new(20.0, 20.0); // −/+
        let large_btn_size = Vec2::new(40.0, 20.0); // 1:1
        let percent_width = 32.0; // 百分比显示宽度
        let spacing = 4.0; // 按钮间距

        // 计算整体面板尺寸（水平单行布局）
        // [−] (20) + 间距 (4) + [百分比] (40) + 间距 (4) + [+] (20) + 间距 (4) + [1:1] (40)
        // 总宽度: 20 + 4 + 40 + 4 + 20 + 4 + 40 = 132
        // 总高度: 20
        let panel_width = small_btn_size.x + spacing + percent_width + spacing + small_btn_size.x + spacing + large_btn_size.x;
        let panel_height = 20.0;

        // 计算面板位置（右下角，向左偏移）
        let panel_pos = rect.right_bottom() - Vec2::new(10.0 + panel_width, 10.0 + panel_height);

        // 半透明黑色背景
        let bg_rect = Rect::from_min_size(panel_pos, Vec2::new(panel_width, panel_height));
        ui.painter().rect_filled(
            bg_rect,
            8.0, // 圆角
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        // ===== 水平布局：[−] [百分比] [+] [1:1] =====
        let mut current_x = panel_pos.x;
        let btn_y = panel_pos.y;

        // − 按钮
        let btn_minus_pos = egui::Pos2::new(current_x, btn_y);
        let btn_minus_rect = Rect::from_min_size(btn_minus_pos, small_btn_size);
        let btn_minus_response = ui.interact(btn_minus_rect, ui.id().with("zoom_minus"), Sense::click());

        let btn_minus_bg = if btn_minus_response.hovered() {
            Color32::from_rgba_premultiplied(60, 60, 60, 220)
        } else {
            Color32::from_rgba_premultiplied(40, 40, 40, 200)
        };

        ui.painter().rect_filled(btn_minus_rect, 10.0, btn_minus_bg);
        ui.painter().text(
            btn_minus_rect.center(),
            egui::Align2::CENTER_CENTER,
            "−",
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        // − 按钮点击处理（缩小，因子 1/1.25）
        if btn_minus_response.clicked() {
            let new_scale = (scale / 1.25).clamp(settings.min_scale, settings.max_scale);
            if new_scale != scale {
                state.scale = Scale::new(new_scale, settings.min_scale, settings.max_scale);
                state.user_zoomed = true;
            }
        }

        // 百分比显示
        current_x += small_btn_size.x + spacing;
        let percent_pos = egui::Pos2::new(current_x, btn_y);
        let percent_rect = Rect::from_min_size(percent_pos, Vec2::new(percent_width, small_btn_size.y));

        let zoom_text = format!("{:.0}%", state.scale.percentage());
        ui.painter().text(
            percent_rect.center(),
            egui::Align2::CENTER_CENTER,
            zoom_text,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // + 按钮
        current_x += percent_width + spacing;
        let btn_plus_pos = egui::Pos2::new(current_x, btn_y);
        let btn_plus_rect = Rect::from_min_size(btn_plus_pos, small_btn_size);
        let btn_plus_response = ui.interact(btn_plus_rect, ui.id().with("zoom_plus"), Sense::click());

        let btn_plus_bg = if btn_plus_response.hovered() {
            Color32::from_rgba_premultiplied(60, 60, 60, 220)
        } else {
            Color32::from_rgba_premultiplied(40, 40, 40, 200)
        };

        ui.painter().rect_filled(btn_plus_rect, 10.0, btn_plus_bg);
        ui.painter().text(
            btn_plus_rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        // + 按钮点击处理（放大，因子 1.25）
        if btn_plus_response.clicked() {
            let new_scale = (scale * 1.25).clamp(settings.min_scale, settings.max_scale);
            if new_scale != scale {
                state.scale = Scale::new(new_scale, settings.min_scale, settings.max_scale);
                state.user_zoomed = true;
            }
        }

        // 1:1 按钮
        current_x += small_btn_size.x + spacing;
        let btn_1_1_pos = egui::Pos2::new(current_x, btn_y);
        let btn_1_1_rect = Rect::from_min_size(btn_1_1_pos, large_btn_size);

        // 1:1 按钮交互区域
        let btn_1_1_response = ui.interact(btn_1_1_rect, ui.id().with("zoom_1_1"), Sense::click());

        // 按钮背景色（非100%时高亮，100%时变灰）
        let btn_1_1_bg = if is_100_percent {
            Color32::from_rgba_premultiplied(80, 80, 80, 180)
        } else if btn_1_1_response.hovered() {
            Color32::from_rgba_premultiplied(60, 60, 60, 220)
        } else {
            Color32::from_rgba_premultiplied(40, 40, 40, 200)
        };

        ui.painter().rect_filled(btn_1_1_rect, 10.0, btn_1_1_bg);
        ui.painter().text(
            btn_1_1_rect.center(),
            egui::Align2::CENTER_CENTER,
            "1:1",
            egui::FontId::proportional(11.0),
            if is_100_percent {
                Color32::from_gray(120)
            } else {
                Color32::WHITE
            },
        );

        // 1:1 按钮点击处理
        if btn_1_1_response.clicked() && !is_100_percent {
            state.scale = Scale::new(1.0, settings.min_scale, settings.max_scale);
            state.user_zoomed = true;
        }
    }

    /// 渲染尺寸指示器
    fn render_dimensions_indicator(&self, ui: &mut Ui, rect: Rect, state: &ViewState) {
        let dimensions_text = if let Some(ref image) = state.current_image {
            let mp = image.megapixels();
            format!(
                "{}x{} / {:.1} MP",
                image.metadata().width,
                image.metadata().height,
                mp
            )
        } else {
            "-".to_string()
        };

        let pos = rect.left_bottom() + Vec2::new(10.0, -10.0);
        let font = egui::FontId::proportional(12.0);

        let text_size = ui
            .painter()
            .layout(
                dimensions_text.clone(),
                font.clone(),
                Color32::WHITE,
                f32::INFINITY,
            )
            .size();

        let pill_rect = Rect::from_center_size(
            pos + Vec2::new(text_size.x / 2.0 + 5.0, -text_size.y / 2.0 - 5.0),
            text_size + Vec2::new(16.0, 10.0),
        );

        ui.painter().rect_filled(
            pill_rect,
            4.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        ui.painter().text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            dimensions_text,
            font,
            Color32::WHITE,
        );
    }
}
