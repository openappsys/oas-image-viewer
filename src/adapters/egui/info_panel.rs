//! 信息面板模块 - 显示图片元数据
//!
//! 显示图片的详细信息，包括文件信息、图片属性和EXIF元数据。
//! 支持异步加载EXIF数据，不阻塞UI。

use crate::adapters::egui::i18n::get_text;
use crate::core::domain::Language;
use crate::utils::format_file_size;
use egui::{Context, Frame, RichText, ScrollArea, SidePanel};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use tracing::{debug, warn};

mod helpers;
mod metadata;
mod receiver;

use helpers::{format_camera_info, render_label_value};
use metadata::{get_file_metadata, read_exif_data};
use receiver::{poll_exif_receiver, spawn_exif_loader, ExifReceiveState};

/// 图像信息面板
pub struct InfoPanel {
    visible: bool,
    width: f32,
    current_info: Option<ImageInfo>,
    exif_receiver: Option<Receiver<receiver::ExifLoadResult>>,
    loading_exif: bool,
    exif_request_seq: u64,
    active_exif_request_id: Option<u64>,
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
    /// 镜头制造商
    pub lens_make: Option<String>,
    /// 软件
    pub software: Option<String>,
    /// ISO感光度
    pub iso: Option<u32>,
    /// 光圈值
    pub aperture: Option<String>,
    /// 快门速度
    pub shutter_speed: Option<String>,
    /// 焦距
    pub focal_length: Option<String>,
    /// 曝光补偿
    pub exposure_bias: Option<String>,
    /// 白平衡
    pub white_balance: Option<String>,
    /// 闪光灯
    pub flash: Option<String>,
    /// 测光模式
    pub metering_mode: Option<String>,
    /// 曝光程序
    pub exposure_program: Option<String>,
    /// 曝光模式
    pub exposure_mode: Option<String>,
    /// GPS纬度
    pub gps_latitude: Option<String>,
    /// GPS经度
    pub gps_longitude: Option<String>,
    /// GPS海拔
    pub gps_altitude: Option<String>,
    /// GPS时间
    pub gps_timestamp: Option<String>,
    /// 其他 EXIF 键值
    pub extra_fields: Vec<(String, String)>,
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
            exif_request_seq: 0,
            active_exif_request_id: None,
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
            exif_request_seq: 0,
            active_exif_request_id: None,
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
        let (file_size, modified_time) = get_file_metadata(path);

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
        self.active_exif_request_id = None;
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
            match poll_exif_receiver(receiver) {
                ExifReceiveState::Loaded(result) => {
                    let is_current_request =
                        self.active_exif_request_id == Some(result.request_id);
                    let is_current_image = self
                        .current_info
                        .as_ref()
                        .map(|info| info.path == result.path)
                        .unwrap_or(false);

                    if let Some(ref mut info) = self.current_info {
                        if is_current_request && is_current_image {
                            info.exif = Some(result.exif_data.clone());
                        }
                    }
                    self.loading_exif = false;
                    self.exif_receiver = None;
                    self.active_exif_request_id = None;
                    debug!("EXIF数据加载完成");
                }
                ExifReceiveState::Pending => {}
                ExifReceiveState::Disconnected => {
                    // 通道断开，加载失败
                    self.loading_exif = false;
                    self.exif_receiver = None;
                    self.active_exif_request_id = None;
                    warn!("EXIF加载通道断开");
                }
            }
        }
    }

    /// 渲染信息面板
    /// 返回：如果本帧用户点击了右上角关闭按钮，则返回 true
    pub fn ui(&mut self, ctx: &Context, language: Language) -> bool {
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
                    .fill(ctx.style().visuals.panel_fill.linear_multiply(0.95))
                    .inner_margin(egui::Margin::same(8)),
            )
            .show(ctx, |ui| {
                // 更新宽度（限制在最小和最大之间）
                let new_width = ui.available_width();
                self.width = new_width.clamp(200.0, 400.0);

                // 标题栏
                ui.horizontal(|ui| {
                    // 用 frame 包裹标题，使其与按钮高度一致
                    egui::Frame::new()
                        .outer_margin(egui::Margin {
                            left: 0,
                            right: 0,
                            top: 8,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "📋 {}",
                                    get_text("image_info", language)
                                ))
                                .size(16.0),
                            );
                        });
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
                            self.render_info_content(ui, info, language);
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.label(
                                    RichText::new(get_text("no_image", language))
                                        .color(ui.style().visuals.weak_text_color()),
                                );
                            });
                        }
                    });
            });

        closed_by_user
    }

    /// 渲染信息内容
    fn render_info_content(&self, ui: &mut egui::Ui, info: &ImageInfo, language: Language) {
        // 文件信息部分
        egui::CollapsingHeader::new(format!("📁 {}", get_text("file_info", language)))
            .default_open(true)
            .show(ui, |ui| {
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("file_name", language)),
                    &info.file_name,
                );
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("file_size", language)),
                    &format_file_size(info.file_size),
                );
                if let Some(ref time) = info.modified_time {
                    render_label_value(
                        ui,
                        &format!("{}: ", get_text("modified_time", language)),
                        time,
                    );
                }
            });

        ui.add_space(8.0);

        // 图像信息部分
        egui::CollapsingHeader::new(format!("🖼 {}", get_text("image_info", language)))
            .default_open(true)
            .show(ui, |ui| {
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("format", language)),
                    &info.format,
                );
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("dimensions", language)),
                    &format!("{} x {} px", info.width, info.height),
                );
                let mp = (info.width as f64 * info.height as f64) / 1_000_000.0;
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("megapixels", language)),
                    &format!("{:.2} MP", mp),
                );
                if let Some(depth) = info.bit_depth {
                    render_label_value(
                        ui,
                        &format!("{}: ", get_text("bit_depth", language)),
                        &format!("{} bit", depth),
                    );
                }
                if let Some(ref space) = info.color_space {
                    render_label_value(
                        ui,
                        &format!("{}: ", get_text("color_space", language)),
                        space,
                    );
                }
            });

        ui.add_space(8.0);

        // EXIF信息部分
        egui::CollapsingHeader::new(format!("📷 {}", get_text("exif_info", language)))
            .default_open(true)
            .show(ui, |ui| {
                if self.loading_exif {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(get_text("loading_exif", language))
                                .color(ui.style().visuals.weak_text_color())
                                .size(12.0),
                        );
                    });
                } else if let Some(ref exif) = info.exif {
                    self.render_exif_content(ui, exif, language);
                } else {
                    ui.label(
                        RichText::new(get_text("no_exif", language))
                            .color(ui.style().visuals.weak_text_color())
                            .size(12.0),
                    );
                }
            });
    }

    /// 渲染EXIF内容
    fn render_exif_content(&self, ui: &mut egui::Ui, exif: &ExifData, language: Language) {
        // 相机信息
        if exif.camera_make.is_some() || exif.camera_model.is_some() {
            let camera =
                format_camera_info(exif.camera_make.as_deref(), exif.camera_model.as_deref());
            render_label_value(ui, &format!("{}: ", get_text("camera", language)), &camera);
        }

        if let Some(ref lens) = exif.lens_model {
            render_label_value(ui, &format!("{}: ", get_text("lens", language)), lens);
        }
        if let Some(ref lens_make) = exif.lens_make {
            render_label_value(ui, &format!("{}: ", get_text("lens_make", language)), lens_make);
        }
        if let Some(ref software) = exif.software {
            render_label_value(ui, &format!("{}: ", get_text("software", language)), software);
        }

        if let Some(ref date) = exif.date_time {
            render_label_value(ui, &format!("{}: ", get_text("date_time", language)), date);
        }

        ui.add_space(4.0);

        // 曝光参数
        if let Some(iso) = exif.iso {
            render_label_value(
                ui,
                &format!("{}: ", get_text("iso", language)),
                &iso.to_string(),
            );
        }

        if let Some(ref aperture) = exif.aperture {
            render_label_value(
                ui,
                &format!("{}: ", get_text("aperture", language)),
                aperture,
            );
        }

        if let Some(ref shutter) = exif.shutter_speed {
            render_label_value(ui, &format!("{}: ", get_text("shutter", language)), shutter);
        }

        if let Some(ref focal) = exif.focal_length {
            render_label_value(
                ui,
                &format!("{}: ", get_text("focal_length", language)),
                focal,
            );
        }
        if let Some(ref bias) = exif.exposure_bias {
            render_label_value(
                ui,
                &format!("{}: ", get_text("exposure_bias", language)),
                bias,
            );
        }
        if let Some(ref wb) = exif.white_balance {
            render_label_value(ui, &format!("{}: ", get_text("white_balance", language)), wb);
        }
        if let Some(ref flash) = exif.flash {
            render_label_value(ui, &format!("{}: ", get_text("flash", language)), flash);
        }
        if let Some(ref metering) = exif.metering_mode {
            render_label_value(
                ui,
                &format!("{}: ", get_text("metering_mode", language)),
                metering,
            );
        }
        if let Some(ref program) = exif.exposure_program {
            render_label_value(
                ui,
                &format!("{}: ", get_text("exposure_program", language)),
                program,
            );
        }
        if let Some(ref mode) = exif.exposure_mode {
            render_label_value(
                ui,
                &format!("{}: ", get_text("exposure_mode", language)),
                mode,
            );
        }

        // GPS信息
        if exif.gps_latitude.is_some()
            || exif.gps_longitude.is_some()
            || exif.gps_altitude.is_some()
            || exif.gps_timestamp.is_some()
        {
            ui.add_space(4.0);
            if let Some(ref lat) = exif.gps_latitude {
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("gps_latitude", language)),
                    lat,
                );
            }
            if let Some(ref lon) = exif.gps_longitude {
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("gps_longitude", language)),
                    lon,
                );
            }
            if let Some(ref altitude) = exif.gps_altitude {
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("gps_altitude", language)),
                    altitude,
                );
            }
            if let Some(ref gps_time) = exif.gps_timestamp {
                render_label_value(
                    ui,
                    &format!("{}: ", get_text("gps_timestamp", language)),
                    gps_time,
                );
            }
        }

        if !exif.extra_fields.is_empty() {
            ui.add_space(6.0);
            ui.label(
                RichText::new(get_text("other_exif", language))
                    .size(12.0)
                    .color(ui.style().visuals.weak_text_color()),
            );
            for (key, value) in &exif.extra_fields {
                let localized = localize_extra_exif_label(key, language);
                render_label_value(ui, &format!("{}: ", localized), value);
            }
        }
    }

    /// 异步加载EXIF数据
    fn load_exif_async(&mut self, path: &Path) {
        self.exif_request_seq = self.exif_request_seq.saturating_add(1);
        let request_id = self.exif_request_seq;
        self.active_exif_request_id = Some(request_id);
        self.exif_receiver = Some(spawn_exif_loader(path, request_id, read_exif_data));
    }
}

