use egui::RichText;

pub(super) fn render_label_value(ui: &mut egui::Ui, label: &str, value: &str) {
    let text_color = ui.style().visuals.text_color();
    let weak_color = ui.style().visuals.weak_text_color();
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(weak_color));
        let mut text = value.to_string();
        let text_edit = egui::TextEdit::singleline(&mut text)
            .text_color(text_color)
            .font(egui::TextStyle::Body)
            .desired_width(ui.available_width());
        let _response = ui.add(text_edit);
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
