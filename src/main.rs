#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::env;
use std::path::PathBuf;

use anyhow::Result;
use eframe::NativeOptions;
use tracing::{info, warn};

use crate::app::ImageViewerApp;
use crate::config::Config;

mod app;
mod config;
mod decoder;
mod dnd;
mod gallery;
mod utils;
mod viewer;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,image_viewer=debug")
        .init();

    info!("Starting Image Viewer v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = env::args().collect();
    let initial_path = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        info!("Opening from command line: {:?}", path);
        Some(path)
    } else {
        None
    };

    let config = match Config::load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            warn!("Failed to load config: {}. Using defaults.", e);
            Config::default()
        }
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([config.window.width, config.window.height])
        .with_min_inner_size([400.0, 300.0]);

    if let Some([x, y]) = config.window.position() {
        viewport = viewport.with_position([x, y]);
    }

    if config.window.maximized {
        viewport = viewport.with_maximized(true);
    }

    let native_options = NativeOptions {
        viewport,
        ..Default::default()
    };

    let initial_path_clone = initial_path.clone();
    eframe::run_native(
        "Image Viewer",
        native_options,
        Box::new(move |cc| {
            // 配置中文字体支持
            setup_fonts(&cc.egui_ctx);
            Box::new(ImageViewerApp::new(cc, config, initial_path_clone))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    Ok(())
}


/// 配置字体支持，包括中文字体
fn setup_fonts(ctx: &egui::Context) {
    use egui::FontFamily;
    
    // 尝试加载系统字体
    let mut fonts = egui::FontDefinitions::default();
    
    // 添加系统默认字体作为备选
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 尝试加载系统字体
        let font_sources = [
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",  // Linux 文泉驿
            "/System/Library/Fonts/PingFang.ttc",            // macOS 苹方
            "C:\\Windows\\Fonts\\msyh.ttc",             // Windows 微软雅黑
            "C:\\Windows\\Fonts\\simhei.ttf",           // Windows 黑体
        ];
        
        for font_path in &font_sources {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "system_font".to_owned(),
                    egui::FontData::from_owned(font_data).into(),
                );
                fonts.families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .push("system_font".to_owned());
                fonts.families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .push("system_font".to_owned());
                info!("Loaded system font from: {}", font_path);
                break;
            }
        }
    }
    
    ctx.set_fonts(fonts);
}
