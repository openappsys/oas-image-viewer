//! Viewer Widget - 查看器 UI 组件

use crate::core::domain::ViewerSettings;
use crate::core::use_cases::ViewState;
use egui::{Color32, Rect, Sense, Ui, Vec2};

/// 查看器组件
#[derive(Default)]
pub struct ViewerWidget {
    dragging: bool,
    /// 累积的滚轮增量，用于平滑缩放
    #[allow(dead_code)]
    zoom_accumulator: f32,
}

impl ViewerWidget {
    /// 渲染查看器
    /// 返回 (是否双击全屏, 缩放因子, 鼠标位置, 拖拽偏移量, 是否触发右键菜单)
    /// 缩放因子: 1.0 表示无变化, >1.0 放大, <1.0 缩小
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        state: &ViewState,
        settings: &ViewerSettings,
        texture: Option<&(String, egui::TextureHandle)>,
    ) -> (bool, f32, Option<egui::Pos2>, Option<Vec2>, bool) {
        let available_size = ui.available_size();
        let bg_color = Color32::from_rgb(
            settings.background_color.r,
            settings.background_color.g,
            settings.background_color.b,
        );

        let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // 处理双击全屏 - 修复: 使用 input().pointer 点击状态来检测双击
        // double_clicked() 需要正确的 Sense 支持，click_and_drag 同时支持点击和拖拽
        let double_clicked = ui.input(|i| {
            i.pointer.button_double_clicked(egui::PointerButton::Primary)
        });

        // 处理拖拽平移（左键拖拽）- 使用 response.drag_delta() 与 v0.2.0 一致
        let drag_delta = if response.dragged() {
            self.dragging = true;
            response.drag_delta()
        } else {
            self.dragging = false;
            Vec2::ZERO
        };

        // 处理滚轮缩放 - 与 v0.2.0 一致：以鼠标为中心
        let mut zoom_factor = 1.0;
        let mut mouse_pos: Option<egui::Pos2> = None;
        
        if response.hovered() && !self.dragging {
            // 与 v0.2.0 一致：直接使用 scroll_delta，不区分普通滚轮还是中键滚轮
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                // 与 v0.2.0 相同的连续缩放公式
                zoom_factor = 1.0 + scroll_delta * 0.001;
                // 获取鼠标位置
                mouse_pos = ui.input(|i| i.pointer.hover_pos());
            }
        }

        // 处理右键菜单触发信号（初始化为 false）
        let mut context_menu_triggered = false;
        
        // 渲染图像或占位符
        if let Some(ref image) = state.current_image {
            // 处理右键菜单 - 与 v0.2.0 一致（需要克隆 response，因为它会被移动）
            response.clone().context_menu(|_ui| {
                // 菜单内容由父组件处理
            });
            context_menu_triggered = true;
            
            self.render_image(ui, image, state, rect, &response, settings, texture);
        } else {
            // 无图像占位符
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "未选择图像\n按 Ctrl+O 打开图像或从图库中选择\n也可以直接拖拽图像到窗口",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
        }

        // 渲染缩放指示器
        self.render_zoom_indicator(ui, rect, state);

        // 渲染尺寸指示器
        self.render_dimensions_indicator(ui, rect, state);
        
        let drag_offset = if drag_delta != Vec2::ZERO { Some(drag_delta) } else { None };
        (double_clicked, zoom_factor, mouse_pos, drag_offset, context_menu_triggered)
    }

    /// 渲染图像
    fn render_image(
        &self,
        ui: &mut Ui,
        image: &crate::core::domain::Image,
        state: &ViewState,
        rect: Rect,
        _response: &egui::Response,
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

            // 设置裁剪区域（与 v0.2.0 一致），避免图片溢出到信息面板
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

    /// 渲染缩放指示器
    fn render_zoom_indicator(&self, ui: &mut Ui, rect: Rect, state: &ViewState) {
        let zoom_text = format!("{:.0}%", state.scale.percentage());
        let pos = rect.right_bottom() - Vec2::new(10.0, 10.0);
        let font = egui::FontId::proportional(12.0);

        let text_size = ui
            .painter()
            .layout(
                zoom_text.clone(),
                font.clone(),
                Color32::WHITE,
                f32::INFINITY,
            )
            .size();

        let pill_rect = Rect::from_center_size(
            pos - Vec2::new(text_size.x / 2.0 + 5.0, text_size.y / 2.0 + 5.0),
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
            zoom_text,
            font,
            Color32::WHITE,
        );
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
