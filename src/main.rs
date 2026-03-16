#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use eframe::NativeOptions;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::{info, warn};

use oas_image_viewer::adapters::egui::EguiApp;
#[cfg(target_os = "macos")]
use oas_image_viewer::adapters::macos_file_open;
use oas_image_viewer::core::domain::Image;
use oas_image_viewer::core::ports::{AppConfig, Storage};
use oas_image_viewer::core::use_cases::{
    ManageConfigUseCase, NavigateGalleryUseCase, OASImageViewerService, ViewImageUseCase,
};
use oas_image_viewer::{FsImageSource, JsonStorage};

struct LogPaths {
    app: PathBuf,
    crash: PathBuf,
}

static LOG_PATHS: OnceLock<LogPaths> = OnceLock::new();

fn resolve_log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Logs")
                .join("OAS Image Viewer")
        } else {
            std::env::temp_dir().join("OAS Image Viewer").join("logs")
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            PathBuf::from(local_app_data)
                .join("OAS Image Viewer")
                .join("logs")
        } else {
            std::env::temp_dir().join("OAS Image Viewer").join("logs")
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(state_home) = std::env::var_os("XDG_STATE_HOME") {
            PathBuf::from(state_home)
                .join("oas-image-viewer")
                .join("logs")
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("state")
                .join("oas-image-viewer")
                .join("logs")
        } else {
            std::env::temp_dir().join("oas-image-viewer").join("logs")
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        std::env::temp_dir().join("oas-image-viewer").join("logs")
    }
}

fn log_paths() -> &'static LogPaths {
    LOG_PATHS.get_or_init(|| {
        let log_dir = resolve_log_dir();
        let _ = std::fs::create_dir_all(&log_dir);
        LogPaths {
            app: log_dir.join("app.log"),
            crash: log_dir.join("crash.log"),
        }
    })
}

fn main() {
    // 注册 panic 钩子，捕获崩溃信息
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("应用崩溃: {:?}\n", info);
        // 写入崩溃日志文件
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_paths().crash)
        {
            let _ = file.write_all(msg.as_bytes());
        }
        // 尝试输出错误日志（Windows 下可能不可见）
        tracing::error!("{}", msg);
    }));

    let _ = std::fs::write(&log_paths().app, "");

    // 默认日志级别为 INFO 及以上，可通过 RUST_LOG 环境变量覆盖
    // 示例：RUST_LOG=debug 显示全部调试日志
    // 示例：RUST_LOG=oas_image_viewer=debug,winit=error 为模块单独设置级别
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    log_to_file("=== 应用启动 ===");
    log_to_file(&format!("版本: v{}", env!("CARGO_PKG_VERSION")));
    log_to_file(&format!("参数: {:?}", env::args().collect::<Vec<_>>()));

    if let Err(e) = run_app() {
        let err_msg = format!("应用错误: {}", e);
        log_to_file(&err_msg);
        tracing::error!("{}", err_msg);
    }
}

