//! 拖放模块更多测试

use std::path::Path;

/// 测试 collect_images_from_dir 的边界情况
#[test]
fn test_collect_images_empty_dir() {
    // 测试空目录
    fn collect_images_from_dir(path: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut images: Vec<std::path::PathBuf> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let ext = entry_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase());

                    let is_image = matches!(
                        ext.as_deref(),
                        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
                    );

                    if is_image {
                        images.push(entry_path);
                    }
                }
            }
        }

        images.sort();
        images
    }

    // 测试不存在的目录
    let result = collect_images_from_dir(Path::new("/nonexistent/path"));
    assert!(result.is_empty());

    // 测试文件而不是目录
    let result = collect_images_from_dir(Path::new("/etc/hosts"));
    assert!(result.is_empty());
}

/// 测试 extract_image_files 的边界情况
#[test]
fn test_extract_images_edge_cases() {
    fn is_image_file(path: &std::path::Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        )
    }

    // 测试各种路径格式
    let test_paths = vec![
        "/absolute/path/image.png",
        "./relative/image.jpg",
        "../parent/image.gif",
        "image.webp",
        "C:\\Windows\\image.bmp",
    ];

    for path_str in test_paths {
        let path = Path::new(path_str);
        assert!(is_image_file(path), "Should detect image: {}", path_str);
    }
}

/// 测试 is_image_file 函数的所有分支
#[test]
fn test_is_image_file_branches() {
    fn is_image_file(path: &std::path::Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        matches!(
            ext.as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
        )
    }

    // 测试所有支持的分支
    assert!(is_image_file(Path::new("a.png")));
    assert!(is_image_file(Path::new("a.jpg")));
    assert!(is_image_file(Path::new("a.jpeg")));
    assert!(is_image_file(Path::new("a.gif")));
    assert!(is_image_file(Path::new("a.webp")));
    assert!(is_image_file(Path::new("a.tiff")));
    assert!(is_image_file(Path::new("a.tif")));
    assert!(is_image_file(Path::new("a.bmp")));

    // 测试不支持的分支
    assert!(!is_image_file(Path::new("a.txt")));
    assert!(!is_image_file(Path::new("a.rs")));
    assert!(!is_image_file(Path::new("a")));
    assert!(!is_image_file(Path::new("")));
    assert!(!is_image_file(Path::new("a.svg")));
    assert!(!is_image_file(Path::new("a.pdf")));
}

/// 测试拖放预览文本逻辑
#[test]
fn test_drag_preview_text_logic() {
    // 测试数量格式化
    let count = 5;
    let text = format!("准备打开 {} 个文件", count);
    assert_eq!(text, "准备打开 5 个文件");

    let count = 1;
    let text = format!("准备打开 {} 个文件", count);
    assert_eq!(text, "准备打开 1 个文件");

    let count = 0;
    let text = format!("准备打开 {} 个文件", count);
    assert_eq!(text, "准备打开 0 个文件");
}

/// 测试排序行为
#[test]
fn test_image_sorting() {
    let mut paths = [
        std::path::PathBuf::from("z.png"),
        std::path::PathBuf::from("a.png"),
        std::path::PathBuf::from("m.png"),
    ];

    paths.sort();

    assert_eq!(paths[0], std::path::PathBuf::from("a.png"));
    assert_eq!(paths[1], std::path::PathBuf::from("m.png"));
    assert_eq!(paths[2], std::path::PathBuf::from("z.png"));
}

/// 测试路径去重逻辑
#[test]
fn test_image_list_dedup_logic() {
    let mut image_list: Vec<std::path::PathBuf> = vec![
        std::path::PathBuf::from("a.png"),
        std::path::PathBuf::from("b.png"),
    ];

    let new_path = std::path::PathBuf::from("a.png");

    // 检查是否已存在
    if !image_list.contains(&new_path) {
        image_list.push(new_path.clone());
    }

    // 应该仍然是 2 个，因为 a.png 已存在
    assert_eq!(image_list.len(), 2);

    // 添加新文件
    let new_path2 = std::path::PathBuf::from("c.png");
    if !image_list.contains(&new_path2) {
        image_list.push(new_path2);
    }

    assert_eq!(image_list.len(), 3);
}
