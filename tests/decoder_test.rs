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
        let ext = path
            .extension()
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
        "png", "PNG", "pNg", "jpg", "JPG", "jpeg", "JPEG", "gif", "GIF", "webp", "WEBP", "tiff",
        "TIFF", "tif", "TIF", "bmp", "BMP",
    ];

    for ext in formats {
        let path_str = format!("test.{}", ext);
        let path = Path::new(&path_str);
        let detected_ext = path
            .extension()
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
        assert_eq!(
            ext, expected_ext,
            "Extension extraction failed for: {}",
            path_str
        );
    }
}

/// 测试路径包含多个点的文件名
#[test]
fn test_filename_with_multiple_dots() {
    let test_cases = vec![
        ("my.image.file.png", Some("png")),
        ("archive.v2.jpg", Some("jpg")),
        ("file.name.with.dots.gif", Some("gif")),
    ];

    for (filename, expected_ext) in test_cases {
        let path = Path::new(filename);
        let ext = path.extension().and_then(|e| e.to_str());
        assert_eq!(ext, expected_ext, "Failed for: {}", filename);
    }
}

/// 测试路径的各种格式
#[test]
fn test_path_formats() {
    let test_cases = vec![
        "/absolute/path/to/image.png",
        "./relative/path/image.jpg",
        "../parent/image.gif",
        "image.webp",
        "~/home/image.bmp",
    ];

    for path_str in test_cases {
        let path = Path::new(path_str);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let is_image = matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp"
        );
        assert!(is_image, "Should be image: {}", path_str);
    }
}

/// 测试Unicode文件名
#[test]
fn test_unicode_filenames() {
    let test_cases = vec![
        ("图片.png", "png"),
        ("画像.jpg", "jpg"),
        ("이미지.gif", "gif"),
        ("изображение.webp", "webp"),
    ];

    for (filename, expected_ext) in test_cases {
        let path = Path::new(filename);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap();
        assert_eq!(ext, expected_ext, "Failed for: {}", filename);
    }
}

/// 测试数字文件名
#[test]
fn test_numeric_filenames() {
    let test_cases = vec![
        ("001.png", true),
        ("123.jpg", true),
        ("image_2024.gif", true),
        ("2024.01.01.png", true),
    ];

    for (filename, expected_supported) in test_cases {
        let path = Path::new(filename);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        let is_supported = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        assert_eq!(is_supported, expected_supported, "Failed for: {}", filename);
    }
}

/// 测试特殊字符文件名
#[test]
fn test_special_char_filenames() {
    let test_cases = vec![
        ("my-image.png", "png"),
        ("my_image.jpg", "jpg"),
        ("image+test.gif", "gif"),
        ("image(test).webp", "webp"),
    ];

    for (filename, expected_ext) in test_cases {
        let path = Path::new(filename);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap();
        assert_eq!(ext, expected_ext, "Failed for: {}", filename);
    }
}

/// 测试空字符串和特殊路径
#[test]
fn test_empty_and_special_paths() {
    let test_cases = vec![
        ("", false),
        (".", false),
        ("..", false),
        ("/", false),
        ("./", false),
        ("../", false),
    ];

    for (path_str, expected_supported) in test_cases {
        let path = Path::new(path_str);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        let is_supported = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );
        assert_eq!(
            is_supported, expected_supported,
            "Failed for: {:?}",
            path_str
        );
    }
}

/// 测试扩展名大小写混合
#[test]
fn test_mixed_case_extensions() {
    let test_cases = vec![
        ("image.Png", "Png"),
        ("image.jPg", "jPg"),
        ("image.GiF", "GiF"),
        ("image.WEBP", "WEBP"),
    ];

    for (filename, expected_ext) in test_cases {
        let path = Path::new(filename);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap();
        assert_eq!(ext, expected_ext, "Failed for: {}", filename);
    }
}
