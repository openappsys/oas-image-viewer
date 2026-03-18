use crate::core::ports::{AppConfig, Storage};
use crate::core::{CoreError, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Json 配置存储实现
pub struct JsonStorage {
    config_path: PathBuf,
    save_tx: Option<Sender<AppConfig>>,
}

impl JsonStorage {
    /// 创建新的 JSON 存储
    pub fn new() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        Self::log_debug(&format!("Config directory: {:?}", config_dir));

        std::fs::create_dir_all(&config_dir).map_err(|e| {
            CoreError::technical(
                "STORAGE_ERROR",
                format!("Failed to create config dir: {}", e),
            )
        })?;

        let config_path = config_dir.join("config.toml");
        Self::log_debug(&format!("Config file path: {:?}", config_path));

        Ok(Self {
            config_path,
            save_tx: None,
        })
    }

    fn log_debug(msg: &str) {
        use std::io::Write;
        let line = format!("[DEBUG] {}\n", msg);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("oas-image-viewer.log")
        {
            if let Err(e) = file.write_all(line.as_bytes()) {
                tracing::warn!(error = %e, "写入调试日志失败");
            }
        }
    }

    /// 从指定路径创建存储（用于临时/回退存储）
    pub fn from_path(config_path: PathBuf) -> Self {
        Self {
            config_path,
            save_tx: None,
        }
    }

    fn config_dir() -> Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "openappsys", "oas-image-viewer")
            .ok_or_else(|| {
                CoreError::technical("STORAGE_ERROR", "Failed to get project dirs".to_string())
            })?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }

    /// 启动防抖保存线程
    pub fn with_debounce(mut self) -> Self {
        let (tx, rx): (Sender<AppConfig>, Receiver<AppConfig>) = channel();
        let config_path = self.config_path.clone();

        thread::spawn(move || {
            use std::time::{Duration, Instant};

            const DEBOUNCE_MS: u64 = 500;
            let mut last_save = Instant::now();
            let mut pending: Option<AppConfig> = None;

            loop {
                match rx.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
                    Ok(config) => {
                        pending = Some(config);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(config) = pending.take() {
                            if last_save.elapsed().as_millis() >= 100 {
                                if let Err(e) = Self::save_to_file(&config_path, &config) {
                                    tracing::warn!(error = %e, "防抖保存配置失败");
                                }
                                last_save = Instant::now();
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        self.save_tx = Some(tx);
        self
    }

    fn save_to_file(path: &Path, config: &AppConfig) -> Result<()> {
        let json = toml::to_string_pretty(config).map_err(|e| {
            CoreError::technical("STORAGE_ERROR", format!("Failed to serialize: {}", e))
        })?;

        let temp_path = path.with_extension("toml.tmp");
        std::fs::write(&temp_path, json).map_err(|e| {
            CoreError::technical("STORAGE_ERROR", format!("Failed to write: {}", e))
        })?;

        std::fs::rename(&temp_path, path).map_err(|e| {
            CoreError::technical("STORAGE_ERROR", format!("Failed to rename: {}", e))
        })?;

        tracing::info!(path = ?path, "配置已保存");
        Ok(())
    }

    fn load_from_file(path: &Path) -> Result<AppConfig> {
        tracing::info!(path = ?path, "正在加载配置文件");

        let content = std::fs::read_to_string(path).map_err(|e| {
            tracing::warn!(error = %e, "读取配置文件失败");
            CoreError::technical("STORAGE_ERROR", format!("Failed to read: {}", e))
        })?;

        let config: AppConfig = toml::from_str(&content).map_err(|e| {
            tracing::warn!(error = %e, "解析配置文件失败");
            CoreError::technical("STORAGE_ERROR", format!("Failed to parse: {}", e))
        })?;

        tracing::info!("配置加载成功");
        Ok(config)
    }
}

impl Default for JsonStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| panic!("创建 JsonStorage 失败: {e}"))
    }
}

impl Storage for JsonStorage {
    fn load_config(&self) -> Result<AppConfig> {
        Self::log_debug(&format!("正在加载配置文件: {:?}", self.config_path));

        if self.config_path.exists() {
            Self::log_debug("配置文件存在，开始读取");
            Self::load_from_file(&self.config_path)
        } else {
            Self::log_debug("配置文件不存在，使用默认配置");
            Ok(AppConfig::default())
        }
    }

    fn save_config(&self, config: &AppConfig) -> Result<()> {
        Self::save_to_file(&self.config_path, config)
    }

    fn request_save(&self, config: &AppConfig) -> Result<()> {
        if let Some(ref tx) = self.save_tx {
            tx.send(config.clone()).map_err(|_| {
                CoreError::technical("STORAGE_ERROR", "Save channel closed".to_string())
            })?;
            Ok(())
        } else {
            self.save_config(config)
        }
    }
}