fn log_to_file(msg: &str) {
    let line = format!(
        "[{}] {}\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        msg
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_paths().app)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

fn run_app() -> Result<()> {
    info!("[步骤1] 开始初始化...");
    log_to_file("[步骤1] 开始初始化");

    // 先初始化国际化系统
    info!("[步骤1.5] 初始化国际化系统...");
    log_to_file("[步骤1.5] 初始化国际化系统");
    oas_image_viewer::adapters::egui::i18n::initialize();

    // 解析命令行参数
    info!("[步骤2] 解析命令行参数...");
    log_to_file("[步骤2] 解析命令行参数");
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
    info!("[步骤3] 创建图像源...");
    log_to_file("[步骤3] 创建图像源");
    let image_source = Arc::new(FsImageSource::new());

    info!("[步骤4] 创建存储...");
    log_to_file("[步骤4] 创建存储");
    let storage: Arc<dyn Storage> = match JsonStorage::new() {
        Ok(s) => {
            info!("存储初始化成功");
            log_to_file("存储初始化成功");
            Arc::new(JsonStorage::with_debounce(s))
        }
        Err(e) => {
            warn!("创建存储失败: {}，使用临时存储", e);
            log_to_file(&format!("存储初始化失败: {}", e));
            let temp_path = std::env::temp_dir().join("oas-image-viewer-temp-config.toml");
            log_to_file(&format!("使用临时路径: {:?}", temp_path));
            Arc::new(JsonStorage::from_path(temp_path))
        }
    };

    // 加载配置（仅加载一次）
    info!("[步骤5] 加载配置...");
    log_to_file("[步骤5] 加载配置");
    let config = match storage.load_config() {
        Ok(cfg) => {
            info!("配置加载成功");
            log_to_file("配置加载成功");
            cfg
        }
        Err(e) => {
            warn!("加载配置失败: {}，使用默认配置", e);
            log_to_file(&format!("配置加载失败: {}", e));
            AppConfig::default()
        }
    };

    // 创建用例
    info!("[步骤6] 创建用例...");
    log_to_file("[步骤6] 创建用例");
    let view_use_case = ViewImageUseCase::new(image_source.clone(), storage.clone());
    let navigate_use_case = NavigateGalleryUseCase;
    let config_use_case = ManageConfigUseCase::new(storage.clone());

    // 创建应用服务
    info!("[步骤7] 创建应用服务...");
    log_to_file("[步骤7] 创建应用服务");
    let service = Arc::new(OASImageViewerService::new(
        view_use_case,
        navigate_use_case,
        config_use_case,
    ));

    // 初始化服务（复用已加载配置，避免重复加载）
    info!("[步骤8] 初始化服务...");
    log_to_file("[步骤8] 初始化服务");
    service.initialize(Some(config.clone()))?;

    // 注册 macOS 文件打开处理器
    #[cfg(target_os = "macos")]
    {
        info!("[步骤8.5] 设置 macOS 文件打开处理程序...");
        log_to_file("[步骤8.5] 设置 macOS 文件打开处理程序");
        macos_file_open::setup_file_open_handler();
    }

    // 如果存在初始路径则执行加载
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
                // 打开单图并加入图库（仅加入当前图片，不扫描整个目录）
                let image = Image::new(
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    path.clone(),
                );
                state.gallery.gallery.add_image(image);

                // 初次加载使用默认窗口尺寸（后续会被真实窗口尺寸覆盖）
                let fit_to_window = state.config.viewer.fit_to_window;
                let _ = service.view_use_case.open_image(
                    path,
                    &mut state.view,
                    None,
                    None,
                    fit_to_window,
                );
            }
        });
    }

    // 配置窗口
    info!("[步骤9] 配置窗口...");
    log_to_file("[步骤9] 配置窗口");
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

    // 启动应用
    info!("[步骤10] 启动UI...");
    log_to_file("[步骤10] 启动UI");
    let service_clone = service.clone();
    let initial_path_clone = initial_path.clone();
    eframe::run_native(
        "OAS Image Viewer",
        native_options,
        Box::new(move |cc| {
            log_to_file("界面初始化回调");
            setup_fonts(&cc.egui_ctx, &config);
            log_to_file("字体设置完成");
            Ok(Box::new(EguiApp::new(
                cc,
                service_clone,
                initial_path_clone,
            )))
        }),
    )
    .map_err(|e| {
        log_to_file(&format!("eframe错误: {}", e));
        anyhow::anyhow!("运行应用失败: {}", e)
    })?;

    log_to_file("应用正常退出");
    Ok(())
}

/// 配置字体支持（包含中文字体）
///
/// 无条件尝试加载系统中文字体（若可用），与当前语言设置无关
fn setup_fonts(ctx: &egui::Context, _config: &AppConfig) {
    use egui::FontFamily;

    let mut fonts = egui::FontDefinitions::default();
    let mut font_loaded = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        // 按优先级尝试加载各平台中文字体（无条件加载）
        for font_path in oas_image_viewer::CHINESE_FONT_PATHS {
            match std::fs::read(font_path) {
                Ok(font_data) => {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
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
                    oas_image_viewer::set_chinese_supported(true);
                    break;
                }
                Err(_) => continue,
            }
        }
    }

    if !font_loaded {
        warn!("未找到中文字体，界面将以英文显示");
        info!("使用egui默认字体(40KB)，仅英文显示");
        oas_image_viewer::set_chinese_supported(false);
    }

    ctx.set_fonts(fonts);
}
