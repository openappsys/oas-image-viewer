//! 国际化 (i18n) 模块 - 提供多语言支持
//!
//! 采用 JSON 语言包文件，运行时加载

use crate::core::domain::Language;
use std::collections::HashMap;
use std::sync::OnceLock;

/// 中文语言包（静态存储）
static CHINESE_PACK: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// 英文语言包（静态存储）
static ENGLISH_PACK: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// 从 JSON 内容加载为静态键值对
fn load_static_pack(json_content: &str) -> HashMap<&'static str, &'static str> {
    let value: serde_json::Value = match serde_json::from_str(json_content) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "解析语言包 JSON 失败");
            return HashMap::new();
        }
    };
    let mut translations = HashMap::new();

    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            if let serde_json::Value::String(s) = val {
                let static_key: &'static str = Box::leak(key.into_boxed_str());
                let static_value: &'static str = Box::leak(s.into_boxed_str());
                translations.insert(static_key, static_value);
            }
        }
    }

    translations
}

/// 获取可能的语言包路径列表
fn get_locale_paths(lang_code: &str) -> Vec<String> {
    let mut paths = vec![];

    // 1. 当前目录下的 locales
    paths.push(format!("./locales/{}.json", lang_code));

    // 2. 可执行文件所在目录的 locales
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            paths.push(format!("{}/locales/{}.json", exe_dir.display(), lang_code));
        }
    }

    // 3. 相对于项目根目录的路径（开发环境）
    paths.push(format!("locales/{}.json", lang_code));

    // 4. Unix 系统标准路径
    #[cfg(unix)]
    {
        paths.push(format!(
            "/usr/share/oas-image-viewer/locales/{}.json",
            lang_code
        ));
        paths.push(format!(
            "/usr/local/share/oas-image-viewer/locales/{}.json",
            lang_code
        ));
    }

    paths
}

/// 尝试从文件系统加载语言包
fn try_load_from_filesystem(lang_code: &str) -> Option<HashMap<&'static str, &'static str>> {
    let paths = get_locale_paths(lang_code);

    for path in &paths {
        if std::path::Path::new(path).exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    tracing::info!("从文件加载语言包: {}", path);
                    return Some(load_static_pack(&content));
                }
                Err(e) => {
                    tracing::warn!("读取语言包文件失败 {}: {}", path, e);
                }
            }
        }
    }

    None
}

/// 从嵌入的 JSON 加载语言包
fn load_embedded_pack(lang_code: &str) -> HashMap<&'static str, &'static str> {
    let json = match lang_code {
        "zh-CN" => include_str!("../../../locales/zh-CN.json"),
        "en-US" => include_str!("../../../locales/en-US.json"),
        _ => {
            tracing::error!("未知的语言代码: {}", lang_code);
            return HashMap::new();
        }
    };
    load_static_pack(json)
}

/// 初始化语言包
///
/// 应在应用启动时调用一次
/// 优先从文件系统加载，失败时回退到嵌入的 JSON
pub fn initialize() {
    // 加载中文语言包
    let chinese_pack = try_load_from_filesystem("zh-CN").unwrap_or_else(|| {
        tracing::info!("使用嵌入的中文语言包");
        load_embedded_pack("zh-CN")
    });
    let _ = CHINESE_PACK.set(chinese_pack);

    // 加载英文语言包
    let english_pack = try_load_from_filesystem("en-US").unwrap_or_else(|| {
        tracing::info!("使用嵌入的英文语言包");
        load_embedded_pack("en-US")
    });
    let _ = ENGLISH_PACK.set(english_pack);
}

/// 获取翻译文本
///
/// # 参数
/// - `key`: 文本键名
/// - `lang`: 目标语言
///
/// # 返回值
/// 返回对应的翻译文本，如果没有找到则返回键名本身
pub fn get_text(key: &str, lang: Language) -> &str {
    let pack = match lang {
        Language::Chinese => CHINESE_PACK.get(),
        Language::English => ENGLISH_PACK.get(),
    };

    match pack {
        Some(map) => map.get(key).copied().unwrap_or(key),
        None => key, // 语言包未初始化
    }
}

/// 获取当前语言的文本（便捷函数）
///
/// 根据当前中文字体支持状态自动选择语言
pub fn t(key: &str) -> &str {
    let lang = if crate::is_chinese_supported() {
        Language::Chinese
    } else {
        Language::English
    };
    get_text(key, lang)
}

/// 为特定语言格式化缩略图大小提示
pub fn format_thumbnail_hint(size: u32, lang: Language) -> String {
    let template = get_text("thumbnail_hint", lang);
    // 如果找不到模板，使用默认格式
    if template == "thumbnail_hint" {
        match lang {
            Language::Chinese => format!("缩略图: {}px", size),
            Language::English => format!("Thumbnail: {}px", size),
        }
    } else {
        template.replace("{}", &size.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_for_test() {
        // 使用 get_or_init 确保线程安全，避免竞态条件
        CHINESE_PACK.get_or_init(|| {
            try_load_from_filesystem("zh-CN").unwrap_or_else(|| load_embedded_pack("zh-CN"))
        });
        ENGLISH_PACK.get_or_init(|| {
            try_load_from_filesystem("en-US").unwrap_or_else(|| load_embedded_pack("en-US"))
        });
    }

    #[test]
    fn test_get_text_menu() {
        init_for_test();
        assert_eq!(get_text("menu_file", Language::Chinese), "文件");
        assert_eq!(get_text("menu_file", Language::English), "File");
        assert_eq!(get_text("menu_view", Language::Chinese), "视图");
        assert_eq!(get_text("menu_view", Language::English), "View");
    }

    #[test]
    fn test_get_text_buttons() {
        init_for_test();
        assert_eq!(get_text("open", Language::Chinese), "打开...");
        assert_eq!(get_text("open", Language::English), "Open...");
        assert_eq!(get_text("close", Language::Chinese), "关闭");
        assert_eq!(get_text("close", Language::English), "Close");
    }

    #[test]
    fn test_get_text_unknown_key() {
        init_for_test();
        let key = "unknown_key_xyz";
        assert_eq!(get_text(key, Language::Chinese), key);
        assert_eq!(get_text(key, Language::English), key);
    }

    #[test]
    fn test_format_thumbnail_hint() {
        init_for_test();
        assert_eq!(
            format_thumbnail_hint(100, Language::Chinese),
            "缩略图: 100px"
        );
        assert_eq!(
            format_thumbnail_hint(100, Language::English),
            "Thumbnail: 100px"
        );
    }

    #[test]
    fn test_all_menu_keys_exist() {
        init_for_test();
        // 确保主要菜单键都有翻译
        let keys = [
            "menu_file",
            "menu_view",
            "menu_image",
            "menu_help",
            "open",
            "exit",
            "gallery",
            "viewer",
            "fullscreen",
            "about",
            "close",
            "drag_hint",
            "no_image",
            "image_info",
            "file_name",
            "dimensions",
            "file_size",
            "shortcuts_title",
            "navigation",
            "zoom",
            "view",
            "other",
        ];

        for key in &keys {
            let chinese = get_text(key, Language::Chinese);
            let english = get_text(key, Language::English);
            assert_ne!(chinese, *key, "Chinese translation missing for: {}", key);
            assert_ne!(english, *key, "English translation missing for: {}", key);
        }
    }
}
