//! 工具函数

use crate::adapters::egui::i18n::get_text;
use crate::core::domain::Language;
use egui::Context;

/// 获取拖拽预览文本
pub fn get_drag_preview_text(ctx: &Context, language: Language) -> Option<String> {
    ctx.input(|i| {
        let count = i.raw.hovered_files.len();
        if count > 1 {
            Some(get_text("files_count", language).replace("{}", &count.to_string()))
        } else if count == 1 {
            i.raw.hovered_files.first().and_then(|f| {
                f.path
                    .as_ref()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            })
        } else {
            None
        }
    })
}

/// 日志函数
#[allow(dead_code)]
pub fn log_panel(msg: &str) {
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
    {
        if let Err(e) = writeln!(file, "{}", msg) {
            tracing::warn!(error = %e, "写入调试日志失败");
        }
    }
}
