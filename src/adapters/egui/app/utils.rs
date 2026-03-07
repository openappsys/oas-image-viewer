//! 工具函数

use egui::Context;

/// 获取拖拽预览文本
pub fn get_drag_preview_text(ctx: &Context) -> Option<String> {
    ctx.input(|i| {
        let count = i.raw.hovered_files.len();
        if count > 1 {
            Some(format!("{} 个文件", count))
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
        let _ = writeln!(file, "{}", msg);
    }
}
