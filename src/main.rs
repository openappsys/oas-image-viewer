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
use oas_image_viewer::core::ports::{AppConfig, FileDialogPort, ImageSource, Storage};
use oas_image_viewer::core::use_cases::{
    BatchUseCase, EditImageUseCase, ManageConfigUseCase, NavigateGalleryUseCase,
    OASImageViewerService, ViewImageUseCase,
};
use oas_image_viewer::{FsBatchPort, FsImageExportPort, FsImageSource, JsonStorage, RfdFileDialog};

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
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("创建日志目录失败: {}", e);
        }
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
            if let Err(e) = file.write_all(msg.as_bytes()) {
                eprintln!("写入崩溃日志失败: {}", e);
            }
        }
        // 尝试输出错误日志（Windows 下可能不可见）
        tracing::error!("{}", msg);
    }));

    if let Err(e) = std::fs::write(&log_paths().app, "") {
        eprintln!("清空应用日志失败: {}", e);
    }

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
        if let Err(e) = file.write_all(line.as_bytes()) {
            eprintln!("写入应用日志失败: {}", e);
        }
    }
}

fn parse_initial_path() -> Option<PathBuf> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        info!("从命令行打开: {:?}", path);
        log_to_file(&format!("命令行路径: {:?}", path));
        Some(path)
    } else {
        log_to_file("无命令行参数");
        None
    }
}

fn create_storage() -> Arc<dyn Storage> {
    match JsonStorage::new() {
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
    }
}

fn load_app_config(storage: &Arc<dyn Storage>) -> AppConfig {
    match storage.load_config() {
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
    }
}

fn create_service(
    image_source: Arc<FsImageSource>,
    storage: Arc<dyn Storage>,
) -> Arc<OASImageViewerService> {
    let view_use_case = ViewImageUseCase::new(image_source, storage.clone());
    let navigate_use_case = NavigateGalleryUseCase;
    let config_use_case = ManageConfigUseCase::new(storage);
    let edit_use_case = EditImageUseCase::new(Arc::new(FsImageExportPort::new()));
    let batch_use_case = BatchUseCase::new(Arc::new(FsBatchPort::new()));
    Arc::new(OASImageViewerService::new(
        view_use_case,
        navigate_use_case,
        config_use_case,
        edit_use_case,
        batch_use_case,
    ))
}

fn apply_initial_path(service: &Arc<OASImageViewerService>, initial_path: &Option<PathBuf>) {
    if let Some(path) = initial_path {
        info!("加载初始路径: {:?}", path);
        log_to_file(&format!("加载初始路径: {:?}", path));
        if path.is_dir() {
            let image_source = FsImageSource::new();
            if let Err(e) = service.load_directory(&image_source, path) {
                tracing::error!(path = %path.display(), error = %e, "处理初始目录失败");
            }
            return;
        }

        if let Err(e) = service.add_image_to_gallery(path) {
            tracing::error!(path = %path.display(), error = %e, "添加初始图片到图库失败");
            return;
        }

        let fit_to_window = service.is_fit_to_window_enabled().unwrap_or(true);
        if let Err(e) = service.open_image(path, None, None, fit_to_window) {
            tracing::error!(path = %path.display(), error = %e, "处理初始图片失败");
        }
    }
}

fn build_native_options(config: &AppConfig) -> NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([config.window.width, config.window.height])
        .with_min_inner_size([400.0, 300.0]);

    if let Some([x, y]) = config.window.position() {
        viewport = viewport.with_position([x, y]);
    }

    if config.window.maximized {
        viewport = viewport.with_maximized(true);
    }

    NativeOptions {
        viewport,
        ..Default::default()
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
    let initial_path = parse_initial_path();

    // 创建依赖
    info!("[步骤3] 创建图像源...");
    log_to_file("[步骤3] 创建图像源");
    let image_source = Arc::new(FsImageSource::new());
    let image_source_port: Arc<dyn ImageSource> = image_source.clone();
    let file_dialog: Arc<dyn FileDialogPort> = Arc::new(RfdFileDialog::new());

    info!("[步骤4] 创建存储...");
    log_to_file("[步骤4] 创建存储");
    let storage = create_storage();

    // 加载配置（仅加载一次）
    info!("[步骤5] 加载配置...");
    log_to_file("[步骤5] 加载配置");
    let config = load_app_config(&storage);

    // 创建用例
    info!("[步骤6] 创建用例...");
    log_to_file("[步骤6] 创建用例");
    info!("[步骤7] 创建应用服务...");
    log_to_file("[步骤7] 创建应用服务");
    let service = create_service(image_source, storage);

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
    apply_initial_path(&service, &initial_path);

    // 配置窗口
    info!("[步骤9] 配置窗口...");
    log_to_file("[步骤9] 配置窗口");
    let native_options = build_native_options(&config);

    // 启动应用
    info!("[步骤10] 启动UI...");
    log_to_file("[步骤10] 启动UI");
    let service_clone = service.clone();
    let file_dialog_clone = file_dialog.clone();
    let image_source_port_clone = image_source_port.clone();
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
                file_dialog_clone,
                image_source_port_clone,
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
