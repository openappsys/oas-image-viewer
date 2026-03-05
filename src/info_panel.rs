//! 信息面板模块 - 显示图片元数据
//!
//! 显示图片的详细信息，包括文件信息、图片属性和EXIF元数据。
//! 支持异步加载EXIF数据，不阻塞UI。

use egui::{Color32, Context, Frame, RichText, ScrollArea, SidePanel, Widget};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use tracing::{debug, error, warn};

/// 图像信息面板
pub struct InfoPanel {
    visible: bool,
    width: f32,
    current_info: Option<ImageInfo>,
    exif_receiver: Option<Receiver<ExifData>>,
    loading_exif: bool,
}

/// 图像信息数据结构
#[derive(Debug, Clone, Default)]
pub struct ImageInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 文件名
    pub file_name: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 文件修改时间
    pub modified_time: Option<String>,
    /// 图像格式
    pub format: String,
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 位深度
    pub bit_depth: Option<u8>,
    /// 色彩空间
    pub color_space: Option<String>,
    /// EXIF数据
    pub exif: Option<ExifData>,
}

/// EXIF元数据
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    /// 拍摄时间
    pub date_time: Option<String>,
    /// 相机型号
    pub camera_model: Option<String>,
    /// 相机制造商
    pub camera_make: Option<String>,
    /// 镜头型号
    pub lens_model: Option<String>,
    /// ISO感光度
    pub iso: Option<u32>,
    /// 光圈值
    pub aperture: Option<String>,
    /// 快门速度
    pub shutter_speed: Option<String>,
    /// 焦距
    pub focal_length: Option<String>,
    /// GPS纬度
    pub gps_latitude: Option<String>,
    /// GPS经度
    pub gps_longitude: Option<String>,
}

impl InfoPanel {
    /// 创建新的信息面板
    pub fn new() -> Self {
        Self {
            visible: false,
            width: 280.0,
            current_info: None,
            exif_receiver: None,
            loading_exif: false,
        }
    }

    /// 从配置创建信息面板
    pub fn with_visibility(visible: bool) -> Self {
        Self {
            visible,
            width: 280.0,
            current_info: None,
            exif_receiver: None,
            loading_exif: false,
        }
    }

