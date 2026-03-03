//! 缩略图加载器 - 后台异步加载图像缩略图

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use tracing::{debug, error, info};

/// 缩略图加载请求
struct ThumbnailRequest {
    path: PathBuf,
    index: usize,
}

/// 缩略图加载结果
struct ThumbnailResult {
    index: usize,
    texture: Option<egui::TextureHandle>,
}

/// 缩略图加载器 - 在后台线程加载缩略图
pub struct ThumbnailLoader {
    sender: Sender<ThumbnailRequest>,
    receiver: Receiver<ThumbnailResult>,
}

impl ThumbnailLoader {
    pub fn new(ctx: egui::Context) -> Self {
        let (request_tx, request_rx) = channel::<ThumbnailRequest>();
        let (result_tx, result_rx) = channel::<ThumbnailResult>();

        // 启动后台线程处理缩略图加载
        std::thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                let ThumbnailRequest { path, index } = request;

                // 加载缩略图
                let texture = Self::load_thumbnail_internal(&path, &ctx);

                // 发送结果回主线程
                let _ = result_tx.send(ThumbnailResult { index, texture });

                // 触发重绘以更新UI
                ctx.request_repaint();
            }
        });

        Self {
            sender: request_tx,
            receiver: result_rx,
        }
    }

    fn load_thumbnail_internal(path: &PathBuf, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        const THUMBNAIL_SIZE: u32 = 120;

        // 首先尝试使用 image::open 加载
        let img_result = image::open(path);

        let img = match img_result {
            Ok(img) => img,
            Err(e) => {
                debug!("缩略图自动格式检测失败 {:?}: {}，尝试备用方法...", path, e);

                match std::fs::read(path) {
                    Ok(data) => match image::load_from_memory(&data) {
                        Ok(img) => {
                            info!("缩略图使用备用方法成功加载: {:?}", path);
                            img
                        }
                        Err(e2) => {
                            error!("缩略图备用解码也失败 {:?}: {}", path, e2);
                            return None;
                        }
                    },
                    Err(io_err) => {
                        error!("无法读取缩略图文件 {:?}: {}", path, io_err);
                        return None;
                    }
                }
            }
        };

        // 调整为缩略图大小
        let resized = img.resize(
            THUMBNAIL_SIZE,
            THUMBNAIL_SIZE,
            image::imageops::FilterType::Lanczos3,
        );

        let rgba = resized.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.as_raw();

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
        let texture_name = format!("thumb_{}", path.file_name()?.to_string_lossy());

        Some(ctx.load_texture(texture_name, color_image, egui::TextureOptions::LINEAR))
    }

    /// 请求加载缩略图
    pub fn request(&self, index: usize, path: PathBuf) {
        let _ = self.sender.send(ThumbnailRequest { path, index });
    }

    /// 处理已完成的缩略图加载 - 返回处理的数量
    pub fn process_results(
        &self,
        thumbnails: &mut Vec<Option<egui::TextureHandle>>,
        loading_states: &mut [bool],
    ) -> usize {
        let mut count = 0;
        while let Ok(result) = self.receiver.try_recv() {
            if result.index < thumbnails.len() {
                thumbnails[result.index] = result.texture;
                loading_states[result.index] = false;
                count += 1;
            }
        }
        count
    }
}

/// 缩略图缓存 - 管理缩略图纹理
pub struct ThumbnailCache {
    loader: Option<ThumbnailLoader>,
    /// 缩略图纹理缓存
    pub textures: Vec<Option<egui::TextureHandle>>,
    /// 加载状态
    pub loading: Vec<bool>,
    /// 已请求的索引
    requested: Vec<bool>,
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self {
            loader: None,
            textures: Vec::new(),
            loading: Vec::new(),
            requested: Vec::new(),
        }
    }
}

impl ThumbnailCache {
    /// 初始化加载器（需要在有 egui 上下文时调用）
    pub fn init(&mut self, ctx: &egui::Context) {
        if self.loader.is_none() {
            self.loader = Some(ThumbnailLoader::new(ctx.clone()));
        }
    }

    /// 设置图片数量（通常从画廊状态调用）
    pub fn resize(&mut self, count: usize) {
        if self.textures.len() != count {
            self.textures.resize_with(count, || None);
            self.loading.resize(count, false);
            self.requested.resize(count, false);
        }
    }

    /// 请求加载指定索引的缩略图
    pub fn request_thumbnail(&mut self, index: usize, path: &std::path::Path) {
        if index >= self.textures.len() {
            return;
        }

        // 如果已加载或正在加载，则跳过
        if self.textures[index].is_some() || self.loading[index] || self.requested[index] {
            return;
        }

        if let Some(ref loader) = self.loader {
            self.loading[index] = true;
            self.requested[index] = true;
            loader.request(index, path.to_path_buf());
        }
    }

    /// 处理异步加载结果
    pub fn process_results(&mut self) -> usize {
        if let Some(ref loader) = self.loader {
            loader.process_results(&mut self.textures, &mut self.loading)
        } else {
            0
        }
    }

    /// 获取缩略图
    pub fn get(&self, index: usize) -> Option<&egui::TextureHandle> {
        self.textures.get(index)?.as_ref()
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.textures.clear();
        self.loading.clear();
        self.requested.clear();
    }
}
