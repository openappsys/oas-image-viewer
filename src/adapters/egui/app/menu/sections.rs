use super::style::MenuStyle;
use super::EguiApp;
use egui::{Color32, CornerRadius, Vec2};
use unicode_segmentation::UnicodeSegmentation;

impl EguiApp {
    pub(super) fn render_menu_item(
        &mut self,
        ui: &mut egui::Ui,
        icon: &str,
        label: &str,
        shortcut: Option<&str>,
        style: &MenuStyle,
        enabled: bool,
    ) -> bool {
        let mut clicked = false;

        ui.add_enabled_ui(enabled, |ui| {
            let available_width = ui.available_width();
            let text_color = if enabled {
                style.text_color
            } else {
                style.shortcut_color
            };
            let icon_color = if enabled {
                style.icon_color
            } else {
                style.shortcut_color
            };
            let shortcut_width = shortcut
                .map(|shortcut_text| {
                    ui.painter()
                        .layout_no_wrap(
                            shortcut_text.to_string(),
                            egui::FontId::monospace(12.0),
                            style.shortcut_color,
                        )
                        .size()
                        .x
                })
                .unwrap_or(0.0);
            let label_left_padding =
                style.layout.row_horizontal_padding + style.layout.icon_column_width;
            let label_right_padding = if shortcut_width > 0.0 {
                style.layout.row_horizontal_padding + shortcut_width + style.layout.shortcut_gap
            } else {
                style.layout.row_horizontal_padding
            };
            let label_max_width = (available_width - label_left_padding - label_right_padding)
                .max(style.layout.label_min_width);
            let label_font = egui::FontId::proportional(14.0);
            let (display_label, label_overflow) =
                truncate_menu_label(ui, label, label_max_width, &label_font, text_color);
            let label_galley = ui.painter().layout_no_wrap(
                display_label,
                label_font,
                text_color,
            );
            let row_height = style.item_height;
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(available_width, row_height), egui::Sense::click());

            let is_hovered = response.hovered();
            let is_active = response.is_pointer_button_down_on();

            let bg_color = if is_active {
                style.active_bg
            } else if is_hovered {
                style.hover_bg
            } else {
                Color32::TRANSPARENT
            };

            if bg_color != Color32::TRANSPARENT {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(style.corner_radius), bg_color);
            }

            let mut left_x = rect.left() + style.layout.row_horizontal_padding;
            let center_y = rect.center().y;

            ui.painter().text(
                egui::pos2(left_x, center_y),
                egui::Align2::LEFT_CENTER,
                icon,
                egui::FontId::proportional(16.0),
                icon_color,
            );
            left_x += style.layout.icon_column_width;

            ui.painter().galley(
                egui::pos2(left_x, center_y - label_galley.size().y / 2.0),
                label_galley,
                text_color,
            );

            if let Some(shortcut_text) = shortcut {
                let shortcut_x = rect.right() - style.layout.row_horizontal_padding;
                ui.painter().text(
                    egui::pos2(shortcut_x, center_y),
                    egui::Align2::RIGHT_CENTER,
                    shortcut_text,
                    egui::FontId::monospace(12.0),
                    style.shortcut_color,
                );
            }

            let response = if label_overflow {
                response.on_hover_text(label)
            } else {
                response
            };
            clicked = response.clicked();
        });

        clicked
    }

    pub(super) fn render_menu_separator(&self, ui: &mut egui::Ui, _style: &MenuStyle) {
        ui.add_space(_style.layout.separator_spacing);
        ui.add(egui::Separator::default().spacing(0.0));
        ui.add_space(_style.layout.separator_spacing);
    }
}

fn truncate_menu_label(
    ui: &egui::Ui,
    text: &str,
    max_width: f32,
    font: &egui::FontId,
    color: Color32,
) -> (String, bool) {
    truncate_with_measure(text, max_width, |s| {
        ui.painter()
            .layout_no_wrap(s.to_string(), font.clone(), color)
            .size()
            .x
    })
}

fn truncate_with_measure<F>(text: &str, max_width: f32, measure: F) -> (String, bool)
where
    F: Fn(&str) -> f32,
{
    if measure(text) <= max_width {
        return (text.to_string(), false);
    }

    let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(text, true).collect();
    if graphemes.is_empty() {
        return (String::new(), false);
    }

    let ellipsis = "…";
    if measure(ellipsis) >= max_width {
        return (ellipsis.to_string(), true);
    }

    let mut lo = 0usize;
    let mut hi = graphemes.len();
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let candidate = format!("{}{}", graphemes[..mid].join(""), ellipsis);
        if measure(&candidate) <= max_width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    (format!("{}{}", graphemes[..lo].join(""), ellipsis), true)
}

#[cfg(test)]
mod tests {
    use super::truncate_with_measure;
    use unicode_segmentation::UnicodeSegmentation;

    fn measured_width(text: &str, scale: f32) -> f32 {
        UnicodeSegmentation::graphemes(text, true)
            .map(|g| {
                let mut chars = g.chars();
                let width = if g.chars().count() > 1 {
                    2.8
                } else if let Some(c) = chars.next() {
                    if c.is_ascii() || c == '…' {
                        1.0
                    } else {
                        2.0
                    }
                } else {
                    0.0
                };
                width
            })
            .sum::<f32>()
            * scale
    }

    #[test]
    fn truncate_english_only_when_needed() {
        let text = "Copy File Path";
        let wide = measured_width(text, 1.0) + 1.0;
        let (same, overflow) = truncate_with_measure(text, wide, |s| measured_width(s, 1.0));
        assert_eq!(same, text);
        assert!(!overflow);

        let narrow = measured_width("Copy File ", 1.0);
        let (truncated, overflow) =
            truncate_with_measure(text, narrow, |s| measured_width(s, 1.0));
        assert!(overflow);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn truncate_chinese_only_when_needed() {
        let text = "复制文件路径在文件夹中显示";
        let wide = measured_width(text, 1.0) + 1.0;
        let (same, overflow) = truncate_with_measure(text, wide, |s| measured_width(s, 1.0));
        assert_eq!(same, text);
        assert!(!overflow);

        let narrow = measured_width("复制文件路径", 1.0);
        let (truncated, overflow) =
            truncate_with_measure(text, narrow, |s| measured_width(s, 1.0));
        assert!(overflow);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn truncate_respects_large_scale() {
        let text = "复制文件路径在文件夹中显示";
        let max_width = measured_width(text, 1.0);
        let (same, overflow) = truncate_with_measure(text, max_width, |s| measured_width(s, 1.0));
        assert_eq!(same, text);
        assert!(!overflow);

        let (truncated, overflow) =
            truncate_with_measure(text, max_width, |s| measured_width(s, 1.5));
        assert!(overflow);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn truncate_does_not_split_grapheme_cluster() {
        let text = "👨‍👩‍👧‍👦Family";
        let max_width = measured_width("👨‍👩‍👧‍👦…", 1.0);
        let (truncated, overflow) =
            truncate_with_measure(text, max_width, |s| measured_width(s, 1.0));
        assert!(overflow);
        assert_eq!(truncated, "👨‍👩‍👧‍👦…");
    }
}
