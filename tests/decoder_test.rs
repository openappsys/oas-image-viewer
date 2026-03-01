//! 解码器模块测试

use std::path::Path;

/// 测试支持的图像格式检测
#[test]
fn test_image_format_detection() {
    let test_cases = vec![
        ("image.png", true),
        ("image.jpg", true),
        ("image.jpeg", true),
        ("image.gif", true),
        ("image.webp", true),
        ("image.tiff", true),
        ("image.tif", true),
        ("image.bmp", true),
        ("image.txt", false),
        ("image.rs", false),
        ("image.pdf", false),
        ("image", false),
        ("", false),
    ];
    
    for (filename, expected) in test_cases {
        let path = Path::new(filename);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_supported = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert_eq!(is_supported, expected, "Failed for: {}", filename);
    }
}

/// 测试图像格式枚举
#[test]
fn test_image_formats() {
    // 确保所有格式都能被正确识别
    let formats = vec![
        "png", "PNG", "pNg",
        "jpg", "JPG",
        "jpeg", "JPEG",
        "gif", "GIF",
        "webp", "WEBP",
        "tiff", "TIFF",
        "tif", "TIF",
        "bmp", "BMP",
    ];
    
    for ext in formats {
        let path_str = format!("test.{}", ext);
        let path = Path::new(&path_str);
        let detected_ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let is_supported = matches!(
            detected_ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        
        assert!(is_supported, "Format check failed for: {}", ext);
    }
}

/// 测试从内存解码的错误处理
#[test]
fn test_decode_from_memory_error() {
    // 测试无效数据
    let invalid_data = vec![0u8; 100];
    let result = image::load_from_memory(&invalid_data);
    assert!(result.is_err());
}

/// 测试从内存解码空数据
#[test]
fn test_decode_from_memory_empty() {
    let empty_data: Vec<u8> = vec![];
    let result = image::load_from_memory(&empty_data);
    assert!(result.is_err());
}

/// 测试文件扩展名提取的各种边界情况
#[test]
fn test_extension_edge_cases() {
    let cases = vec![
        (".hidden", None),
        ("file.", Some("")),
        ("file", None),
        (".png", None),
    ];
    
    for (path_str, expected_ext) in cases {
        let path = Path::new(path_str);
        let ext = path.extension().and_then(|e| e.to_str());
        assert_eq!(ext, expected_ext, "Extension extraction failed for: {}", path_str);
    }
}
