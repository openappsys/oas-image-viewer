//! 工具模块

pub mod threading;

/// 格式化文件大小用于显示
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return "0 B".to_string();
    }

    let exp = (size as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let size = size as f64 / 1024f64.powi(exp as i32);

    if exp == 0 {
        format!("{:.0} {}", size, UNITS[exp])
    } else {
        format!("{:.2} {}", size, UNITS[exp])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_format_file_size_gigabytes() {
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    // 从旧代码迁移的额外测试

    #[test]
    fn test_format_file_size_terabytes() {
        assert_eq!(format_file_size(1024u64.pow(4)), "1.00 TB");
        assert_eq!(format_file_size(1024u64.pow(4) * 5), "5.00 TB");
    }

    #[test]
    fn test_format_file_size_large_values() {
        // 测试非常大的值不会溢出
        let size = u64::MAX;
        let formatted = format_file_size(size);
        assert!(formatted.contains("TB") || formatted.contains("GB"));
    }

    #[test]
    fn test_format_file_size_exact_boundaries() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
