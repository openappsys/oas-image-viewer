//! 图像解码器模块

use std::path::Path;

use image::DynamicImage;
use tracing::{debug, error, info, instrument};

use crate::utils::errors::DecoderError;

/// 支持的图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Tiff,
    Bmp,
}

/// 图像解码器
pub struct ImageDecoder;

impl ImageDecoder {
    /// 创建新的图像解码器
    pub fn new() -> Self {
        Self
    }

    /// 从文件路径解码图像
    #[instrument(skip(self, path))]
    pub fn decode_from_file(&self,
        path: &Path,
    ) -> Result<DynamicImage, DecoderError> {
        debug!("从 {:?} 解码图像", path);

        // Try to detect format from file content first (more reliable)
        let format_hint = self.detect_format(path).ok();
        if let Some(fmt) = format_hint {
            debug!("根据扩展名检测到格式: {:?}", fmt);
        }

        // image::open automatically detects format from file content (magic number)
        // This is more reliable than extension-based detection
        match image::open(path) {
            Ok(img) => {
                debug!("使用自动格式检测成功解码图像");
                Ok(img)
            }
            Err(e) => {
                error!("自动格式检测失败: {}", e);
                
                // Fallback: read raw bytes and try to guess format from content
                debug!("尝试备用解码方法...");
                match std::fs::read(path) {
                    Ok(data) => {
                        match image::load_from_memory(&data) {
                            Ok(img) => {
                                info!("使用备用方法成功解码图像");
                                Ok(img)
                            }
                            Err(e2) => {
                                error!("备用解码也失败: {}", e2);
                                Err(DecoderError::DecodeFailed(format!(
                                    "主解码: {} | 备用解码: {}", e, e2
                                )))
                            }
                        }
                    }
                    Err(io_err) => {
                        error!("无法读取文件: {}", io_err);
                        Err(DecoderError::DecodeFailed(format!(
                            "解码失败: {} | 文件读取失败: {}", e, io_err
                        )))
                    }
                }
            }
        }
    }

    /// 从内存解码图像
    #[instrument(skip(self, data))]
    pub fn decode_from_memory(
        &self,
        data: &[u8],
    ) -> Result<DynamicImage, DecoderError> {
        debug!("从内存解码图像, 大小: {} 字节", data.len());

        let img = image::load_from_memory(data).map_err(|e| {
            error!("从内存解码图像失败: {}", e);
            DecoderError::DecodeFailed(e.to_string())
        })?;

        Ok(img)
    }

    /// 从文件扩展名检测图像格式
    fn detect_format(&self,
        path: &Path,
    ) -> Result<ImageFormat, DecoderError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| DecoderError::UnsupportedFormat)?;

        match ext.to_lowercase().as_str() {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "gif" => Ok(ImageFormat::Gif),
            "webp" => Ok(ImageFormat::Webp),
            "tiff" | "tif" => Ok(ImageFormat::Tiff),
            "bmp" => Ok(ImageFormat::Bmp),
            _ => Err(DecoderError::UnsupportedFormat),
        }
    }

    /// 检查文件是否是支持的图像
    pub fn is_supported(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        )
    }
}

impl Default for ImageDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_decoder_new() {
        let _decoder = ImageDecoder::new();
    }

    #[test]
    fn test_image_decoder_default() {
        let _decoder: ImageDecoder = Default::default();
    }

    #[test]
    fn test_is_supported_png() {
        assert!(ImageDecoder::is_supported(Path::new("image.png")));
    }

    #[test]
    fn test_is_supported_jpg() {
        assert!(ImageDecoder::is_supported(Path::new("image.jpg")));
        assert!(ImageDecoder::is_supported(Path::new("image.jpeg")));
    }

    #[test]
    fn test_is_supported_gif() {
        assert!(ImageDecoder::is_supported(Path::new("anim.gif")));
    }

    #[test]
    fn test_is_supported_webp() {
        assert!(ImageDecoder::is_supported(Path::new("pic.webp")));
    }

    #[test]
    fn test_is_supported_tiff() {
        assert!(ImageDecoder::is_supported(Path::new("scan.tiff")));
        assert!(ImageDecoder::is_supported(Path::new("scan.tif")));
    }

    #[test]
    fn test_is_supported_bmp() {
        assert!(ImageDecoder::is_supported(Path::new("image.bmp")));
    }

    #[test]
    fn test_is_supported_not_supported() {
        assert!(!ImageDecoder::is_supported(Path::new("file.txt")));
        assert!(!ImageDecoder::is_supported(Path::new("script.js")));
    }

    #[test]
    fn test_is_supported_no_extension() {
        assert!(!ImageDecoder::is_supported(Path::new("README")));
    }

    #[test]
    fn test_image_format_enum() {
        let formats = vec![
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Gif,
            ImageFormat::Webp,
            ImageFormat::Tiff,
            ImageFormat::Bmp,
        ];
        assert_eq!(formats.len(), 6);
    }

    #[test]
    fn test_image_format_equality() {
        assert_eq!(ImageFormat::Png, ImageFormat::Png);
        assert_ne!(ImageFormat::Png, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_format_debug() {
        let format = ImageFormat::Png;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Png"));
    }

    #[test]
    fn test_detect_format_png() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("image.png")).unwrap();
        assert_eq!(format, ImageFormat::Png);
    }

    #[test]
    fn test_detect_format_jpg() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("image.jpg")).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_format_gif() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("anim.gif")).unwrap();
        assert_eq!(format, ImageFormat::Gif);
    }

    #[test]
    fn test_detect_format_webp() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("pic.webp")).unwrap();
        assert_eq!(format, ImageFormat::Webp);
    }

    #[test]
    fn test_detect_format_bmp() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("image.bmp")).unwrap();
        assert_eq!(format, ImageFormat::Bmp);
    }

    #[test]
    fn test_detect_format_tiff() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("scan.tiff")).unwrap();
        assert_eq!(format, ImageFormat::Tiff);
        let format = decoder.detect_format(Path::new("scan.tif")).unwrap();
        assert_eq!(format, ImageFormat::Tiff);
    }

    #[test]
    fn test_detect_format_unsupported() {
        let decoder = ImageDecoder::new();
        let result = decoder.detect_format(Path::new("file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_format_no_extension() {
        let decoder = ImageDecoder::new();
        let result = decoder.detect_format(Path::new("README"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_format_uppercase() {
        let decoder = ImageDecoder::new();
        let format = decoder.detect_format(Path::new("image.PNG")).unwrap();
        assert_eq!(format, ImageFormat::Png);
        let format = decoder.detect_format(Path::new("image.JPG")).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_decode_from_memory_invalid() {
        let decoder = ImageDecoder::new();
        let invalid_data = vec![0u8; 100];
        let result = decoder.decode_from_memory(&invalid_data);
        assert!(result.is_err());
    }
}
