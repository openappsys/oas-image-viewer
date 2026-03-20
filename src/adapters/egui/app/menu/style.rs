//! 菜单样式参数定义与主题映射

use egui::{Color32, Context};

pub(super) struct MenuLayoutMetrics {
    pub(super) menu_bar_height: f32,
    pub(super) row_horizontal_padding: f32,
    pub(super) icon_column_width: f32,
    pub(super) shortcut_gap: f32,
    pub(super) popup_horizontal_overhead: f32,
    pub(super) label_min_width: f32,
    pub(super) separator_spacing: f32,
    pub(super) popup_inner_vertical_padding: f32,
    pub(super) popup_margin: i8,
    pub(super) popup_height_margin: f32,
    pub(super) popup_min_height: f32,
    pub(super) popup_max_height: f32,
    pub(super) viewport_width_margin: f32,
}

pub(super) struct MenuStyle {
    pub(super) bg_color: Color32,
    pub(super) hover_bg: Color32,
    pub(super) active_bg: Color32,
    pub(super) text_color: Color32,
    pub(super) shortcut_color: Color32,
    pub(super) icon_color: Color32,
    pub(super) corner_radius: u8,
    pub(super) item_height: f32,
    pub(super) menu_min_width: f32,
    pub(super) menu_max_width_ratio: f32,
    pub(super) layout: MenuLayoutMetrics,
}

impl MenuStyle {
    pub(super) fn new(ctx: &Context) -> Self {
        let is_dark = ctx.style().visuals.dark_mode;

        if is_dark {
            Self {
                bg_color: Color32::from_rgb(45, 45, 48),
                hover_bg: Color32::from_rgb(60, 60, 65),
                active_bg: Color32::from_rgb(0, 122, 204),
                text_color: Color32::from_rgb(240, 240, 240),
                shortcut_color: Color32::from_rgb(160, 160, 160),
                icon_color: Color32::from_rgb(200, 200, 200),
                corner_radius: 6,
                item_height: 28.0,
                menu_min_width: 220.0,
                menu_max_width_ratio: 0.78,
                layout: default_layout_metrics(),
            }
        } else {
            Self {
                bg_color: Color32::from_rgb(250, 250, 250),
                hover_bg: Color32::from_rgb(230, 240, 250),
                active_bg: Color32::from_rgb(0, 122, 204),
                text_color: Color32::from_rgb(30, 30, 30),
                shortcut_color: Color32::from_rgb(120, 120, 120),
                icon_color: Color32::from_rgb(80, 80, 80),
                corner_radius: 6,
                item_height: 28.0,
                menu_min_width: 220.0,
                menu_max_width_ratio: 0.78,
                layout: default_layout_metrics(),
            }
        }
    }
}

fn default_layout_metrics() -> MenuLayoutMetrics {
    MenuLayoutMetrics {
        menu_bar_height: 40.0,
        row_horizontal_padding: 12.0,
        icon_column_width: 26.0,
        shortcut_gap: 16.0,
        popup_horizontal_overhead: 20.0,
        label_min_width: 100.0,
        separator_spacing: 6.0,
        popup_inner_vertical_padding: 4.0,
        popup_margin: 6,
        popup_height_margin: 80.0,
        popup_min_height: 180.0,
        popup_max_height: 560.0,
        viewport_width_margin: 12.0,
    }
}
