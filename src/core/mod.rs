//! Core 模块 - 纯业务逻辑，零外部依赖
//!
//! 本模块包含：
//! - domain: 实体和值对象
//! - ports: 端口接口（trait）
//! - use_cases: 业务用例

pub mod domain;
pub mod ports;
pub mod use_cases;

// 重新导出领域错误类型
pub use domain::{Boundary, ConfigError, GalleryError, UnavailableReason, ViewError};

/// Core 模块版本
pub const VERSION: &str = "0.3.0";

/// 通用结果类型
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core 层统一错误类型
///
/// 包含业务错误（GalleryError, ViewError, ConfigError）和技术错误
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// 图库相关错误
    Gallery(GalleryError),
    /// 视图相关错误
    View(ViewError),
    /// 配置相关错误
    Config(ConfigError),
    /// 技术错误（用于向后兼容和基础设施错误）
    Technical { code: String, message: String },
}

impl CoreError {
    /// 创建技术错误
    pub fn technical(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Technical {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 判断是否为用户可理解的业务错误
    pub fn is_business_error(&self) -> bool {
        matches!(self, Self::Gallery(_) | Self::View(_) | Self::Config(_))
    }

    /// 获取用户友好的错误消息（中文）
    pub fn user_message(&self) -> String {
        match self {
            CoreError::Gallery(e) => match e {
                GalleryError::EmptyGallery => "图库为空".to_string(),
                GalleryError::BoundaryReached { boundary, .. } => match boundary {
                    Boundary::FirstImage => "已经是第一张图片了".to_string(),
                    Boundary::LastImage => "已经是最后一张图片了".to_string(),
                },
                GalleryError::ImageNotAvailable { reason, .. } => match reason {
                    UnavailableReason::FileNotFound => "图片文件已移动或删除".to_string(),
                    UnavailableReason::PermissionDenied => "无法访问该图片（权限不足）".to_string(),
                    UnavailableReason::Corrupted => "图片文件已损坏".to_string(),
                    UnavailableReason::UnsupportedFormat { detected } => {
                        format!("不支持的图片格式: {}", detected)
                    }
                    UnavailableReason::FileLocked => "图片文件被其他程序占用".to_string(),
                },
                GalleryError::InvalidIndex { .. } => "无效的图片位置".to_string(),
            },
            CoreError::View(e) => match e {
                ViewError::NoCurrentImage => "没有可显示的图片".to_string(),
                ViewError::ZoomOutOfRange { .. } => "缩放级别超出范围".to_string(),
                ViewError::ImageNotLoaded { .. } => "图片正在加载中".to_string(),
            },
            CoreError::Config(e) => match e {
                ConfigError::ReadFailed { .. } => "无法读取配置".to_string(),
                ConfigError::WriteFailed { .. } => "无法保存配置".to_string(),
                ConfigError::InvalidValue { key, .. } => format!("配置项 '{}' 无效", key),
            },
            CoreError::Technical { message, .. } => format!("发生错误: {}", message),
        }
    }
}

impl From<GalleryError> for CoreError {
    fn from(err: GalleryError) -> Self {
        CoreError::Gallery(err)
    }
}

impl From<ViewError> for CoreError {
    fn from(err: ViewError) -> Self {
        CoreError::View(err)
    }
}

impl From<ConfigError> for CoreError {
    fn from(err: ConfigError) -> Self {
        CoreError::Config(err)
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Gallery(e) => write!(f, "[图库错误] {}", e),
            CoreError::View(e) => write!(f, "[视图错误] {}", e),
            CoreError::Config(e) => write!(f, "[配置错误] {}", e),
            CoreError::Technical { code, message } => write!(f, "[技术错误 {}] {}", code, message),
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_core_error_gallery() {
        let err: CoreError = GalleryError::EmptyGallery.into();
        assert!(matches!(err, CoreError::Gallery(_)));
        assert!(err.is_business_error());
        assert_eq!(err.user_message(), "图库为空");
    }

    #[test]
    fn test_core_error_boundary_first() {
        let err: CoreError = GalleryError::BoundaryReached {
            boundary: Boundary::FirstImage,
            current_index: 0,
            total_count: 10,
        }
        .into();
        assert_eq!(err.user_message(), "已经是第一张图片了");
    }

    #[test]
    fn test_core_error_boundary_last() {
        let err: CoreError = GalleryError::BoundaryReached {
            boundary: Boundary::LastImage,
            current_index: 9,
            total_count: 10,
        }
        .into();
        assert_eq!(err.user_message(), "已经是最后一张图片了");
    }

    #[test]
    fn test_core_error_image_not_available() {
        let err: CoreError = GalleryError::ImageNotAvailable {
            path: PathBuf::from("test.png"),
            reason: UnavailableReason::FileNotFound,
        }
        .into();
        assert_eq!(err.user_message(), "图片文件已移动或删除");
    }

    #[test]
    fn test_core_error_unsupported_format() {
        let err: CoreError = GalleryError::ImageNotAvailable {
            path: PathBuf::from("test.xyz"),
            reason: UnavailableReason::UnsupportedFormat {
                detected: "XYZ".to_string(),
            },
        }
        .into();
        assert!(err.user_message().contains("XYZ"));
    }

    #[test]
    fn test_core_error_technical() {
        let err = CoreError::technical("IO_ERROR", "磁盘读取失败");
        assert!(!err.is_business_error());
        assert!(err.user_message().contains("磁盘读取失败"));
    }

    #[test]
    fn test_core_error_display() {
        let err: CoreError = GalleryError::EmptyGallery.into();
        let display = format!("{}", err);
        assert!(display.contains("图库错误"));
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(VERSION, "0.3.0");
    }

    #[test]
    fn test_core_error_equality() {
        let err1: CoreError = GalleryError::EmptyGallery.into();
        let err2: CoreError = GalleryError::EmptyGallery.into();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_result_type() {
        let result: Result<i32> = Ok(42);
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn test_result_type_error() {
        let result: Result<i32> = Err(GalleryError::EmptyGallery.into());
        assert!(result.is_err());
    }

    #[test]
    fn test_view_error_no_current_image() {
        let err: CoreError = ViewError::NoCurrentImage.into();
        assert_eq!(err.user_message(), "没有可显示的图片");
    }

    #[test]
    fn test_config_error_invalid_value() {
        let err: CoreError = ConfigError::InvalidValue {
            key: "thumbnail_size".to_string(),
            value: "abc".to_string(),
            reason: "不是数字".to_string(),
        }
        .into();
        assert!(err.user_message().contains("thumbnail_size"));
    }

    #[test]
    fn test_all_error_variants() {
        let errors: Vec<CoreError> = vec![
            GalleryError::EmptyGallery.into(),
            ViewError::NoCurrentImage.into(),
            ConfigError::ReadFailed {
                path: PathBuf::from("test"),
            }
            .into(),
            CoreError::technical("TEST", "test"),
        ];

        assert_eq!(errors.len(), 4);
        assert!(errors[0].is_business_error());
        assert!(errors[1].is_business_error());
        assert!(errors[2].is_business_error());
        assert!(!errors[3].is_business_error());
    }

    #[test]
    fn test_version_format() {
        // 验证版本号格式是 x.y.z
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);

        for part in parts {
            assert!(part.parse::<u32>().is_ok());
        }
    }
}
