use super::*;
use std::sync::mpsc::channel;

fn format_test_path(path: &Path) -> String {
    let path_str = path.display().to_string();
    if path_str.len() > 40 {
        format!("...{}", &path_str[path_str.len() - 37..])
    } else {
        path_str
    }
}

#[test]
fn test_info_panel_new() {
    let panel = InfoPanel::new();
    assert!(!panel.is_visible());
    assert_eq!(panel.width, 280.0);
}

#[test]
fn test_info_panel_with_visibility() {
    let panel_visible = InfoPanel::with_visibility(true);
    assert!(panel_visible.is_visible());

    let panel_hidden = InfoPanel::with_visibility(false);
    assert!(!panel_hidden.is_visible());
}

#[test]
fn test_info_panel_default() {
    let panel: InfoPanel = Default::default();
    assert!(!panel.is_visible());
}

#[test]
fn test_toggle_visibility() {
    let mut panel = InfoPanel::new();
    assert!(!panel.is_visible());

    panel.toggle();
    assert!(panel.is_visible());

    panel.toggle();
    assert!(!panel.is_visible());
}

#[test]
fn test_show_hide() {
    let mut panel = InfoPanel::new();

    panel.show();
    assert!(panel.is_visible());

    panel.hide();
    assert!(!panel.is_visible());
}

#[test]
fn test_multiple_toggles() {
    let mut panel = InfoPanel::new();

    for i in 1..=10 {
        panel.toggle();
        assert_eq!(panel.is_visible(), i % 2 == 1);
    }
}

#[test]
fn test_set_image_info() {
    let mut panel = InfoPanel::new();
    let path = Path::new("/test/image.png");
    let dimensions = (1920u32, 1080u32);
    let format = "PNG";

    panel.set_image_info(path, dimensions, format);

    assert!(panel.current_info.is_some());
    assert!(panel.loading_exif);
    assert!(panel.exif_receiver.is_some());

    let info = panel.current_info.clone().unwrap_or_default();
    assert_eq!(info.file_name, "image.png");
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
    assert_eq!(info.format, "PNG");
}

#[test]
fn test_set_image_info_clears_previous() {
    let mut panel = InfoPanel::new();

    panel.set_image_info(Path::new("/test/first.png"), (100, 100), "PNG");
    panel.set_image_info(Path::new("/test/second.jpg"), (200, 200), "JPEG");

    let info = panel.current_info.clone().unwrap_or_default();
    assert_eq!(info.file_name, "second.jpg");
    assert_eq!(info.width, 200);
}

#[test]
fn test_clear() {
    let mut panel = InfoPanel::new();
    panel.set_image_info(Path::new("/test/image.png"), (100, 100), "PNG");

    panel.clear();

    assert!(panel.current_info.is_none());
    assert!(!panel.loading_exif);
    assert!(panel.exif_receiver.is_none());
}

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
fn test_format_path_short() {
    let path = Path::new("/test/image.png");
    assert_eq!(format_test_path(path), "/test/image.png");
}

#[test]
fn test_format_path_long() {
    let path = Path::new("/very/long/path/to/the/image/file/that/needs/truncating.png");
    let result = format_test_path(path);
    assert!(result.starts_with("..."));
    assert!(result.len() <= 40);
}

#[test]
fn test_format_camera_info_both() {
    assert_eq!(
        format_camera_info(Some("Canon"), Some("Canon EOS 5D")),
        "Canon EOS 5D"
    );
    assert_eq!(
        format_camera_info(Some("Nikon"), Some("D850")),
        "Nikon D850"
    );
}

#[test]
fn test_format_camera_info_make_only() {
    assert_eq!(format_camera_info(Some("Sony"), None), "Sony");
}

#[test]
fn test_format_camera_info_model_only() {
    assert_eq!(
        format_camera_info(None, Some("iPhone 14 Pro")),
        "iPhone 14 Pro"
    );
}

#[test]
fn test_format_camera_info_none() {
    assert_eq!(format_camera_info(None, None), "Unknown");
}

#[test]
fn test_image_info_default() {
    let info = ImageInfo::default();
    assert_eq!(info.file_name, "");
    assert_eq!(info.width, 0);
    assert_eq!(info.height, 0);
    assert!(info.exif.is_none());
}

#[test]
fn test_exif_data_default() {
    let exif = ExifData::default();
    assert!(exif.date_time.is_none());
    assert!(exif.camera_model.is_none());
    assert!(exif.iso.is_none());
}

#[test]
fn test_exif_data_clone() {
    let exif = ExifData {
        date_time: Some("2026-01-01".to_string()),
        camera_model: Some("Test Camera".to_string()),
        iso: Some(100),
        ..Default::default()
    };
    let cloned = exif.clone();
    assert_eq!(exif.date_time, cloned.date_time);
    assert_eq!(exif.camera_model, cloned.camera_model);
    assert_eq!(exif.iso, cloned.iso);
}

#[test]
fn test_image_info_clone() {
    let info = ImageInfo {
        path: PathBuf::from("/test.png"),
        file_name: "test.png".to_string(),
        file_size: 1024,
        width: 100,
        height: 100,
        format: "PNG".to_string(),
        ..Default::default()
    };
    let cloned = info.clone();
    assert_eq!(info.file_name, cloned.file_name);
    assert_eq!(info.width, cloned.width);
}

#[test]
fn test_set_image_info_unknown_filename() {
    let mut panel = InfoPanel::new();
    panel.set_image_info(Path::new("/"), (100, 100), "PNG");

    let info = panel.current_info.clone().unwrap_or_default();
    assert_eq!(info.file_name, "Unknown");
}

