//! 图片查看器的错误类型

use std::io;

use thiserror::Error;

/// 主应用程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO错误: {0}")]
    Io(#[from] io::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("图像解码错误: {0}")]
    Decode(#[from] DecoderError),

    #[error("UI错误: {0}")]
    Ui(String),

    #[error("未知错误: {0}")]
    Unknown(String),
}

/// 图像解码器特定错误
#[derive(Error, Debug)]
pub enum DecoderError {
    #[error("不支持的图像格式")]
    UnsupportedFormat,

    #[error("解码图像失败: {0}")]
    DecodeFailed(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("无效的图像数据")]
    InvalidData,
}

/// 图库相关错误
#[derive(Error, Debug)]
pub enum GalleryError {
    #[error("加载图像失败: {0}")]
    LoadFailed(String),

    #[error("缩略图生成失败: {0}")]
    ThumbnailFailed(String),

    #[error("目录未找到: {0}")]
    DirectoryNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_decoder_error_variants() {
        let err1 = DecoderError::UnsupportedFormat;
        assert!(err1.to_string().contains("不支持"));

        let err2 = DecoderError::DecodeFailed("test error".to_string());
        assert!(err2.to_string().contains("test error"));

        let err3 = DecoderError::FileNotFound("/path/to/file".to_string());
        assert!(err3.to_string().contains("/path/to/file"));

        let err4 = DecoderError::InvalidData;
        assert!(err4.to_string().contains("无效"));
    }

    #[test]
    fn test_decoder_error_debug() {
        let err = DecoderError::UnsupportedFormat;
        let debug = format!("{:?}", err);
        assert!(debug.contains("UnsupportedFormat"));
    }

    #[test]
    fn test_gallery_error_variants() {
        let err1 = GalleryError::LoadFailed("image.png".to_string());
        assert!(err1.to_string().contains("image.png"));

        let err2 = GalleryError::ThumbnailFailed("resize error".to_string());
        assert!(err2.to_string().contains("resize error"));

        let err3 = GalleryError::DirectoryNotFound("/path/to/dir".to_string());
        assert!(err3.to_string().contains("/path/to/dir"));
    }

    #[test]
    fn test_gallery_error_debug() {
        let err = GalleryError::LoadFailed("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("LoadFailed"));
    }

    #[test]
    fn test_app_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::from(io_err);
        
        match app_err {
            AppError::Io(_) => {},
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_app_error_from_decoder() {
        let decoder_err = DecoderError::UnsupportedFormat;
        let app_err = AppError::from(decoder_err);
        
        match app_err {
            AppError::Decode(_) => {},
            _ => panic!("Expected Decode error"),
        }
    }

    #[test]
    fn test_app_error_variants() {
        let err1 = AppError::Io(io::Error::new(io::ErrorKind::Other, "io error"));
        assert!(err1.to_string().contains("IO错误"));

        let err2 = AppError::Config("invalid config".to_string());
        assert!(err2.to_string().contains("配置错误"));
        assert!(err2.to_string().contains("invalid config"));

        let err3 = AppError::Ui("ui error".to_string());
        assert!(err3.to_string().contains("UI错误"));

        let err4 = AppError::Unknown("unknown".to_string());
        assert!(err4.to_string().contains("未知错误"));
    }

    #[test]
    fn test_error_display_messages() {
        let decoder_err = DecoderError::DecodeFailed("corrupt data".to_string());
        assert_eq!(
            decoder_err.to_string(),
            "解码图像失败: corrupt data"
        );

        let gallery_err = GalleryError::ThumbnailFailed("timeout".to_string());
        assert_eq!(
            gallery_err.to_string(),
            "缩略图生成失败: timeout"
        );
    }
}
