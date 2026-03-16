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
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    /// 获取翻译键名（用于 i18n）
    ///
    /// 返回的是翻译键名，而非实际消息文本。
    /// UI 层应使用 get_text(key, language) 获取本地化文本。
    pub fn translation_key(&self) -> &'static str {
        match self {
            CoreError::Gallery(e) => match e {
                GalleryError::EmptyGallery => "error_empty_gallery",
                GalleryError::BoundaryReached { boundary, .. } => match boundary {
                    Boundary::FirstImage => "error_first_image",
                    Boundary::LastImage => "error_last_image",
                },
                GalleryError::ImageNotAvailable { reason, .. } => match reason {
                    UnavailableReason::FileNotFound => "error_file_not_found",
                    UnavailableReason::PermissionDenied => "error_permission_denied",
                    UnavailableReason::Corrupted => "error_corrupted",
                    UnavailableReason::UnsupportedFormat { .. } => "error_unsupported_format",
                    UnavailableReason::FileLocked => "error_file_locked",
                },
                GalleryError::InvalidIndex { .. } => "error_invalid_index",
            },
            CoreError::View(e) => match e {
                ViewError::NoCurrentImage => "error_no_current_image",
                ViewError::ZoomOutOfRange { .. } => "error_zoom_out_of_range",
                ViewError::ImageNotLoaded { .. } => "error_image_not_loaded",
            },
            CoreError::Config(e) => match e {
                ConfigError::ReadFailed { .. } => "error_read_config",
                ConfigError::WriteFailed { .. } => "error_write_config",
                ConfigError::InvalidValue { .. } => "error_config_invalid",
            },
            CoreError::Technical { .. } => "error_technical",
        }
    }

    /// 获取格式化参数（用于带参数的错误消息）
    pub fn format_args(&self) -> Vec<String> {
        match self {
            CoreError::Gallery(e) => match e {
                #[allow(clippy::collapsible_match)]
                GalleryError::ImageNotAvailable {
                    reason: UnavailableReason::UnsupportedFormat { detected },
                    ..
                } => {
                    vec![detected.clone()]
                }
                GalleryError::InvalidIndex { index, total_count } => {
                    vec![index.to_string(), total_count.to_string()]
                }
                _ => vec![],
            },
            #[allow(clippy::collapsible_match)]
            CoreError::Config(ConfigError::InvalidValue { key, .. }) => vec![key.clone()],
            CoreError::Technical { message, .. } => vec![message.clone()],
            _ => vec![],
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
            CoreError::Gallery(e) => write!(f, "[Gallery] {}", e),
            CoreError::View(e) => write!(f, "[View] {}", e),
            CoreError::Config(e) => write!(f, "[Config] {}", e),
            CoreError::Technical { code, message } => write!(f, "[Error {}] {}", code, message),
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
        assert_eq!(err.translation_key(), "error_empty_gallery");
    }

    #[test]
    fn test_core_error_boundary_first() {
        let err: CoreError = GalleryError::BoundaryReached {
            boundary: Boundary::FirstImage,
            current_index: 0,
            total_count: 10,
        }
        .into();
        assert_eq!(err.translation_key(), "error_first_image");
    }

    #[test]
    fn test_core_error_boundary_last() {
        let err: CoreError = GalleryError::BoundaryReached {
            boundary: Boundary::LastImage,
            current_index: 9,
            total_count: 10,
        }
        .into();
        assert_eq!(err.translation_key(), "error_last_image");
    }

    #[test]
    fn test_core_error_image_not_available() {
        let err: CoreError = GalleryError::ImageNotAvailable {
            path: PathBuf::from("test.png"),
            reason: UnavailableReason::FileNotFound,
        }
        .into();
        assert_eq!(err.translation_key(), "error_file_not_found");
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
        assert_eq!(err.translation_key(), "error_unsupported_format");
        let args = err.format_args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "XYZ");
    }

    #[test]
    fn test_core_error_technical() {
        let err = CoreError::technical("IO_ERROR", "disk read failed");
        assert!(!err.is_business_error());
        assert_eq!(err.translation_key(), "error_technical");
        let args = err.format_args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "disk read failed");
    }

    #[test]
    fn test_core_error_display() {
        let err: CoreError = GalleryError::EmptyGallery.into();
        let display = format!("{}", err);
        assert!(display.contains("[Gallery]"));
    }

    #[test]
    fn test_version_constant() {}

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
        assert_eq!(err.translation_key(), "error_no_current_image");
    }

    #[test]
    fn test_config_error_invalid_value() {
        let err: CoreError = ConfigError::InvalidValue {
            key: "thumbnail_size".to_string(),
            value: "abc".to_string(),
            reason: "not a number".to_string(),
        }
        .into();
        assert_eq!(err.translation_key(), "error_config_invalid");
        let args = err.format_args();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "thumbnail_size");
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
        // Verify version format is x.y.z
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);

        for part in parts {
            assert!(part.parse::<u32>().is_ok());
        }
    }

    #[test]
    fn test_format_args_empty() {
        let err: CoreError = GalleryError::EmptyGallery.into();
        let args = err.format_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_translation_key_all_variants() {
        // Gallery errors
        assert_eq!(
            CoreError::from(GalleryError::EmptyGallery).translation_key(),
            "error_empty_gallery"
        );
        assert_eq!(
            CoreError::from(GalleryError::BoundaryReached {
                boundary: Boundary::FirstImage,
                current_index: 0,
                total_count: 5,
            })
            .translation_key(),
            "error_first_image"
        );
        assert_eq!(
            CoreError::from(GalleryError::BoundaryReached {
                boundary: Boundary::LastImage,
                current_index: 4,
                total_count: 5,
            })
            .translation_key(),
            "error_last_image"
        );

        // View errors
        assert_eq!(
            CoreError::from(ViewError::NoCurrentImage).translation_key(),
            "error_no_current_image"
        );
        assert_eq!(
            CoreError::from(ViewError::ZoomOutOfRange {
                requested: 10.0,
                min: 0.1,
                max: 5.0
            })
            .translation_key(),
            "error_zoom_out_of_range"
        );

        // Config errors
        assert_eq!(
            CoreError::from(ConfigError::ReadFailed {
                path: PathBuf::from("test")
            })
            .translation_key(),
            "error_read_config"
        );
    }
}