#[test]
fn test_format_file_size_gigabytes() {
    assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
}

#[test]
fn test_format_file_size_terabytes() {
    assert_eq!(format_file_size(1024u64 * 1024 * 1024 * 1024), "1.00 TB");
}

#[test]
fn test_rapid_toggle() {
    let mut panel = InfoPanel::new();

    for _ in 0..100 {
        panel.toggle();
    }

    assert!(!panel.is_visible());
}

#[test]
fn test_exif_data_full() {
    let exif = ExifData {
        date_time: Some("2026:01:15 10:30:00".to_string()),
        camera_model: Some("Canon EOS R5".to_string()),
        camera_make: Some("Canon".to_string()),
        lens_model: Some("RF 24-70mm F2.8".to_string()),
        iso: Some(100),
        aperture: Some("f/2.8".to_string()),
        shutter_speed: Some("1/250".to_string()),
        focal_length: Some("50 mm".to_string()),
        gps_latitude: Some("39° 54' 15\" N".to_string()),
        gps_longitude: Some("116° 24' 25\" E".to_string()),
    };

    assert!(exif.date_time.is_some());
    assert!(exif.camera_model.is_some());
    assert_eq!(exif.iso, Some(100));
}

#[test]
fn test_image_info_with_exif() {
    let exif = ExifData {
        camera_model: Some("Sony A7IV".to_string()),
        iso: Some(400),
        ..Default::default()
    };

    let info = ImageInfo {
        path: PathBuf::from("/test/photo.jpg"),
        file_name: "photo.jpg".to_string(),
        file_size: 2048,
        modified_time: Some("2026-01-01 12:00:00".to_string()),
        format: "JPEG".to_string(),
        width: 4000,
        height: 3000,
        bit_depth: Some(8),
        color_space: Some("sRGB".to_string()),
        exif: Some(exif),
    };

    assert_eq!(info.file_size, 2048);
    assert_eq!(info.width, 4000);
    assert_eq!(info.height, 3000);
    assert!(info.exif.is_some());
    assert_eq!(info.exif.as_ref().map(|e| e.iso), Some(Some(400)));
}

#[test]
fn test_format_path_exact_40() {
    let path = Path::new("/123456789/123456789/123456789/123456789.png");
    let result = format_test_path(path);
    assert_eq!(result.len(), 40);
}

#[test]
fn test_format_path_unicode() {
    let path = Path::new("/图片/照片/test.png");
    let result = format_test_path(path);
    assert!(result.contains("test.png"));
}

#[test]
fn test_format_camera_info_whitespace() {
    assert_eq!(
        format_camera_info(Some("  Canon  "), Some("  EOS R5  ")),
        "Canon EOS R5"
    );
}

#[test]
fn test_format_camera_info_model_starts_with_make_uppercase() {
    assert_eq!(
        format_camera_info(Some("SONY"), Some("SONY ILCE-7M4")),
        "SONY ILCE-7M4"
    );
}

#[test]
fn test_format_file_size_boundary() {
    assert_eq!(format_file_size(1024 * 1024 - 1), "1024.00 KB");
    assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
}

#[test]
fn test_format_file_size_large() {
    assert_eq!(format_file_size(1024u64 * 1024 * 1024 * 1024), "1.00 TB");
}

#[test]
fn test_exif_receiver_disconnected() {
    let mut panel = InfoPanel::new();
    let (sender, receiver) = channel::<ExifData>();
    panel.exif_receiver = Some(receiver);
    drop(sender);

    panel.check_exif_receiver();

    assert!(!panel.loading_exif);
    assert!(panel.exif_receiver.is_none());
}

#[test]
fn test_exif_receiver_empty() {
    let mut panel = InfoPanel::new();
    let (_sender, receiver) = channel::<ExifData>();
    panel.exif_receiver = Some(receiver);
    panel.loading_exif = true;

    panel.check_exif_receiver();

    assert!(panel.loading_exif);
    assert!(panel.exif_receiver.is_some());
}

#[test]
fn test_exif_receiver_success() {
    let mut panel = InfoPanel::new();
    let (sender, receiver) = channel::<ExifData>();
    panel.exif_receiver = Some(receiver);
    panel.loading_exif = true;
    panel.current_info = Some(ImageInfo::default());

    let exif = ExifData {
        camera_model: Some("Test".to_string()),
        ..Default::default()
    };
    let _ = sender.send(exif);

    panel.check_exif_receiver();

    assert!(!panel.loading_exif);
    assert!(panel.exif_receiver.is_none());
    assert!(panel.current_info.as_ref().map(|i| i.exif.is_some()).unwrap_or(false));
}

#[test]
fn test_info_panel_width_persistence() {
    let mut panel = InfoPanel::new();
    assert_eq!(panel.width, 280.0);
    panel.width = 300.0;
    assert_eq!(panel.width, 300.0);
}

#[test]
fn test_multiple_info_panel_instances() {
    let panel1 = InfoPanel::with_visibility(true);
    let panel2 = InfoPanel::with_visibility(false);

    assert!(panel1.is_visible());
    assert!(!panel2.is_visible());
}

#[test]
fn test_image_info_with_all_fields_none() {
    let info = ImageInfo {
        path: PathBuf::from("/test.png"),
        file_name: "test.png".to_string(),
        file_size: 0,
        modified_time: None,
        format: "PNG".to_string(),
        width: 100,
        height: 100,
        bit_depth: None,
        color_space: None,
        exif: None,
    };

    assert!(info.modified_time.is_none());
    assert!(info.bit_depth.is_none());
    assert!(info.color_space.is_none());
    assert!(info.exif.is_none());
}
