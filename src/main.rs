#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
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
use image_viewer::core::domain::Image;
use image_viewer::infrastructure::{FsImageSource, JsonStorage};

fn main() {
    // 设置 panic 钩子，捕获 panic 信息
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("程序崩溃: {:?}\n", info);
        // 写入日志文件
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("image-viewer-error.log") {
            let _ = file.write_all(msg.as_bytes());
        }
        // 尝试显示错误（Windows 可能看不到）
        eprintln!("{}", msg);
    }));

    // 初始化日志到文件
    let _ = std::fs::write("image-viewer.log", ""); // 清空或创建日志文件
    
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    log_to_file("=== 程序启动 ===");
    log_to_file(&format!("版本: v{}", env!("CARGO_PKG_VERSION")));
    log_to_file(&format!("参数: {:?}", env::args().collect::<Vec<_>>()));

    if let Err(e) = run_app() {
        let err_msg = format!("程序错误: {}", e);
        log_to_file(&err_msg);
        eprintln!("{}", err_msg);
    }
}

fn log_to_file(msg: &str) {
    let line = format!("[{}] {}\n", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        msg
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("image-viewer.log") {
        let _ = file.write_all(line.as_bytes());
    }
}

fn run_app() -> Result<()> {
    info!("[STEP 1] 开始初始化...");
    log_to_file("[STEP 1] 开始初始化");

    // 解析命令行参数
    info!("[STEP 2] 解析命令行参数...");
    log_to_file("[STEP 2] 解析命令行参数");
    let args: Vec<String> = env::args().collect();
    let initial_path = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        info!("从命令行打开: {:?}", path);
        log_to_file(&format!("命令行路径: {:?}", path));
        Some(path)
    } else {
        log_to_file("无命令行参数");
        None
    };

    // 创建依赖
    info!("[STEP 3] 创建图像源...");
    log_to_file("[STEP 3] 创建图像源");
    let image_source = Arc::new(FsImageSource::new());
    
    info!("[STEP 4] 创建存储...");
    log_to_file("[STEP 4] 创建存储");
    let storage: Arc<dyn Storage> = match JsonStorage::new() {
        Ok(s) => {
            info!("存储初始化成功");
            log_to_file("存储初始化成功");
            Arc::new(s.with_debounce())
        }
        Err(e) => {
            warn!("创建存储失败: {}. 使用临时存储。", e);
            log_to_file(&format!("存储初始化失败: {}", e));
            let temp_path = std::env::temp_dir().join("image-viewer-temp-config.json");
            log_to_file(&format!("使用临时路径: {:?}", temp_path));
            Arc::new(JsonStorage::from_path(temp_path))
        }
    };

    // 加载配置（只加载一次）
    info!("[STEP 5] 加载配置...");
    log_to_file("[STEP 5] 加载配置");
    let config = match storage.load_config() {
        Ok(cfg) => {
            info!("配置加载成功");
            log_to_file("配置加载成功");
            cfg
        }
        Err(e) => {
            warn!("加载配置失败: {}. 使用默认配置。", e);
            log_to_file(&format!("配置加载失败: {}", e));
            AppConfig::default()
        }
    };
    
    // 创建用例
    info!("[STEP 6] 创建用例...");
    log_to_file("[STEP 6] 创建用例");
    let view_use_case = ViewImageUseCase::new(image_source.clone(), storage.clone());
    let navigate_use_case = NavigateGalleryUseCase;
    let config_use_case = ManageConfigUseCase::new(storage.clone());

    // 创建应用服务
    info!("[STEP 7] 创建应用服务...");
    log_to_file("[STEP 7] 创建应用服务");
    let service = Arc::new(ImageViewerService::new(
        view_use_case,
        navigate_use_case,
        config_use_case,
    ));

    // 初始化配置（使用已加载的配置，避免重复加载）
    info!("[STEP 8] 初始化服务...");
    log_to_file("[STEP 8] 初始化服务");
    service.initialize(Some(config.clone()))?;

    // 如果有初始路径，加载它
    if let Some(ref path) = initial_path {
        info!("加载初始路径: {:?}", path);
        log_to_file(&format!("加载初始路径: {:?}", path));
        let _ = service.update_state(|state| {
            if path.is_dir() {
                let image_source = FsImageSource::new();
                let _ = service.navigate_use_case.load_directory(
                    &mut state.gallery,
                    &image_source,
                    path,
                );
            } else {
                // 打开图片并添加到画廊（只添加当前图片，不加载整个目录）
                let image = Image::new(
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    path.clone(),
                );
                state.gallery.gallery.add_image(image);
                
                // 初始加载时使用默认窗口大小（稍后会被实际窗口大小覆盖）
                let fit_to_window = state.config.viewer.fit_to_window;
                let _ = service.view_use_case.open_image(path, &mut state.view, None, None, fit_to_window);
            }
        });
    }

    // 配置窗口
    info!("[STEP 9] 配置窗口...");
    log_to_file("[STEP 9] 配置窗口");
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
    info!("[STEP 10] 启动 UI...");
    log_to_file("[STEP 10] 启动 UI");
    let service_clone = service.clone();
    eframe::run_native(
        "Image Viewer",
        native_options,
        Box::new(move |cc| {
            log_to_file("UI 初始化回调");
            cc.egui_ctx.set_pixels_per_point(1.0);
            setup_fonts(&cc.egui_ctx);
            log_to_file("字体设置完成");
            Box::new(EguiApp::new(cc, service_clone))
        }),
    )
    .map_err(|e| {
        log_to_file(&format!("eframe 错误: {}", e));
        anyhow::anyhow!("运行应用程序失败: {}", e)
    })?;

    log_to_file("程序正常退出");
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
