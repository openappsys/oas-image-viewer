use super::ExifData;
use std::path::Path;
use tracing::{debug, warn};

pub(super) fn read_exif_data(path: &Path) -> ExifData {
    use exif::{Reader, Tag};

    let mut exif_data = ExifData::default();

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            debug!("无法打开文件读取EXIF: {}", e);
            return exif_data;
        }
    };

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = Reader::new();

    let exif = match exifreader.read_from_container(&mut bufreader) {
        Ok(e) => e,
        Err(e) => {
            debug!("读取EXIF失败: {}", e);
            return exif_data;
        }
    };

    for field in exif.fields() {
        match field.tag {
            Tag::DateTime | Tag::DateTimeOriginal => {
                if exif_data.date_time.is_none() {
                    exif_data.date_time = Some(field.display_value().with_unit(&exif).to_string());
                }
            }
            Tag::Make => {
                exif_data.camera_make = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::Model => {
                exif_data.camera_model = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::LensModel => {
                exif_data.lens_model = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::ISOSpeed => {
                if let Some(val) = field.value.get_uint(0) {
                    exif_data.iso = Some(val);
                }
            }
            Tag::FNumber => {
                exif_data.aperture = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::ExposureTime => {
                exif_data.shutter_speed = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::FocalLength => {
                exif_data.focal_length = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::GPSLatitude => {
                exif_data.gps_latitude = Some(field.display_value().with_unit(&exif).to_string());
            }
            Tag::GPSLongitude => {
                exif_data.gps_longitude = Some(field.display_value().with_unit(&exif).to_string());
            }
            _ => {}
        }
    }

    exif_data
}

pub(super) fn get_file_metadata(path: &Path) -> (u64, Option<String>) {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            warn!("无法获取文件元数据: {}", e);
            return (0, None);
        }
    };

    let size = metadata.len();
    let modified = metadata.modified().ok().and_then(|t| {
        use std::time::SystemTime;
        let duration = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
        let secs = duration.as_secs();
        let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)?;
        Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
    });

    (size, modified)
}