fn localize_extra_exif_label(tag_name: &str, language: Language) -> String {
    match tag_name {
        "Orientation" => get_text("exif_tag_orientation", language).to_string(),
        "XResolution" => get_text("exif_tag_x_resolution", language).to_string(),
        "YResolution" => get_text("exif_tag_y_resolution", language).to_string(),
        "ResolutionUnit" => get_text("exif_tag_resolution_unit", language).to_string(),
        "ImageDescription" => get_text("exif_tag_image_description", language).to_string(),
        "YCbCrPositioning" => get_text("exif_tag_ycbcr_positioning", language).to_string(),
        "Compression" => get_text("exif_tag_compression", language).to_string(),
        "ExifVersion" => get_text("exif_tag_exif_version", language).to_string(),
        "ComponentsConfiguration" => {
            get_text("exif_tag_components_configuration", language).to_string()
        }
        "LightSource" => get_text("exif_tag_light_source", language).to_string(),
        "FlashpixVersion" => get_text("exif_tag_flashpix_version", language).to_string(),
        "ColorSpace" => get_text("exif_tag_color_space", language).to_string(),
        "PixelXDimension" => get_text("exif_tag_pixel_x_dimension", language).to_string(),
        "PixelYDimension" => get_text("exif_tag_pixel_y_dimension", language).to_string(),
        "InteroperabilityIndex" => {
            get_text("exif_tag_interoperability_index", language).to_string()
        }
        "InteroperabilityVersion" => {
            get_text("exif_tag_interoperability_version", language).to_string()
        }
        "DigitalZoomRatio" => get_text("exif_tag_digital_zoom_ratio", language).to_string(),
        "SceneCaptureType" => get_text("exif_tag_scene_capture_type", language).to_string(),
        "JPEGInterchangeFormat" => {
            get_text("exif_tag_jpeg_interchange_format", language).to_string()
        }
        "JPEGInterchangeFormatLength" => {
            get_text("exif_tag_jpeg_interchange_format_length", language).to_string()
        }
        _ => tag_name.to_string(),
    }
}

impl Default for InfoPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
