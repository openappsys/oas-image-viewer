//! 领域错误类型 - 业务视角的错误定义
//!
//! 这些错误表示业务规则的违反，而非技术故障。
//! 每个错误都包含足够的上下文信息，以便在 UI 层翻译为用户友好的消息。

use std::path::PathBuf;

/// 图库相关错误
#[derive(Debug, Clone, PartialEq)]
pub enum GalleryError {
    /// 图库为空
    EmptyGallery,

    /// 图片不可用
    ImageNotAvailable {
        path: PathBuf,
        reason: UnavailableReason,
    },

    /// 已到达图库边界
    BoundaryReached {
        boundary: Boundary,
        current_index: usize,
        total_count: usize,
    },

    /// 无效的索引
    InvalidIndex { index: usize, total_count: usize },
}

/// 图片不可用的原因
#[derive(Debug, Clone, PartialEq)]
pub enum UnavailableReason {
    /// 文件不存在
    FileNotFound,
    /// 权限不足
    PermissionDenied,
    /// 文件损坏
    Corrupted,
    /// 不支持的格式
    UnsupportedFormat { detected: String },
    /// 文件被其他程序锁定
    FileLocked,
}

/// 图库边界
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Boundary {
    /// 第一张图片
    FirstImage,
    /// 最后一张图片
    LastImage,
}

/// 视图相关错误
#[derive(Debug, Clone, PartialEq)]
pub enum ViewError {
    /// 没有当前图片可显示
    NoCurrentImage,

    /// 缩放超出范围
    ZoomOutOfRange { requested: f32, min: f32, max: f32 },

    /// 图片尚未加载
    ImageNotLoaded { path: PathBuf },
}

/// 配置相关错误
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// 配置文件读取失败
    ReadFailed { path: PathBuf },

    /// 配置文件写入失败
    WriteFailed { path: PathBuf },

    /// 配置项无效
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
}

impl std::fmt::Display for GalleryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GalleryError::EmptyGallery => {
                write!(f, "图库为空")
            }
            GalleryError::ImageNotAvailable { path, reason } => {
                write!(f, "图片不可用: {} (原因: {:?})", path.display(), reason)
            }
            GalleryError::BoundaryReached { boundary, .. } => match boundary {
                Boundary::FirstImage => write!(f, "已经是第一张图片"),
                Boundary::LastImage => write!(f, "已经是最后一张图片"),
            },
            GalleryError::InvalidIndex { index, total_count } => {
                write!(f, "无效的图片索引: {} (总数: {})", index, total_count)
            }
        }
    }
}

impl std::fmt::Display for ViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewError::NoCurrentImage => {
                write!(f, "没有当前图片")
            }
            ViewError::ZoomOutOfRange {
                requested,
                min,
                max,
            } => {
                write!(f, "缩放级别 {} 超出范围 [{}, {}]", requested, min, max)
            }
            ViewError::ImageNotLoaded { path } => {
                write!(f, "图片尚未加载: {}", path.display())
            }
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadFailed { path } => {
                write!(f, "无法读取配置文件: {}", path.display())
            }
            ConfigError::WriteFailed { path } => {
                write!(f, "无法写入配置文件: {}", path.display())
            }
            ConfigError::InvalidValue { key, value, reason } => {
                write!(f, "配置项 '{}' 的值 '{}' 无效: {}", key, value, reason)
            }
        }
    }
}

impl std::error::Error for GalleryError {}
impl std::error::Error for ViewError {}
impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_error_empty() {
        let err = GalleryError::EmptyGallery;
        assert!(err.to_string().contains("空"));
    }

    #[test]
    fn test_gallery_error_boundary() {
        let err = GalleryError::BoundaryReached {
            boundary: Boundary::FirstImage,
            current_index: 0,
            total_count: 10,
        };
        assert!(err.to_string().contains("第一张"));
    }

    #[test]
    fn test_view_error_zoom() {
        let err = ViewError::ZoomOutOfRange {
            requested: 10.0,
            min: 0.1,
            max: 5.0,
        };
        assert!(err.to_string().contains("10"));
        assert!(err.to_string().contains("0.1"));
    }
}
