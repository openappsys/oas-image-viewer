//! 信息面板 EXIF 元数据读取逻辑

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
        let raw_value = field.display_value().with_unit(&exif).to_string();
        let value = sanitize_exif_value(&raw_value);
        let tag_name = field.tag.to_string();
        match field.tag {
            Tag::DateTime | Tag::DateTimeOriginal => {
                if exif_data.date_time.is_none() {
                    exif_data.date_time = value.clone();
                }
            }
            Tag::Make => {
                exif_data.camera_make = value.clone();
            }
            Tag::Model => {
                exif_data.camera_model = value.clone();
            }
            Tag::LensModel => {
                exif_data.lens_model = value.clone();
            }
            Tag::ISOSpeed => {
                if let Some(val) = field.value.get_uint(0) {
                    exif_data.iso = Some(val);
                }
            }
            Tag::FNumber => {
                exif_data.aperture = value.clone();
            }
            Tag::ExposureTime => {
                exif_data.shutter_speed = value.clone();
            }
            Tag::FocalLength => {
                exif_data.focal_length = value.clone();
            }
            Tag::GPSLatitude => {
                exif_data.gps_latitude = value.clone();
            }
            Tag::GPSLongitude => {
                exif_data.gps_longitude = value.clone();
            }
            _ => {}
        }

        if exif_data.iso.is_none()
            && matches!(tag_name.as_str(), "PhotographicSensitivity" | "ISOSpeedRatings")
        {
            if let Some(val) = field.value.get_uint(0) {
                exif_data.iso = Some(val);
            } else if let Some(v) = &value {
                if let Ok(parsed) = v.trim().parse::<u32>() {
                    exif_data.iso = Some(parsed);
                }
            } else if let Ok(parsed) = raw_value.trim().parse::<u32>() {
                exif_data.iso = Some(parsed);
            }
        }

        match tag_name.as_str() {
            "LensMake" => assign_if_none(&mut exif_data.lens_make, value.clone()),
            "Software" => assign_if_none(&mut exif_data.software, value.clone()),
            "ExposureBiasValue" => assign_if_none(&mut exif_data.exposure_bias, value.clone()),
            "WhiteBalance" => assign_if_none(&mut exif_data.white_balance, value.clone()),
            "Flash" => assign_if_none(&mut exif_data.flash, value.clone()),
            "MeteringMode" => assign_if_none(&mut exif_data.metering_mode, value.clone()),
            "ExposureProgram" => assign_if_none(&mut exif_data.exposure_program, value.clone()),
            "ExposureMode" => assign_if_none(&mut exif_data.exposure_mode, value.clone()),
            "GPSAltitude" => assign_if_none(&mut exif_data.gps_altitude, value.clone()),
            "GPSTimeStamp" | "GPSDateStamp" => {
                if let Some(v) = &value {
                    if let Some(existing) = &exif_data.gps_timestamp {
                        if !existing.contains(v) {
                            exif_data.gps_timestamp = Some(format!("{} {}", existing, v));
                        }
                    } else {
                        exif_data.gps_timestamp = Some(v.clone());
                    }
                }
            }
            _ => {}
        }

        if let Some(v) = &value {
            if !is_primary_exif_tag(&tag_name)
                && !exif_data
                    .extra_fields
                    .iter()
                    .any(|(key, val)| key == &tag_name && val == v)
            {
                exif_data.extra_fields.push((tag_name, v.clone()));
            }
        }
    }

    exif_data
}

fn assign_if_none(target: &mut Option<String>, value: Option<String>) {
    if target.is_none() {
        *target = value;
    }
}

fn is_primary_exif_tag(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "DateTime"
            | "DateTimeOriginal"
            | "DateTimeDigitized"
            | "Make"
            | "Model"
            | "LensModel"
            | "LensMake"
            | "Software"
            | "ISOSpeed"
            | "PhotographicSensitivity"
            | "ISOSpeedRatings"
            | "FNumber"
            | "ExposureTime"
            | "FocalLength"
            | "ExposureBiasValue"
            | "WhiteBalance"
            | "Flash"
            | "MeteringMode"
            | "ExposureProgram"
            | "ExposureMode"
            | "GPSLatitude"
            | "GPSLongitude"
            | "GPSAltitude"
            | "GPSTimeStamp"
            | "GPSDateStamp"
    )
}

fn sanitize_exif_value(raw: &str) -> Option<String> {
    let cleaned_segments: Vec<String> = raw
        .split(',')
        .map(normalize_exif_segment)
        .filter(|v| !v.is_empty())
        .collect();

    if cleaned_segments.is_empty() {
        return None;
    }

    let mut deduped: Vec<String> = Vec::with_capacity(cleaned_segments.len());
    for segment in cleaned_segments {
        if deduped.last() != Some(&segment) {
            deduped.push(segment);
        }
    }
    Some(deduped.join(", "))
}

fn normalize_exif_segment(segment: &str) -> String {
    let trimmed = segment.trim();
    let unquoted = trimmed.trim_matches(|c| c == '"' || c == '\'');
    let collapsed = unquoted.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().to_string()
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

#[cfg(test)]
mod tests {
    use super::sanitize_exif_value;

    #[test]
    fn sanitize_exif_value_removes_padding_and_quotes() {
        let raw = "\"Lenovo                         \"";
        assert_eq!(sanitize_exif_value(raw), Some("Lenovo".to_string()));
    }

    #[test]
    fn sanitize_exif_value_compacts_sparse_lists() {
        let raw = "\"MediaTek\", \"\", \"\", \"\"";
        assert_eq!(sanitize_exif_value(raw), Some("MediaTek".to_string()));
    }

    #[test]
    fn sanitize_exif_value_drops_whitespace_only() {
        let raw = "\"                               \"";
        assert_eq!(sanitize_exif_value(raw), None);
    }
}
