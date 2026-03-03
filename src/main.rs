#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use eframe::NativeOptions;
use tracing::{info, warn};

use image_viewer::adapters::egui::EguiApp;
use image_viewer::core::ports::{AppConfig, Storage};
use image_viewer::core::use_cases::{
    ImageViewerService, ManageConfigUseCase, NavigateGalleryUseCase, ViewImageUseCase,
};
use image_viewer::infrastructure::{FsImageSource, JsonStorage};

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,image_viewer=debug")
        .init();

    info!("正在启动图片查看器 v{}", env!("CARGO_PKG_VERSION"));

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    let initial_path = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        info!("从命令行打开: {:?}", path);
        Some(path)
    } else {
        None
    };

    // 创建依赖
    let image_source = Arc::new(FsImageSource::new());
    let storage = Arc::new(JsonStorage::new()?.with_debounce());

    // 加载配置
    let config = match storage.load_config() {
        Ok(cfg) => {
            info!("配置加载成功");
            cfg
        }
        Err(e) => {
            warn!("加载配置失败: {}. 使用默认配置。", e);
            AppConfig::default()
        }
    };

    // 创建用例
    let view_use_case = ViewImageUseCase::new(image_source.clone(), storage.clone());
    let navigate_use_case = NavigateGalleryUseCase;
    let config_use_case = ManageConfigUseCase::new(storage.clone());

    // 创建应用服务
    let service = Arc::new(ImageViewerService::new(
        view_use_case,
        navigate_use_case,
        config_use_case,
    ));

    // 初始化配置
    service.initialize()?;

    // 如果有初始路径，加载它
    if let Some(ref path) = initial_path {
        let _ = service.update_state(|state| {
            if path.is_dir() {
                // 加载目录到画廊
                let image_source = FsImageSource::new();
                let _ = service.navigate_use_case.load_directory(
                    &mut state.gallery,
                    &image_source,
                    path,
                );
            } else {
                // 打开单个图像
                let _ = service.view_use_case.open_image(path, &mut state.view);
            }
        });
    }

    // 配置窗口
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

    // 运行应用程序
    let service_clone = service.clone();
    eframe::run_native(
        "Image Viewer",
        native_options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(EguiApp::new(cc, service_clone))
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
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Supplemental/PingFang.ttc",
            "/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Hiragino Sans GB.ttc",
            // ===== Linux =====
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/wqy-zenhei/wqy-zenhei.ttc",
            // ===== Windows =====
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
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
