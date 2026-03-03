//! 工具模块

pub mod errors;
pub mod threading;

use std::path::Path;

/// 格式化文件大小用于显示
pub fn format_file_size(size: u64) -> String {
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

/// 从路径获取文件名，带后备处理
pub fn file_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "未知".to_string())
}

/// 检查路径是否为图像文件
pub fn is_image_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    matches!(
        ext.as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_format_file_size_gigabytes() {
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_file_name_from_path_normal() {
        let path = std::path::Path::new("/home/user/image.png");
        assert_eq!(file_name_from_path(path), "image.png");
    }

    #[test]
    fn test_file_name_from_path_relative() {
        let path = std::path::Path::new("images/photo.jpg");
        assert_eq!(file_name_from_path(path), "photo.jpg");
    }

    #[test]
    fn test_is_image_file_png() {
        assert!(is_image_file(std::path::Path::new("image.png")));
        assert!(is_image_file(std::path::Path::new("photo.jpg")));
        assert!(is_image_file(std::path::Path::new("anim.gif")));
    }

    #[test]
    fn test_is_image_file_not_image() {
        assert!(!is_image_file(std::path::Path::new("doc.txt")));
        assert!(!is_image_file(std::path::Path::new("script.js")));
    }

    // 从旧代码迁移的额外测试

    #[test]
    fn test_format_file_size_terabytes() {
        assert_eq!(format_file_size(1024u64.pow(4)), "1.00 TB");
        assert_eq!(format_file_size(1024u64.pow(4) * 5), "5.00 TB");
    }

    #[test]
    fn test_format_file_size_large_values() {
        // 测试非常大的值不会溢出
        let size = u64::MAX;
        let formatted = format_file_size(size);
        assert!(formatted.contains("TB") || formatted.contains("GB"));
    }

    #[test]
    fn test_format_file_size_exact_boundaries() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_file_name_from_path_empty() {
        let path = std::path::Path::new("");
        assert_eq!(file_name_from_path(path), "未知");
    }

    #[test]
    fn test_file_name_from_path_root() {
        let path = std::path::Path::new("/");
        assert_eq!(file_name_from_path(path), "未知");
    }

    #[test]
    fn test_file_name_from_path_unicode() {
        let path = std::path::Path::new("/home/user/图片.png");
        assert_eq!(file_name_from_path(path), "图片.png");
    }

    #[test]
    fn test_is_image_file_various() {
        assert!(is_image_file(std::path::Path::new("image.jpeg")));
        assert!(is_image_file(std::path::Path::new("image.webp")));
        assert!(is_image_file(std::path::Path::new("image.tiff")));
        assert!(is_image_file(std::path::Path::new("image.tif")));
        assert!(is_image_file(std::path::Path::new("image.bmp")));
    }

    #[test]
    fn test_is_image_file_case_insensitive() {
        assert!(is_image_file(std::path::Path::new("image.PNG")));
        assert!(is_image_file(std::path::Path::new("image.JPG")));
        assert!(is_image_file(std::path::Path::new("image.GIF")));
    }

    #[test]
    fn test_is_image_file_with_paths() {
        assert!(is_image_file(std::path::Path::new("/path/to/image.png")));
        assert!(is_image_file(std::path::Path::new("./relative/image.jpg")));
        assert!(is_image_file(std::path::Path::new("C:\\Users\\image.gif")));
    }

    #[test]
    fn test_is_image_file_no_extension() {
        assert!(!is_image_file(std::path::Path::new("Makefile")));
        assert!(!is_image_file(std::path::Path::new("README")));
        assert!(!is_image_file(std::path::Path::new("LICENSE")));
    }

    #[test]
    fn test_is_image_file_empty_extension() {
        assert!(!is_image_file(std::path::Path::new("file.")));
    }

    #[test]
    fn test_is_image_file_dots_in_name() {
        assert!(is_image_file(std::path::Path::new("my.image.file.png")));
        assert!(is_image_file(std::path::Path::new("archive.v2.jpg")));
    }
}
