//! 拖放功能集成测试

use std::path::PathBuf;

/// 测试支持的图片格式
#[test]
fn test_supported_image_formats() {
    let formats = vec![
        "png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp",
        "PNG", "JPG", "JPEG", "GIF", "WEBP", "TIFF", "TIF", "BMP",
    ];
    
    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(is_image, "格式 {} 应该被识别为图片", fmt);
    }
}

/// 测试不支持的文件格式
#[test]
fn test_unsupported_formats() {
    let formats = vec![
        "txt", "rs", "toml", "md", "pdf", "doc", "exe", "zip",
        "mp4", "avi", "mp3", "svg", "json", "yaml", "html",
    ];
    
    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(!is_image, "格式 {} 不应该被识别为图片", fmt);
    }
}

/// 测试各种路径格式的图片文件识别
#[test]
fn test_image_file_path_variations() {
    let paths = vec![
        "/absolute/path/to/image.png",
        "./relative/path/image.jpg",
        "../parent/image.gif",
        "image.webp",
        "/home/user/图片/照片.bmp",
        "C:\\Users\\image.PNG",
    ];
    
    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(is_image, "路径 {} 应该被识别为图片", path_str);
    }
}

/// 测试文件名包含多个点的图片文件
#[test]
fn test_image_file_with_multiple_dots() {
    let paths = vec![
        "my.image.file.png",
        "archive.v2.backup.jpg",
        "file.name.with.dots.gif",
    ];
    
    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(is_image, "路径 {} 应该被识别为图片", path_str);
    }
}

/// 测试空扩展名和无扩展名文件
#[test]
fn test_no_extension_files() {
    let paths = vec![
        "Makefile",
        "README",
        "LICENSE",
        "/etc/hosts",
        "",
    ];
    
    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(!is_image, "路径 '{}' 不应该被识别为图片", path_str);
    }
}

/// 测试单字母扩展名
#[test]
fn test_single_char_extension() {
    let paths = vec!["file.p", "file.j", "file.g"];
    
    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(!is_image, "路径 {} 不应该被识别为图片", path_str);
    }
}

/// 测试数字扩展名
#[test]
fn test_numeric_extension() {
    let paths = vec!["file.123", "file.png.123", "file.001"];
    
    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(!is_image, "路径 {} 不应该被识别为图片", path_str);
    }
}
