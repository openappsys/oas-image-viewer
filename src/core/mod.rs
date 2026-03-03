//! Core 模块 - 纯业务逻辑，零外部依赖
//!
//! 本模块包含：
//! - domain: 实体和值对象
//! - ports: 端口接口（trait）
//! - use_cases: 业务用例

pub mod domain;
pub mod ports;
pub mod use_cases;

/// Core 模块版本
pub const VERSION: &str = "0.3.0";

/// 通用结果类型
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core 层错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    ImageNotFound(String),
    InvalidImageFormat(String),
    StorageError(String),
    NavigationError(String),
    ConfigError(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::ImageNotFound(msg) => write!(f, "Image not found: {}", msg),
            CoreError::InvalidImageFormat(msg) => write!(f, "Invalid image format: {}", msg),
            CoreError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            CoreError::NavigationError(msg) => write!(f, "Navigation error: {}", msg),
            CoreError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_display() {
        let err = CoreError::ImageNotFound("test.png".to_string());
        assert!(err.to_string().contains("test.png"));
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(VERSION, "0.3.0");
    }

    // 从旧代码迁移的额外测试

    #[test]
    fn test_core_error_image_not_found() {
        let err = CoreError::ImageNotFound("image.png".to_string());
        assert!(err.to_string().contains("Image not found"));
        assert!(err.to_string().contains("image.png"));
    }

    #[test]
    fn test_core_error_invalid_image_format() {
        let err = CoreError::InvalidImageFormat("unsupported".to_string());
        assert!(err.to_string().contains("Invalid image format"));
        assert!(err.to_string().contains("unsupported"));
    }

    #[test]
    fn test_core_error_storage_error() {
        let err = CoreError::StorageError("disk full".to_string());
        assert!(err.to_string().contains("Storage error"));
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_core_error_navigation_error() {
        let err = CoreError::NavigationError("out of bounds".to_string());
        assert!(err.to_string().contains("Navigation error"));
        assert!(err.to_string().contains("out of bounds"));
    }

    #[test]
    fn test_core_error_config_error() {
        let err = CoreError::ConfigError("invalid key".to_string());
        assert!(err.to_string().contains("Config error"));
        assert!(err.to_string().contains("invalid key"));
    }

    #[test]
    fn test_core_error_equality() {
        let err1 = CoreError::ImageNotFound("a.png".to_string());
        let err2 = CoreError::ImageNotFound("a.png".to_string());
        let err3 = CoreError::ImageNotFound("b.png".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_core_error_clone() {
        let err = CoreError::ImageNotFound("test.png".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_core_error_debug() {
        let err = CoreError::ImageNotFound("test.png".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ImageNotFound"));
    }

    #[test]
    fn test_core_error_error_trait() {
        let err = CoreError::StorageError("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_result_type() {
        let result: Result<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_error() {
        let result: Result<i32> = Err(CoreError::ImageNotFound("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            CoreError::ImageNotFound("test".to_string()),
            CoreError::InvalidImageFormat("test".to_string()),
            CoreError::StorageError("test".to_string()),
            CoreError::NavigationError("test".to_string()),
            CoreError::ConfigError("test".to_string()),
        ];

        assert_eq!(errors.len(), 5);
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
