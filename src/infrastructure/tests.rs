use super::*;
use super::async_image_source::AsyncFsImageSource;
use super::image_decoder::ImageDecoderBackend;
use crate::core::ports::ImageSource;
use crate::core::Result;
use image::DynamicImage;
use std::path::Path;
use std::sync::Arc;

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

struct MockDecoderBackend;

impl ImageDecoderBackend for MockDecoderBackend {
    fn decode_path(&self, _path: &Path) -> Result<DynamicImage> {
        Ok(DynamicImage::new_rgba8(4, 3))
    }

    fn decode_bytes(&self, _data: &[u8]) -> Result<DynamicImage> {
        Ok(DynamicImage::new_rgba8(1, 1))
    }

    fn dimensions(&self, _path: &Path) -> Result<(u32, u32)> {
        Ok((7, 9))
    }
}

#[test]
fn test_fs_image_source_with_custom_decoder_load_image_data() {
    let source = FsImageSource::with_decoder(Arc::new(MockDecoderBackend));
    let (width, height, data) = source
        .load_image_data(Path::new("ignored.png"))
        .expect("should decode via mock backend");
    assert_eq!(width, 4);
    assert_eq!(height, 3);
    assert_eq!(data.len(), 4 * 3 * 4);
}

#[test]
fn test_fs_image_source_with_custom_decoder_load_metadata_dimensions() {
    let source = FsImageSource::with_decoder(Arc::new(MockDecoderBackend));
    let file_path = std::env::temp_dir().join(format!(
        "oas_image_viewer_decoder_test_{}.png",
        std::process::id()
    ));
    std::fs::write(&file_path, [0u8; 8]).expect("should create temp file");
    let metadata = source
        .load_metadata(&file_path)
        .expect("should read metadata via mock backend");
    assert_eq!(metadata.width, 7);
    assert_eq!(metadata.height, 9);
    let _ = std::fs::remove_file(&file_path);
}