    /// 切换面板可见性
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        debug!("信息面板可见性切换为: {}", self.visible);
    }

    /// 显示面板
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// 隐藏面板
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// 检查面板是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置图像信息（同步部分）
    pub fn set_image_info(&mut self, path: &Path, dimensions: (u32, u32), format: &str) {
        debug!("设置图像信息: {:?}", path);

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 获取文件元数据
        let (file_size, modified_time) = Self::get_file_metadata(path);

        // 创建基本信息
        let info = ImageInfo {
            path: path.to_path_buf(),
            file_name,
            file_size,
            modified_time,
            format: format.to_string(),
            width: dimensions.0,
            height: dimensions.1,
            bit_depth: None, // 需要实际解码才能获取
            color_space: None,
            exif: None,
        };

        self.current_info = Some(info);
        self.loading_exif = true;

        // 异步加载EXIF数据
        self.load_exif_async(path);
    }

    /// 清除当前图像信息
    pub fn clear(&mut self) {
        self.current_info = None;
        self.exif_receiver = None;
        self.loading_exif = false;
    }

    /// 处理输入（F键和ESC键）
    pub fn handle_input(&mut self, ctx: &Context) -> bool {
        // F键切换面板
        let f_pressed = ctx.input(|i| i.key_pressed(egui::Key::F) && !i.modifiers.any());

        if f_pressed {
            self.toggle();
            return true;
        }

        // ESC键关闭面板（仅当面板可见时）
        let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        if esc_pressed && self.visible {
            self.hide();
            return true;
        }

        false
    }

    /// 检查并接收异步加载的EXIF数据
    fn check_exif_receiver(&mut self) {
        if let Some(receiver) = &self.exif_receiver {
            match receiver.try_recv() {
                Ok(exif_data) => {
                    if let Some(ref mut info) = self.current_info {
                        info.exif = Some(exif_data);
                    }
                    self.loading_exif = false;
                    self.exif_receiver = None;
                    debug!("EXIF数据加载完成");
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // 仍在加载中
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // 通道断开，加载失败
                    self.loading_exif = false;
                    self.exif_receiver = None;
                    warn!("EXIF加载通道断开");
                }
            }
        }
    }

    /// 渲染信息面板
    /// 返回：如果本帧用户点击了右上角关闭按钮，则返回 true
    pub fn ui(&mut self, ctx: &Context) -> bool {
        // 检查是否有新的EXIF数据
        self.check_exif_receiver();

        if !self.visible {
            return false;
        }

        let mut closed_by_user = false;

        let panel_width = self.width;

        SidePanel::right("info_panel")
            .resizable(true)
            .min_width(200.0)
            .max_width(400.0)
            .default_width(panel_width)
            .frame(
                Frame::side_top_panel(&ctx.style())
                    .fill(Color32::from_rgba_premultiplied(35, 35, 40, 240)),
            )
            .show(ctx, |ui| {
                // 更新宽度（限制在最小和最大之间）
                let new_width = ui.available_width();
                self.width = new_width.clamp(200.0, 400.0);

                // 标题栏
                ui.horizontal(|ui| {
                    ui.heading("📋 图像信息");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("×").clicked() {
                            self.hide();
                            closed_by_user = true;
                        }
                    });
                });

                ui.separator();

                // 内容区域
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if let Some(ref info) = self.current_info {
                            self.render_info_content(ui, info);
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.label(RichText::new("未选择图像").color(Color32::GRAY));
                                ui.label(
                                    RichText::new("打开图像以查看详细信息")
                                        .color(Color32::GRAY)
                                        .size(12.0),
                                );
                            });
                        }
                    });
            });

        closed_by_user
    }

    /// 渲染信息内容
    fn render_info_content(&self, ui: &mut egui::Ui, info: &ImageInfo) {
        // 文件信息部分
        egui::CollapsingHeader::new("\u{1f4c1} 文件信息")
            .default_open(true)
            .show(ui, |ui| {
                render_label_value(ui, "文件名:", &info.file_name);
                render_label_value(ui, "路径:", &format_path(&info.path));
                render_label_value(ui, "大小:", &format_file_size(info.file_size));
                if let Some(ref time) = info.modified_time {
                    render_label_value(ui, "修改时间:", time);
                }
            });

        ui.add_space(8.0);

        // 图像信息部分
        egui::CollapsingHeader::new("\u{1f5bc} 图像信息")
            .default_open(true)
            .show(ui, |ui| {
                render_label_value(ui, "格式:", &info.format);
                render_label_value(
                    ui,
                    "尺寸:",
                    &format!("{} x {} 像素", info.width, info.height),
                );
                let mp = (info.width as f64 * info.height as f64) / 1_000_000.0;
                render_label_value(ui, "百万像素:", &format!("{:.2} MP", mp));
                if let Some(depth) = info.bit_depth {
                    render_label_value(ui, "位深度:", &format!("{} bit", depth));
                }
                if let Some(ref space) = info.color_space {
                    render_label_value(ui, "色彩空间:", space);
                }
            });

        ui.add_space(8.0);

        // EXIF信息部分
        egui::CollapsingHeader::new("\u{1f4f7} EXIF 信息")
            .default_open(true)
            .show(ui, |ui| {
                if self.loading_exif {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("正在加载EXIF数据...")
                                .color(Color32::GRAY)
                                .size(12.0),
                        );
                    });
                } else if let Some(ref exif) = info.exif {
                    self.render_exif_content(ui, exif);
                } else {
                    ui.label(RichText::new("无EXIF数据").color(Color32::GRAY).size(12.0));
                }
            });
    }

    /// 渲染EXIF内容
    fn render_exif_content(&self, ui: &mut egui::Ui, exif: &ExifData) {
        // 相机信息
        if exif.camera_make.is_some() || exif.camera_model.is_some() {
            let camera =
                format_camera_info(exif.camera_make.as_deref(), exif.camera_model.as_deref());
            render_label_value(ui, "相机:", &camera);
        }

        if let Some(ref lens) = exif.lens_model {
            render_label_value(ui, "镜头:", lens);
        }

        if let Some(ref date) = exif.date_time {
            render_label_value(ui, "拍摄时间:", date);
        }

        ui.add_space(4.0);

        // 曝光参数
        if let Some(iso) = exif.iso {
            render_label_value(ui, "ISO:", &iso.to_string());
        }

        if let Some(ref aperture) = exif.aperture {
            render_label_value(ui, "光圈:", aperture);
        }

        if let Some(ref shutter) = exif.shutter_speed {
            render_label_value(ui, "快门:", shutter);
        }

        if let Some(ref focal) = exif.focal_length {
            render_label_value(ui, "焦距:", focal);
        }

        // GPS信息
        if exif.gps_latitude.is_some() || exif.gps_longitude.is_some() {
            ui.add_space(4.0);
            if let Some(ref lat) = exif.gps_latitude {
                render_label_value(ui, "纬度:", lat);
            }
            if let Some(ref lon) = exif.gps_longitude {
                render_label_value(ui, "经度:", lon);
            }
        }
    }

    /// 异步加载EXIF数据
    fn load_exif_async(&mut self, path: &Path) {
        let path = path.to_path_buf();
        let (sender, receiver) = channel::<ExifData>();

        self.exif_receiver = Some(receiver);

        thread::spawn(move || {
            let exif_data = Self::read_exif_data(&path);
            if let Err(e) = sender.send(exif_data) {
                error!("发送EXIF数据失败: {:?}", e);
            }
        });
    }

    /// 读取EXIF数据
    fn read_exif_data(path: &Path) -> ExifData {
        use exif::{Reader, Tag};

        let mut exif_data = ExifData::default();

        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                debug!("无法打开文件读取EXIF: {}", e);
                return exif_data;
            }
        };

        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = Reader::new();

        let exif = match exifreader.read_from_container(&mut bufreader) {
            Ok(e) => e,
            Err(e) => {
                debug!("读取EXIF失败: {}", e);
                return exif_data;
            }
        };

        for field in exif.fields() {
            match field.tag {
                Tag::DateTime | Tag::DateTimeOriginal => {
                    if exif_data.date_time.is_none() {
                        exif_data.date_time =
                            Some(field.display_value().with_unit(&exif).to_string());
                    }
                }
                Tag::Make => {
                    exif_data.camera_make =
                        Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::Model => {
                    exif_data.camera_model =
                        Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::LensModel => {
                    exif_data.lens_model = Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::ISOSpeed => {
                    if let Some(val) = field.value.get_uint(0) {
                        exif_data.iso = Some(val);
                    }
                }
                Tag::FNumber => {
                    exif_data.aperture = Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::ExposureTime => {
                    exif_data.shutter_speed =
                        Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::FocalLength => {
                    exif_data.focal_length =
                        Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::GPSLatitude => {
                    exif_data.gps_latitude =
                        Some(field.display_value().with_unit(&exif).to_string());
                }
                Tag::GPSLongitude => {
                    exif_data.gps_longitude =
                        Some(field.display_value().with_unit(&exif).to_string());
                }
                _ => {}
            }
        }

        exif_data
    }

    /// 获取文件元数据
    fn get_file_metadata(path: &Path) -> (u64, Option<String>) {
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                warn!("无法获取文件元数据: {}", e);
                return (0, None);
            }
        };

        let size = metadata.len();
        let modified = metadata.modified().ok().and_then(|t| {
            use std::time::SystemTime;
            let duration = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
            let secs = duration.as_secs();
            // 格式化为本地时间字符串
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)?;
            Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
        });

        (size, modified)
    }
}

impl Default for InfoPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// 渲染标签-值对
fn render_label_value(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(13.0)
                .color(Color32::LIGHT_GRAY)
                .strong(),
        );
        egui::Label::new(RichText::new(value).size(13.0).color(Color32::WHITE))
            .wrap()
            .ui(ui);
    });
}

