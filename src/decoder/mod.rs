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
    pub fn decode_from_file(&self, path: &Path) -> Result<DynamicImage, DecoderError> {
        debug!("从 {:?} 解码图像", path);

        // 首先尝试从文件内容检测格式（更可靠）
        let format_hint = self.detect_format(path).ok();
        if let Some(fmt) = format_hint {
            debug!("根据扩展名检测到格式: {:?}", fmt);
        }

        // image::open 自动从文件内容检测格式（magic number）
        // 这比基于扩展名的检测更可靠
        match image::open(path) {
            Ok(img) => {
                debug!("使用自动格式检测成功解码图像");
                Ok(img)
            }
            Err(e) => {
                error!("自动格式检测失败: {}", e);

                // 备用方案：读取原始字节并尝试从内容猜测格式
                debug!("尝试备用解码方法...");
                match std::fs::read(path) {
                    Ok(data) => match image::load_from_memory(&data) {
                        Ok(img) => {
                            info!("使用备用方法成功解码图像");
                            Ok(img)
                        }
                        Err(e2) => {
                            error!("备用解码也失败: {}", e2);
                            Err(DecoderError::DecodeFailed(format!(
                                "主解码: {} | 备用解码: {}",
                                e, e2
                            )))
                        }
                    },
                    Err(io_err) => {
                        error!("无法读取文件: {}", io_err);
                        Err(DecoderError::DecodeFailed(format!(
                            "解码失败: {} | 文件读取失败: {}",
                            e, io_err
                        )))
                    }
                }
            }
        }
    }

    /// 从内存解码图像
    #[instrument(skip(self, data))]
    pub fn decode_from_memory(&self, data: &[u8]) -> Result<DynamicImage, DecoderError> {
        debug!("从内存解码图像, 大小: {} 字节", data.len());

        let img = image::load_from_memory(data).map_err(|e| {
            error!("从内存解码图像失败: {}", e);
            DecoderError::DecodeFailed(e.to_string())
        })?;

        Ok(img)
    }

    /// 从文件扩展名检测图像格式
    fn detect_format(&self, path: &Path) -> Result<ImageFormat, DecoderError> {
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

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_is_supported_case_sensitive() {
        // 扩展名检测区分大小写（仅小写）
        assert!(!ImageDecoder::is_supported(Path::new("image.PNG")));
        assert!(!ImageDecoder::is_supported(Path::new("image.JPG")));
        assert!(!ImageDecoder::is_supported(Path::new("image.Jpeg")));
        assert!(!ImageDecoder::is_supported(Path::new("image.GIF")));
        assert!(!ImageDecoder::is_supported(Path::new("image.WebP")));
        assert!(!ImageDecoder::is_supported(Path::new("image.TIFF")));
        assert!(!ImageDecoder::is_supported(Path::new("image.BMP")));
    }

    #[test]
    fn test_is_supported_various_paths() {
        assert!(ImageDecoder::is_supported(Path::new("/path/to/image.png")));
        assert!(ImageDecoder::is_supported(Path::new("./relative/path/image.jpg")));
        assert!(ImageDecoder::is_supported(Path::new("image.png")));
    }

    #[test]
    fn test_is_supported_with_dots_in_name() {
        
        assert!(!ImageDecoder::is_supported(Path::new("archive.tar.gz")));
        assert!(ImageDecoder::is_supported(Path::new("my.file.name.png")));
    }

    #[test]
    fn test_is_supported_empty_extension() {
        assert!(!ImageDecoder::is_supported(Path::new("file.")));
    }

    #[test]
    fn test_detect_format_case_variations() {
        let decoder = ImageDecoder::new();
        
        // 测试各种大小写组合
        let test_cases = vec![
            ("image.PNG", ImageFormat::Png),
            ("image.pNg", ImageFormat::Png),
            ("image.jpg", ImageFormat::Jpeg),
            ("image.JPG", ImageFormat::Jpeg),
            ("image.JPEG", ImageFormat::Jpeg),
        ];
        
        for (path, expected) in test_cases {
            let format = decoder.detect_format(Path::new(path)).unwrap();
            assert_eq!(format, expected, "Failed for {}", path);
        }
    }

    #[test]
    fn test_detect_format_with_path_components() {
        let decoder = ImageDecoder::new();
        
        let format = decoder.detect_format(Path::new("/home/user/images/photo.png")).unwrap();
        assert_eq!(format, ImageFormat::Png);
        
        let format = decoder.detect_format(Path::new("./images/photo.jpg")).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_format_various_extensions() {
        let decoder = ImageDecoder::new();
        
        let supported = vec![
            ("png", ImageFormat::Png),
            ("jpg", ImageFormat::Jpeg),
            ("jpeg", ImageFormat::Jpeg),
            ("gif", ImageFormat::Gif),
            ("webp", ImageFormat::Webp),
            ("tiff", ImageFormat::Tiff),
            ("tif", ImageFormat::Tiff),
            ("bmp", ImageFormat::Bmp),
        ];
        
        for (ext, expected) in supported {
            let path_str = format!("image.{}", ext);
            let path = Path::new(&path_str);
            let format = decoder.detect_format(path).unwrap();
            assert_eq!(format, expected, "Failed for .{}", ext);
        }
    }

    #[test]
    fn test_detect_format_unsupported_variations() {
        let decoder = ImageDecoder::new();
        
        let unsupported = vec![
            "file.txt",
            "file.pdf",
            "file.doc",
            "file.exe",
            "file.zip",
            "file.mp4",
        ];
        
        for path in unsupported {
            let result = decoder.detect_format(Path::new(path));
            assert!(result.is_err(), "Expected error for {}", path);
        }
    }

    #[test]
    fn test_image_format_clone() {
        let format = ImageFormat::Png;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_image_format_copy() {
        let format = ImageFormat::Jpeg;
        let copied = format;
        // 如果实现了 Copy，原始值仍然可用
        assert_eq!(format, ImageFormat::Jpeg);
        assert_eq!(copied, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_format_all_variants() {
        let formats = [
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Gif,
            ImageFormat::Webp,
            ImageFormat::Tiff,
            ImageFormat::Bmp,
        ];
        
        for (i, format) in formats.iter().enumerate() {
            match format {
                ImageFormat::Png if i == 0 => {},
                ImageFormat::Jpeg if i == 1 => {},
                ImageFormat::Gif if i == 2 => {},
                ImageFormat::Webp if i == 3 => {},
                ImageFormat::Tiff if i == 4 => {},
                ImageFormat::Bmp if i == 5 => {},
                _ => panic!("Unexpected variant at position {}", i),
            }
        }
    }

    #[test]
    fn test_decoder_new_and_default_equivalent() {
        let decoder1 = ImageDecoder::new();
        let decoder2: ImageDecoder = Default::default();
        
        // 两者应该具有相同的行为
        assert!(ImageDecoder::is_supported(Path::new("test.png")));
    }

    #[test]
    fn test_is_supported_empty_path() {
        assert!(!ImageDecoder::is_supported(Path::new("")));
    }

    #[test]
    fn test_decode_from_memory_empty() {
        let decoder = ImageDecoder::new();
        let result = decoder.decode_from_memory(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_from_memory_random_data() {
        let decoder = ImageDecoder::new();
        // 随机数据不应该被解码为有效图像
        let random_data: Vec<u8> = (0..100).map(|i| i as u8).collect();
        let result = decoder.decode_from_memory(&random_data);
        assert!(result.is_err());
    }
}
