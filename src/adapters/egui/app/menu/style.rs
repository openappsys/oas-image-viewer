use egui::{Color32, Context};

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
    pub(super) menu_max_width: f32,
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
                menu_max_width: 320.0,
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
                menu_max_width: 320.0,
            }
        }
    }
}