/// 格式化文件大小
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return "0 B".to_string();
    }

    let exp = (size as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let size = size as f64 / 1024f64.powi(exp as i32);

    if exp == 0 {
        format!("{:.0} {}", size, UNITS[exp])
    } else {
        format!("{:.2} {}", size, UNITS[exp])
    }
}

/// 格式化路径（截断长路径）
fn format_path(path: &Path) -> String {
    let path_str = path.display().to_string();
    if path_str.len() > 40 {
        format!("...{}", &path_str[path_str.len() - 37..])
    } else {
        path_str
    }
}

/// 格式化相机信息
fn format_camera_info(make: Option<&str>, model: Option<&str>) -> String {
    match (make, model) {
        (Some(m), Some(n)) => {
            let make = m.trim();
            let model = n.trim();
            if model.starts_with(make) {
                model.to_string()
            } else {
                format!("{} {}", make, model)
            }
        }
        (Some(m), None) => m.trim().to_string(),
        (None, Some(n)) => n.trim().to_string(),
        (None, None) => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 基础初始化测试
    // =========================================================================

    #[test]
    fn test_info_panel_new() {
        let panel = InfoPanel::new();
        assert!(!panel.is_visible());
        assert_eq!(panel.width, 280.0);
    }

    #[test]
    fn test_info_panel_with_visibility() {
        let panel_visible = InfoPanel::with_visibility(true);
        assert!(panel_visible.is_visible());

        let panel_hidden = InfoPanel::with_visibility(false);
        assert!(!panel_hidden.is_visible());
    }

    #[test]
    fn test_info_panel_default() {
        let panel: InfoPanel = Default::default();
        assert!(!panel.is_visible());
    }

    // =========================================================================
    // 状态管理测试
    // =========================================================================

    #[test]
    fn test_toggle_visibility() {
        let mut panel = InfoPanel::new();
        assert!(!panel.is_visible());

        panel.toggle();
        assert!(panel.is_visible());

        panel.toggle();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_show_hide() {
        let mut panel = InfoPanel::new();

        panel.show();
        assert!(panel.is_visible());

        panel.hide();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_multiple_toggles() {
        let mut panel = InfoPanel::new();

        for i in 1..=10 {
            panel.toggle();
            assert_eq!(panel.is_visible(), i % 2 == 1);
        }
    }

    // =========================================================================
    // 图像信息设置测试
    // =========================================================================

    #[test]
    fn test_set_image_info() {
        let mut panel = InfoPanel::new();
        let path = Path::new("/test/image.png");
        let dimensions = (1920u32, 1080u32);
        let format = "PNG";

        panel.set_image_info(path, dimensions, format);

        assert!(panel.current_info.is_some());
        assert!(panel.loading_exif);
        assert!(panel.exif_receiver.is_some());

        let info = panel.current_info.unwrap();
        assert_eq!(info.file_name, "image.png");
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
        assert_eq!(info.format, "PNG");
    }

    #[test]
    fn test_set_image_info_clears_previous() {
        let mut panel = InfoPanel::new();

        panel.set_image_info(Path::new("/test/first.png"), (100, 100), "PNG");
        panel.set_image_info(Path::new("/test/second.jpg"), (200, 200), "JPEG");

        let info = panel.current_info.unwrap();
        assert_eq!(info.file_name, "second.jpg");
        assert_eq!(info.width, 200);
    }

    #[test]
    fn test_clear() {
        let mut panel = InfoPanel::new();
        panel.set_image_info(Path::new("/test/image.png"), (100, 100), "PNG");

        panel.clear();

        assert!(panel.current_info.is_none());
        assert!(!panel.loading_exif);
        assert!(panel.exif_receiver.is_none());
    }

    // =========================================================================
    // 辅助函数测试
    // =========================================================================

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(100), "100 B");
        assert_eq!(format_file_size(1023), "1023 B");
    }

    #[test]
    fn test_format_file_size_kilobytes() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_file_size_megabytes() {
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.00 MB");
    }

    #[test]
    fn test_format_path_short() {
        let path = Path::new("/test/image.png");
        assert_eq!(format_path(path), "/test/image.png");
    }

    #[test]
    fn test_format_path_long() {
        let path = Path::new("/very/long/path/to/the/image/file/that/needs/truncating.png");
        let result = format_path(path);
        assert!(result.starts_with("..."));
        assert!(result.len() <= 40);
    }

    #[test]
    fn test_format_camera_info_both() {
        assert_eq!(
            format_camera_info(Some("Canon"), Some("Canon EOS 5D")),
            "Canon EOS 5D"
        );
        assert_eq!(
            format_camera_info(Some("Nikon"), Some("D850")),
            "Nikon D850"
        );
    }

    #[test]
    fn test_format_camera_info_make_only() {
        assert_eq!(format_camera_info(Some("Sony"), None), "Sony");
    }

    #[test]
    fn test_format_camera_info_model_only() {
        assert_eq!(
            format_camera_info(None, Some("iPhone 14 Pro")),
            "iPhone 14 Pro"
        );
    }

    #[test]
    fn test_format_camera_info_none() {
        assert_eq!(format_camera_info(None, None), "Unknown");
    }

    // =========================================================================
    // 数据结构测试
    // =========================================================================

    #[test]
    fn test_image_info_default() {
        let info = ImageInfo::default();
        assert_eq!(info.file_name, "");
        assert_eq!(info.width, 0);
        assert_eq!(info.height, 0);
        assert!(info.exif.is_none());
    }

    #[test]
    fn test_exif_data_default() {
        let exif = ExifData::default();
        assert!(exif.date_time.is_none());
        assert!(exif.camera_model.is_none());
        assert!(exif.iso.is_none());
    }

    #[test]
    fn test_exif_data_clone() {
        let exif = ExifData {
            date_time: Some("2024-01-01".to_string()),
            camera_model: Some("Test Camera".to_string()),
            iso: Some(100),
            ..Default::default()
        };
        let cloned = exif.clone();
        assert_eq!(exif.date_time, cloned.date_time);
        assert_eq!(exif.camera_model, cloned.camera_model);
        assert_eq!(exif.iso, cloned.iso);
    }

    #[test]
    fn test_image_info_clone() {
        let info = ImageInfo {
            path: PathBuf::from("/test.png"),
            file_name: "test.png".to_string(),
            file_size: 1024,
            width: 100,
            height: 100,
            format: "PNG".to_string(),
            ..Default::default()
        };
        let cloned = info.clone();
        assert_eq!(info.file_name, cloned.file_name);
        assert_eq!(info.width, cloned.width);
    }

    // =========================================================================
    // 边界条件测试
    // =========================================================================

    #[test]
    fn test_set_image_info_unknown_filename() {
        let mut panel = InfoPanel::new();
        panel.set_image_info(Path::new("/"), (100, 100), "PNG");

        let info = panel.current_info.unwrap();
        assert_eq!(info.file_name, "Unknown");
    }

    #[test]
    fn test_format_file_size_gigabytes() {
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_file_size_terabytes() {
        assert_eq!(format_file_size(1024u64 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn test_rapid_toggle() {
        let mut panel = InfoPanel::new();

        for _ in 0..100 {
            panel.toggle();
        }

        // 100次toggle后应回到初始状态
        assert!(!panel.is_visible());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_exif_data_full() {
        let exif = ExifData {
            date_time: Some("2024:01:15 10:30:00".to_string()),
            camera_model: Some("Canon EOS R5".to_string()),
            camera_make: Some("Canon".to_string()),
            lens_model: Some("RF 24-70mm F2.8".to_string()),
            iso: Some(100),
            aperture: Some("f/2.8".to_string()),
            shutter_speed: Some("1/250".to_string()),
            focal_length: Some("50 mm".to_string()),
            gps_latitude: Some("39° 54' 15\" N".to_string()),
            gps_longitude: Some("116° 24' 25\" E".to_string()),
        };

        assert!(exif.date_time.is_some());
        assert!(exif.camera_model.is_some());
        assert_eq!(exif.iso, Some(100));
    }

    #[test]
    fn test_image_info_with_exif() {
        let exif = ExifData {
            camera_model: Some("Sony A7IV".to_string()),
            iso: Some(400),
            ..Default::default()
        };

        let info = ImageInfo {
            path: PathBuf::from("/test/photo.jpg"),
            file_name: "photo.jpg".to_string(),
            file_size: 2048,
            modified_time: Some("2024-01-01 12:00:00".to_string()),
            format: "JPEG".to_string(),
            width: 4000,
            height: 3000,
            bit_depth: Some(8),
            color_space: Some("sRGB".to_string()),
            exif: Some(exif),
        };

        assert_eq!(info.file_size, 2048);
        assert_eq!(info.width, 4000);
        assert_eq!(info.height, 3000);
        assert!(info.exif.is_some());
        assert_eq!(info.exif.as_ref().unwrap().iso, Some(400));
    }

    #[test]
    fn test_format_path_exact_40() {
        // 测试恰好40个字符的路径
        let path = Path::new("/123456789/123456789/123456789/123456789.png");
        let result = format_path(path);
        assert_eq!(result.len(), 40);
    }

    #[test]
    fn test_format_path_unicode() {
        let path = Path::new("/图片/照片/test.png");
        let result = format_path(path);
        assert!(result.contains("test.png"));
    }

    #[test]
    fn test_format_camera_info_whitespace() {
        assert_eq!(
            format_camera_info(Some("  Canon  "), Some("  EOS R5  ")),
            "Canon EOS R5"
        );
    }

    #[test]
    fn test_format_camera_info_model_starts_with_make_uppercase() {
        assert_eq!(
            format_camera_info(Some("SONY"), Some("SONY ILCE-7M4")),
            "SONY ILCE-7M4"
        );
    }

    #[test]
    fn test_format_file_size_boundary() {
        // 测试边界值
        assert_eq!(format_file_size(1024 * 1024 - 1), "1024.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_format_file_size_large() {
        // 测试非常大的文件
        assert_eq!(format_file_size(1024u64 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn test_exif_receiver_disconnected() {
        let mut panel = InfoPanel::new();
        let (sender, receiver) = channel::<ExifData>();
        panel.exif_receiver = Some(receiver);
        drop(sender); // 立即丢弃发送者

        // 第一次调用应该处理断开连接
        panel.check_exif_receiver();

        assert!(!panel.loading_exif);
        assert!(panel.exif_receiver.is_none());
    }

    #[test]
    fn test_exif_receiver_empty() {
        let mut panel = InfoPanel::new();
        let (_sender, receiver) = channel::<ExifData>();
        panel.exif_receiver = Some(receiver);
        panel.loading_exif = true;

        // 没有发送数据，应该保持 loading 状态
        panel.check_exif_receiver();

        assert!(panel.loading_exif);
        assert!(panel.exif_receiver.is_some());
    }

    #[test]
    fn test_exif_receiver_success() {
        let mut panel = InfoPanel::new();
        let (sender, receiver) = channel::<ExifData>();
        panel.exif_receiver = Some(receiver);
        panel.loading_exif = true;
        panel.current_info = Some(ImageInfo::default());

        let exif = ExifData {
            camera_model: Some("Test".to_string()),
            ..Default::default()
        };
        sender.send(exif).unwrap();

        panel.check_exif_receiver();

        assert!(!panel.loading_exif);
        assert!(panel.exif_receiver.is_none());
        assert!(panel.current_info.as_ref().unwrap().exif.is_some());
    }

    #[test]
    fn test_info_panel_width_persistence() {
        let mut panel = InfoPanel::new();
        assert_eq!(panel.width, 280.0);

        // 模拟宽度改变（实际在UI中发生）
        panel.width = 300.0;
        assert_eq!(panel.width, 300.0);
    }

    #[test]
    fn test_multiple_info_panel_instances() {
        let panel1 = InfoPanel::with_visibility(true);
        let panel2 = InfoPanel::with_visibility(false);

        assert!(panel1.is_visible());
        assert!(!panel2.is_visible());
    }

    #[test]
    fn test_image_info_with_all_fields_none() {
        let info = ImageInfo {
            path: PathBuf::from("/test.png"),
            file_name: "test.png".to_string(),
            file_size: 0,
            modified_time: None,
            format: "PNG".to_string(),
            width: 100,
            height: 100,
            bit_depth: None,
            color_space: None,
            exif: None,
        };

        assert!(info.modified_time.is_none());
        assert!(info.bit_depth.is_none());
        assert!(info.color_space.is_none());
        assert!(info.exif.is_none());
    }
}
