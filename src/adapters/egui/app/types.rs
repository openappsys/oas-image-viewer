//! egui 应用程序类型定义

use crate::adapters::clipboard::ClipboardManager;
use crate::adapters::egui::widgets::{GalleryWidget, ViewerWidget};
use crate::adapters::info_panel::InfoPanel;
use crate::adapters::shortcuts_help::ShortcutsHelpPanel;
use crate::core::ports::{FileDialogPort, ImageSource};
use crate::core::use_cases::OASImageViewerService;

use egui::Context;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

/// egui 应用程序适配器
pub struct EguiApp {
    pub(crate) service: Arc<OASImageViewerService>,
    pub(crate) file_dialog: Arc<dyn FileDialogPort>,
    pub(crate) image_source: Arc<dyn ImageSource>,
    pub(crate) viewer_widget: ViewerWidget,
    pub(crate) gallery_widget: GalleryWidget,
    pub(crate) info_panel: InfoPanel,
    pub(crate) shortcuts_help_panel: ShortcutsHelpPanel,
    pub(crate) clipboard_manager: ClipboardManager,
    pub(crate) show_about: bool,
    pub(crate) pending_files: Vec<PathBuf>,
    pub(crate) drag_hovering: bool,
    pub(crate) current_texture: Option<(String, egui::TextureHandle)>,
    pub(crate) current_texture_data: Option<(usize, usize, Vec<u8>)>,
    pub(crate) current_image_path: Option<PathBuf>,
    pub(crate) about_window_pos: Option<egui::Pos2>,
    pub(crate) last_context_menu_result: Option<String>,
    pub(crate) last_saved_window_pos: Option<egui::Pos2>,
    // 交互状态（用于拆分 update 函数）
    pub(crate) pending_clicked_image: Option<PathBuf>,
    pub(crate) pending_double_click: bool,
    // 延迟加载初始文件（命令行参数传入）
    pub(crate) initial_file: Option<PathBuf>,
    pub(crate) initial_file_processed: bool,
    pub(crate) integration_task_receiver: Option<Receiver<String>>,
    pub(crate) integration_task_running: bool,
}

impl EguiApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        service: Arc<OASImageViewerService>,
        file_dialog: Arc<dyn FileDialogPort>,
        image_source: Arc<dyn ImageSource>,
        initial_file: Option<PathBuf>,
    ) -> Self {
        Self::configure_styles(&cc.egui_ctx);

        let about_window_pos = service
            .get_state()
            .ok()
            .and_then(|state| state.config.viewer.about_window_pos)
            .map(|p| egui::pos2(p.x, p.y));

        let last_saved_window_pos = service.get_state().ok().and_then(|state| {
            let w = &state.config.window;
            match (w.x, w.y) {
                (Some(x), Some(y)) => Some(egui::pos2(x, y)),
                _ => None,
            }
        });

        Self {
            service,
            file_dialog,
            image_source,
            viewer_widget: ViewerWidget::default(),
            gallery_widget: GalleryWidget::default(),
            info_panel: InfoPanel::new(),
            shortcuts_help_panel: ShortcutsHelpPanel::new(),
            show_about: false,
            pending_files: Vec::new(),
            drag_hovering: false,
            current_texture: None,
            current_texture_data: None,
            current_image_path: None,
            about_window_pos,
            clipboard_manager: ClipboardManager::new(),
            last_context_menu_result: None,
            last_saved_window_pos,
            // 初始化交互状态
            pending_clicked_image: None,
            pending_double_click: false,
            // 延迟加载初始文件
            initial_file,
            initial_file_processed: false,
            integration_task_receiver: None,
            integration_task_running: false,
        }
    }

    fn configure_styles(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(10);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
        ctx.set_style(style);
    }
}
