use super::*;
use super::async_image_source::AsyncFsImageSource;
use crate::core::ports::ImageSource;
use std::path::Path;

#[test]
fn test_fs_image_source_is_supported() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("test.png")));
    assert!(source.is_supported(Path::new("test.PNG")));
    assert!(source.is_supported(Path::new("test.jpg")));
    assert!(!source.is_supported(Path::new("test.txt")));
    assert!(!source.is_supported(Path::new("test")));
}

#[test]
fn test_fs_image_source_supported_extensions() {
    assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(&"png"));
    assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(&"jpg"));
    assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(&"webp"));
}

#[test]
fn test_json_storage_default() {
    let storage = JsonStorage::new();
    assert!(storage.is_ok());
}

#[test]
fn test_rfd_file_dialog() {
    let _dialog = RfdFileDialog::new();
}

#[test]
#[allow(unused_variables)]
fn test_fs_image_source_new() {
    let source = FsImageSource::new();
}

#[test]
#[allow(unused_variables)]
fn test_fs_image_source_default() {
    let source: FsImageSource = Default::default();
}

#[test]
fn test_fs_image_source_is_supported_various() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("image.png")));
    assert!(source.is_supported(Path::new("image.jpg")));
    assert!(source.is_supported(Path::new("image.jpeg")));
    assert!(source.is_supported(Path::new("image.gif")));
    assert!(source.is_supported(Path::new("image.webp")));
    assert!(source.is_supported(Path::new("image.tiff")));
    assert!(source.is_supported(Path::new("image.tif")));
    assert!(source.is_supported(Path::new("image.bmp")));
    assert!(!source.is_supported(Path::new("image.txt")));
    assert!(!source.is_supported(Path::new("image.rs")));
    assert!(!source.is_supported(Path::new("image.pdf")));
    assert!(!source.is_supported(Path::new("image.zip")));
    assert!(!source.is_supported(Path::new("image")));
    assert!(!source.is_supported(Path::new("")));
}

#[test]
fn test_fs_image_source_is_supported_case_insensitive() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("image.PNG")));
    assert!(source.is_supported(Path::new("image.JPG")));
    assert!(source.is_supported(Path::new("image.JPEG")));
    assert!(source.is_supported(Path::new("image.GIF")));
    assert!(source.is_supported(Path::new("image.WEBP")));
    assert!(source.is_supported(Path::new("image.TIFF")));
    assert!(source.is_supported(Path::new("image.BMP")));
}

#[test]
fn test_fs_image_source_is_supported_with_paths() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("/path/to/image.png")));
    assert!(source.is_supported(Path::new("./relative/path/image.jpg")));
    assert!(source.is_supported(Path::new("C:\\Users\\image.gif")));
}

#[test]
fn test_fs_image_source_is_supported_dots_in_name() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("my.image.file.png")));
    assert!(source.is_supported(Path::new("archive.v2.jpg")));
    assert!(!source.is_supported(Path::new("archive.tar.gz")));
}

#[test]
fn test_supported_extensions_count() {
    assert_eq!(FsImageSource::SUPPORTED_EXTENSIONS.len(), 8);
}

#[test]
fn test_supported_extensions_all_present() {
    let expected = vec!["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"];
    for ext in &expected {
        assert!(FsImageSource::SUPPORTED_EXTENSIONS.contains(ext));
    }
}

#[test]
fn test_json_storage_new_success() {
    let storage = JsonStorage::new();
    assert!(storage.is_ok());
}

#[test]
fn test_async_fs_image_source_new() {
    let source = AsyncFsImageSource::new();
    drop(source);
}

#[test]
fn test_rfd_file_dialog_new() {
    let _dialog = RfdFileDialog::new();
}

#[test]
fn test_rfd_file_dialog_default() {
    let _dialog: RfdFileDialog = Default::default();
}

#[test]
fn test_fs_image_source_empty_extension() {
    let source = FsImageSource::new();
    assert!(!source.is_supported(Path::new("file.")));
}

#[test]
fn test_fs_image_source_unicode_paths() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("图片.png")));
    assert!(source.is_supported(Path::new("画像.jpg")));
    assert!(source.is_supported(Path::new("이미지.gif")));
}

#[test]
fn test_fs_image_source_numeric_names() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("001.png")));
    assert!(source.is_supported(Path::new("12345.jpg")));
}

#[test]
fn test_fs_image_source_special_chars() {
    let source = FsImageSource::new();
    assert!(source.is_supported(Path::new("my-image_file.png")));
    assert!(source.is_supported(Path::new("image+test.jpg")));
}
