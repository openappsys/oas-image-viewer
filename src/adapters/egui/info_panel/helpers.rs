//! 信息面板渲染与文本格式化辅助函数

use egui::RichText;

pub(super) fn render_label_value(ui: &mut egui::Ui, label: &str, value: &str) {
    let text_color = ui.style().visuals.text_color();
    let weak_color = ui.style().visuals.weak_text_color();
    ui.horizontal_top(|ui| {
        ui.label(RichText::new(label).size(13.0).color(weak_color));
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                let mut text = value.to_string();
                let response = ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(ui.available_width())
                        .desired_rows(1)
                        .font(egui::TextStyle::Body)
                        .text_color(text_color),
                );
                response.on_hover_text(value);
            },
        );
    });
}

pub(super) fn format_camera_info(make: Option<&str>, model: Option<&str>) -> String {
    match (make, model) {
        (Some(m), Some(n)) => {
            let make = m.trim();
            let model = n.trim();
            if model.starts_with(make) {
                model.to_string()
            } else {
                format!("{} {}", make, model)
            }
        }
        (Some(m), None) => m.trim().to_string(),
        (None, Some(n)) => n.trim().to_string(),
        (None, None) => "Unknown".to_string(),
    }
}
