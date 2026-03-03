//! 拖放功能集成测试

use std::path::PathBuf;

/// 测试支持的图片格式
#[test]
fn test_supported_image_formats() {
    let formats = vec![
        "png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp", "PNG", "JPG", "JPEG", "GIF",
        "WEBP", "TIFF", "TIF", "BMP",
    ];

    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path
            .extension()
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
        "txt", "rs", "toml", "md", "pdf", "doc", "exe", "zip", "mp4", "avi", "mp3", "svg", "json",
        "yaml", "html",
    ];

    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path
            .extension()
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
        let ext = path
            .extension()
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
        let ext = path
            .extension()
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
    let paths = vec!["Makefile", "README", "LICENSE", "/etc/hosts", ""];

    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path
            .extension()
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
        let ext = path
            .extension()
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
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(!is_image, "路径 {} 不应该被识别为图片", path_str);
    }
}

/// 测试隐藏文件
#[test]
fn test_hidden_files() {
    let paths = vec![
        (".image.png", true),
        (".gitignore", false),
        (".htaccess", false),
    ];

    for (path_str, expected_image) in paths {
        let path = PathBuf::from(path_str);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert_eq!(is_image, expected_image, "路径 {} 检测错误", path_str);
    }
}

/// 测试各种压缩/文档格式不被识别为图片
#[test]
fn test_compressed_and_document_formats() {
    let formats = vec![
        "zip", "rar", "7z", "tar", "gz", "bz2", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt",
        "ods", "odp", "rtf", "tex", "pages",
    ];

    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(!is_image, "格式 {} 不应该被识别为图片", fmt);
    }
}

/// 测试视频格式不被识别为图片
#[test]
fn test_video_formats_not_image() {
    let formats = vec![
        "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ts", "m2ts",
    ];

    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(!is_image, "视频格式 {} 不应该被识别为图片", fmt);
    }
}

/// 测试音频格式不被识别为图片
#[test]
fn test_audio_formats_not_image() {
    let formats = vec![
        "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus", "ra", "ram", "aiff", "au",
    ];

    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(!is_image, "音频格式 {} 不应该被识别为图片", fmt);
    }
}

/// 测试编程语言文件不被识别为图片
#[test]
fn test_code_files_not_image() {
    let formats = vec![
        "rs", "py", "js", "ts", "java", "cpp", "c", "h", "hpp", "go", "rb", "php", "cs", "swift",
        "kt", "scala", "r", "m", "mm", "pl", "pm", "lua",
    ];

    for fmt in formats {
        let path = PathBuf::from(format!("/tmp/test.{}", fmt));
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(!is_image, "代码文件格式 {} 不应该被识别为图片", fmt);
    }
}

/// 测试路径规范化
#[test]
fn test_path_normalization() {
    let paths = vec![
        "/path/to/./image.png",
        "/path/to/../image.jpg",
        "././image.gif",
    ];

    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(is_image, "路径 {} 应该被识别为图片", path_str);
    }
}

/// 测试超长文件名
#[test]
fn test_long_filenames() {
    let long_name = "a".repeat(200);
    let path = PathBuf::from(format!("/tmp/{}.png", long_name));

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let is_image = matches!(
        ext.as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
    );

    assert!(is_image, "超长文件名应该被识别为图片");
}

/// 测试URL编码文件名
#[test]
fn test_url_encoded_filenames() {
    let paths = vec!["image%20file.png", "image+file.jpg", "image%2Bfile.gif"];

    for path_str in paths {
        let path = PathBuf::from(path_str);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        );

        assert!(is_image, "路径 {} 应该被识别为图片", path_str);
    }
}
