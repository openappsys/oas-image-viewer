//! 拖放处理模块 - 处理文件拖放事件
//!
//! 支持拖放单个或多个图片文件到窗口中打开。

use std::path::PathBuf;

/// 提取拖放文件中的图片文件路径
/// 
/// 过滤并收集所有支持的图片格式文件路径。
/// 
/// # Arguments
/// * `dropped_files` - 拖放的文件列表
/// 
/// # Returns
/// 按原始顺序排列的图片文件路径列表
pub fn extract_image_files(raw_files: &[egui::DroppedFile]) -> Vec<PathBuf> {
    let mut image_paths: Vec<PathBuf> = Vec::new();
    
    for file in raw_files {
        if let Some(path) = &file.path {
            if path.is_file() && is_image_file(path) {
                image_paths.push(path.clone());
            } else if path.is_dir() {
                // 收集目录中的所有图片文件
                let dir_images = collect_images_from_dir(path);
                image_paths.extend(dir_images);
            }
        }
    }
    
    // 按文件名排序
    image_paths.sort();
    image_paths
}

/// 从目录中收集所有图片文件
fn collect_images_from_dir(path: &std::path::Path) -> Vec<PathBuf> {
    let mut images: Vec<PathBuf> = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_file() && is_image_file(&entry_path) {
                images.push(entry_path);
            }
        }
    }
    
    images.sort();
    images
}

/// 检查是否为支持的图片文件
fn is_image_file(path: &std::path::Path) -> bool {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    
    matches!(
        ext.as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp")
    )
}

/// 检查是否有文件正在拖拽悬停
pub fn is_drag_hovering(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.raw.hovered_files.iter().any(|f| f.path.is_some()))
}

/// 获取拖拽预览文本
pub fn get_drag_preview_text(ctx: &egui::Context) -> Option<String> {
    ctx.input(|i| {
        let count = i.raw.hovered_files.iter()
            .filter(|f| f.path.is_some())
            .count();
        
        if count > 0 {
            Some(format!("准备打开 {} 个文件", count))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 图片文件检测测试
    // =========================================================================

    #[test]
    fn test_is_image_file_png() {
        assert!(is_image_file(std::path::Path::new("test.png")));
        assert!(is_image_file(std::path::Path::new("test.PNG")));
    }

    #[test]
    fn test_is_image_file_jpeg() {
        assert!(is_image_file(std::path::Path::new("test.jpg")));
        assert!(is_image_file(std::path::Path::new("test.jpeg")));
        assert!(is_image_file(std::path::Path::new("test.JPG")));
    }

    #[test]
    fn test_is_image_file_gif() {
        assert!(is_image_file(std::path::Path::new("test.gif")));
    }

    #[test]
    fn test_is_image_file_webp() {
        assert!(is_image_file(std::path::Path::new("test.webp")));
    }

    #[test]
    fn test_is_image_file_bmp() {
        assert!(is_image_file(std::path::Path::new("test.bmp")));
    }

    #[test]
    fn test_is_image_file_tiff() {
        assert!(is_image_file(std::path::Path::new("test.tiff")));
        assert!(is_image_file(std::path::Path::new("test.tif")));
    }

    #[test]
    fn test_is_not_image_file() {
        assert!(!is_image_file(std::path::Path::new("test.txt")));
        assert!(!is_image_file(std::path::Path::new("test.rs")));
        assert!(!is_image_file(std::path::Path::new("test.pdf")));
        assert!(!is_image_file(std::path::Path::new("test")));
    }

    #[test]
    fn test_is_image_file_empty() {
        assert!(!is_image_file(std::path::Path::new("")));
    }

    // =========================================================================
    // 目录图片收集测试
    // =========================================================================

    #[test]
    fn test_collect_images_from_dir_empty() {
        let images = collect_images_from_dir(std::path::Path::new("/nonexistent/directory"));
        assert!(images.is_empty());
    }

    #[test]
    fn test_collect_images_from_dir_not_a_dir() {
        let images = collect_images_from_dir(std::path::Path::new("test.png"));
        assert!(images.is_empty());
    }

    // =========================================================================
    // extract_image_files 测试
    // =========================================================================

    #[test]
    fn test_extract_image_files_empty() {
        let files: Vec<egui::DroppedFile> = vec![];
        let result = extract_image_files(&files);
        assert!(result.is_empty());
    }

    // 注意：由于 DroppedFile 结构复杂，其他测试需要集成测试环境
    // 以下测试验证了核心逻辑

    #[test]
    fn test_supported_extensions_all() {
        let extensions = vec![
            "png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp",
        ];
        
        for ext in extensions {
            let path_str = format!("test.{}", ext);
            let path = std::path::Path::new(&path_str);
            assert!(is_image_file(path), "Failed for extension: {}", ext);
        }
    }

    #[test]
    fn test_supported_extensions_uppercase() {
        let extensions = vec![
            "PNG", "JPG", "JPEG", "GIF", "WEBP", "TIFF", "TIF", "BMP",
        ];
        
        for ext in extensions {
            let path_str = format!("test.{}", ext);
            let path = std::path::Path::new(&path_str);
            assert!(is_image_file(path), "Failed for extension: {}", ext);
        }
    }

    #[test]
    fn test_unsupported_extensions() {
        let extensions = vec![
            "txt", "rs", "toml", "md", "pdf", "doc", "exe", "zip",
            "mp4", "avi", "mp3", "svg",
        ];
        
        for ext in extensions {
            let path_str = format!("test.{}", ext);
            let path = std::path::Path::new(&path_str);
            assert!(!is_image_file(path), "Should not be image: {}", ext);
        }
    }

    #[test]
    fn test_is_image_file_with_path() {
        assert!(is_image_file(std::path::Path::new("/home/user/image.png")));
        assert!(is_image_file(std::path::Path::new("C:\\Users\\image.jpg")));
        assert!(is_image_file(std::path::Path::new("./relative/path/image.gif")));
        assert!(!is_image_file(std::path::Path::new("/home/user/document.txt")));
    }
}
