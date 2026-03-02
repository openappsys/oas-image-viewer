#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::env;
use std::path::PathBuf;

use anyhow::Result;
use eframe::NativeOptions;
use tracing::{info, warn};

use crate::app::ImageViewerApp;
use crate::config::{Config, DebouncedConfigSaver};

mod app;
mod config;
mod decoder;
mod dnd;
mod gallery;
mod shortcuts_help;
mod utils;
mod info_panel;
mod viewer;
mod clipboard;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,image_viewer=debug")
        .init();

    info!("正在启动图片查看器 v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = env::args().collect();
    let initial_path = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        info!("从命令行打开: {:?}", path);
        Some(path)
    } else {
        None
    };

    let config = match Config::load() {
        Ok(cfg) => {
            info!("配置加载成功");
            cfg
        }
        Err(e) => {
            warn!("加载配置失败: {}. 使用默认配置。", e);
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

    // 创建防抖配置保存器
    let config_saver = DebouncedConfigSaver::new();

    let initial_path_clone = initial_path.clone();
    eframe::run_native(
        "Image Viewer",
        native_options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(ImageViewerApp::new(
                cc,
                config,
                initial_path_clone,
                config_saver,
            ))
        }),
    )
    .map_err(|e| anyhow::anyhow!("运行应用程序失败: {}", e))?;

    Ok(())
}

/// 配置字体支持，包括中文字体
fn setup_fonts(ctx: &egui::Context) {
    use egui::FontFamily;

    let mut fonts = egui::FontDefinitions::default();
    let mut font_loaded = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        // 按优先级尝试不同平台的中文字体
        let font_sources = [
            // ===== macOS =====
            // 苹方
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Supplemental/PingFang.ttc",
            "/Library/Fonts/PingFang.ttc",
            // 黑体
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/STHeiti.ttc",
            "/Library/Fonts/STHeiti Light.ttc",
            "/Library/Fonts/STHeiti Medium.ttc",
            // 冬青黑体
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Hiragino Sans GB.ttc",
            // Arial Unicode (备用)
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            // ===== Linux =====
            // Noto Sans CJK (主流发行版)
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
            // 文泉驿 (Ubuntu/Debian)
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/wqy-zenhei/wqy-zenhei.ttc",
            "/usr/local/share/fonts/wqy-zenhei.ttc",
            // 文鼎 PL (Fedora/CentOS)
            "/usr/share/fonts/cjkuni/uming.ttc",
            "/usr/share/fonts/cjkuni/ukai.ttc",
            "/usr/share/fonts/opentype/source-han-sans/SourceHanSansCN-Regular.otf",
            // 思源黑体
            "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Regular.otf",
            "/usr/share/fonts/opentype/adobe-source-han-sans/SourceHanSansCN-Regular.otf",
            // 方正 (一些发行版)
            "/usr/share/fonts/fangzheng/fzyh.ttf",
            // ===== Windows =====
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\msyhbd.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "C:\\Windows\\Fonts\\simsun.ttc",
            "C:\\Windows\\Fonts\\simkai.ttf",
            "C:\\Windows\\Fonts\\simfang.ttf",
        ];

        for font_path in &font_sources {
            match std::fs::read(font_path) {
                Ok(font_data) => {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts
                        .families
                        .entry(FontFamily::Proportional)
                        .or_default()
                        .insert(0, "chinese_font".to_owned());
                    fonts
                        .families
                        .entry(FontFamily::Monospace)
                        .or_default()
                        .push("chinese_font".to_owned());
                    info!("已加载中文字体: {}", font_path);
                    font_loaded = true;
                    break;
                }
                Err(_) => continue,
            }
        }
    }

    if !font_loaded {
        warn!("未加载中文字体，菜单在某些系统上可能显示为方块");
    }

    ctx.set_fonts(fonts);
}
