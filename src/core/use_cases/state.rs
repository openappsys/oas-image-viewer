use crate::core::domain::{Gallery, GalleryLayout, Image, Position, Scale, ViewMode};
use crate::core::ports::AppConfig;

#[derive(Debug, Clone)]
pub struct ViewState {
    pub current_image: Option<Image>,
    pub scale: Scale,
    pub offset: Position,
    pub view_mode: ViewMode,
    pub user_zoomed: bool,
}

#[derive(Debug, Clone)]
pub struct GalleryState {
    pub gallery: Gallery,
    pub layout: GalleryLayout,
    pub items_per_row: usize,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub view: ViewState,
    pub gallery: GalleryState,
    pub config: AppConfig,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            current_image: None,
            scale: Scale::new(1.0, 0.1, 20.0),
            offset: Position::default(),
            view_mode: ViewMode::Gallery,
            user_zoomed: false,
        }
    }
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            gallery: Gallery::new("Default"),
            layout: GalleryLayout::default(),
            items_per_row: 0,
        }
    }
}
