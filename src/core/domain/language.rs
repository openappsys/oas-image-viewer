//! 语言类型定义

use serde::{Deserialize, Serialize};

/// 应用支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Chinese,
    English,
}

impl Language {
    /// 获取语言显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Chinese => "中文",
            Language::English => "English",
        }
    }

    /// 检测系统语言
    pub fn detect_system() -> Self {
        match sys_locale::get_locale() {
            Some(lang) if lang.starts_with("zh") => Language::Chinese,
            _ => Language::English,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_display_name() {
        assert_eq!(Language::Chinese.display_name(), "中文");
        assert_eq!(Language::English.display_name(), "English");
    }

    #[test]
    fn test_language_default() {
        let lang: Language = Default::default();
        assert_eq!(lang, Language::Chinese);
    }

    #[test]
    fn test_language_clone() {
        let lang = Language::Chinese;
        let cloned = lang;
        assert_eq!(lang, cloned);
    }

    #[test]
    fn test_language_equality() {
        assert_eq!(Language::Chinese, Language::Chinese);
        assert_eq!(Language::English, Language::English);
        assert_ne!(Language::Chinese, Language::English);
    }

    #[test]
    fn test_language_serialize() {
        let lang = Language::Chinese;
        let json = serde_json::to_string(&lang).unwrap();
        assert!(!json.is_empty());

        let deserialized: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(lang, deserialized);
    }
}
