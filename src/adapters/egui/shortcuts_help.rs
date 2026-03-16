//! 快捷键帮助面板模块
//!
//! 显示应用程序支持的所有键盘快捷键列表。
//! 按 ? 键打开/关闭帮助面板。

use crate::adapters::egui::i18n::get_text;
use crate::core::domain::Language;
use egui::{Color32, Context, FontId, RichText, Vec2, Window};

/// 快捷键帮助面板状态
#[derive(Debug, Clone, Default)]
pub struct ShortcutsHelpPanel {
    pub visible: bool,
}

impl ShortcutsHelpPanel {
    /// 创建新的帮助面板（默认隐藏）
    pub fn new() -> Self {
        Self::default()
    }

    /// 切换面板可见性
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// 显示面板
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// 隐藏面板
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// 检查面板是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 渲染帮助面板
    pub fn ui(&mut self, ctx: &Context, language: Language) {
        if !self.visible {
            return;
        }

        // 获取视口大小，计算面板最大尺寸（留出边距）
        let screen_rect = ctx.viewport_rect();
        let max_width = screen_rect.width().clamp(280.0, 420.0);
        let max_height = screen_rect.height().clamp(200.0, 520.0);

        // 创建半透明背景遮罩
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            "shortcuts_help_bg".into(),
        ));
        painter.rect_filled(
            screen_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 120),
        );

        let title = format!("⌨️ {}", get_text("shortcuts_title", language));
        let heading = get_text("shortcuts_keyboard", language);
        let close_btn = format!("{} (Esc)", get_text("close", language));

        // 帮助面板窗口 - 响应式设计
        Window::new(title)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .max_size([max_width, max_height])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(Color32::from_rgb(40, 40, 45))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 90)))
                    .corner_radius(egui::CornerRadius::same(8)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(heading);
                    ui.add_space(12.0);
                });

                // 使用 ScrollArea 确保内容可滚动
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.scope(|ui| {
                            // 设置面板内文字样式
                            ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 10.0);

                            // 文件操作
                            render_shortcut_category(
                                ui,
                                &format!("📁 {}", get_text("file_ops", language)),
                                &[
                                    ("Ctrl + O", get_text("shortcut_open", language)),
                                    ("Esc", get_text("shortcut_exit_fullscreen", language)),
                                ],
                            );

                            ui.add_space(6.0);

                            // 导航操作
                            render_shortcut_category(
                                ui,
                                &format!("🧭 {}", get_text("navigation", language)),
                                &[
                                    ("← / →", get_text("shortcut_prev_next", language)),
                                    ("G", get_text("shortcut_toggle_view", language)),
                                ],
                            );

                            ui.add_space(6.0);

                            // 视图操作
                            render_shortcut_category(
                                ui,
                                &format!("👁 {}", get_text("view", language)),
                                &[
                                    ("F11", get_text("shortcut_fullscreen", language)),
                                    ("Ctrl + +", get_text("shortcut_zoom_in", language)),
                                    ("Ctrl + -", get_text("shortcut_zoom_out", language)),
                                    ("Ctrl + 0", get_text("shortcut_fit_window", language)),
                                    ("Ctrl + 1", get_text("shortcut_original", language)),
                                    ("F", get_text("shortcut_info_panel", language)),
                                    (
                                        get_text("shortcut_dbl_click", language),
                                        get_text("shortcut_fullscreen", language),
                                    ),
                                ],
                            );

                            ui.add_space(6.0);

                            // 其他操作
                            render_shortcut_category(
                                ui,
                                &format!("🔧 {}", get_text("other", language)),
                                &[("?", get_text("shortcut_help", language))],
                            );
                        });

                        ui.add_space(12.0);

                        // 关闭按钮
                        ui.vertical_centered(|ui| {
                            if ui.button(close_btn).clicked() {
                                self.hide();
                            }
                        });
                    });
            });
    }

    /// 处理键盘输入（? 键和 ESC 键）
    /// 返回 true 表示按键已被处理
    pub fn handle_input(&mut self, ctx: &Context) -> bool {
        // 检查 ? 键 - 通过 Text 事件检测（兼容中英文）
        let question_pressed = ctx.input(|i| {
            let question_typed = i.events.iter().any(|e| {
                if let egui::Event::Text(text) = e {
                    text == "?" || text == "？"
                } else {
                    false
                }
            });
            question_typed
        });

        if question_pressed {
            self.toggle();
            return true;
        }

        // 检查 ESC 键
        let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        if esc_pressed && self.visible {
            self.hide();
            return true;
        }

        false
    }
}

/// 渲染快捷键分类
fn render_shortcut_category(ui: &mut egui::Ui, title: &str, shortcuts: &[(&str, &str)]) {
    ui.group(|ui| {
        ui.set_min_width(360.0);

        // 分类标题
        ui.label(
            RichText::new(title)
                .font(FontId::proportional(16.0))
                .color(Color32::from_rgb(100, 180, 255)),
        );

        ui.add_space(8.0);

        // 快捷键列表
        for (shortcut, description) in shortcuts {
            ui.horizontal(|ui| {
                ui.add_space(16.0);

                // 快捷键显示
                ui.label(
                    RichText::new(*shortcut)
                        .font(FontId::monospace(13.0))
                        .color(Color32::from_rgb(255, 200, 100)),
                );

                ui.add_space(16.0);

                // 分隔符
                ui.label(RichText::new("—").color(Color32::GRAY));

                ui.add_space(8.0);

                // 功能描述
                ui.label(
                    RichText::new(*description)
                        .font(FontId::proportional(14.0))
                        .color(Color32::LIGHT_GRAY),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |_ui| {});
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 基础初始化测试
    // =========================================================================

    #[test]
    fn test_shortcuts_help_panel_new() {
        let panel = ShortcutsHelpPanel::new();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_shortcuts_help_panel_default() {
        let panel: ShortcutsHelpPanel = Default::default();
        assert!(!panel.is_visible());
    }

    // =========================================================================
    // 状态管理测试
    // =========================================================================

    #[test]
    fn test_toggle_visibility() {
        let mut panel = ShortcutsHelpPanel::new();
        assert!(!panel.is_visible());

        panel.toggle();
        assert!(panel.is_visible());

        panel.toggle();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_show_hide() {
        let mut panel = ShortcutsHelpPanel::new();

        panel.show();
        assert!(panel.is_visible());

        panel.hide();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_show_when_already_visible() {
        let mut panel = ShortcutsHelpPanel::new();
        panel.show();
        assert!(panel.is_visible());

        // 再次调用 show 应保持可见
        panel.show();
        assert!(panel.is_visible());
    }

    #[test]
    fn test_hide_when_already_hidden() {
        let mut panel = ShortcutsHelpPanel::new();
        assert!(!panel.is_visible());

        // 再次调用 hide 应保持隐藏
        panel.hide();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_multiple_toggles() {
        let mut panel = ShortcutsHelpPanel::new();

        for i in 1..=10 {
            panel.toggle();
            assert_eq!(panel.is_visible(), i % 2 == 1, "Toggle {} failed", i + 1);
        }
    }

    #[test]
    fn test_toggle_twice_returns_to_original() {
        let mut panel = ShortcutsHelpPanel::new();
        assert!(!panel.is_visible());

        panel.toggle();
        panel.toggle();

        assert!(!panel.is_visible());
    }

    // =========================================================================
    // 边界条件测试
    // =========================================================================

    #[test]
    fn test_visibility_consistency() {
        let mut panel = ShortcutsHelpPanel::new();

        // 复杂状态转换序列
        panel.show();
        panel.show();
        panel.toggle();
        assert!(!panel.is_visible());
        panel.hide();
        assert!(!panel.is_visible());
        panel.toggle();
        assert!(panel.is_visible());
        panel.show();
        assert!(panel.is_visible());
    }

    #[test]
    fn test_clone_panel() {
        let mut panel = ShortcutsHelpPanel::new();
        panel.show();

        let cloned = panel.clone();
        assert_eq!(panel.is_visible(), cloned.is_visible());
        assert_eq!(panel.visible, cloned.visible);
    }

    #[test]
    fn test_clone_hidden_panel() {
        let panel = ShortcutsHelpPanel::new();
        let cloned = panel.clone();
        assert!(!cloned.is_visible());
    }

    #[test]
    fn test_debug_format() {
        let panel = ShortcutsHelpPanel::new();
        let debug_str = format!("{:?}", panel);
        assert!(debug_str.contains("ShortcutsHelpPanel"));
        assert!(debug_str.contains("visible"));
        assert!(debug_str.contains("false"));
    }

    #[test]
    fn test_debug_format_visible() {
        let mut panel = ShortcutsHelpPanel::new();
        panel.show();
        let debug_str = format!("{:?}", panel);
        assert!(debug_str.contains("true"));
    }

    // =========================================================================
    // 状态组合测试
    // =========================================================================

    #[test]
    fn test_all_state_transitions() {
        let mut panel = ShortcutsHelpPanel::new();

        // 初始状态：隐藏
        assert!(!panel.is_visible());

        // 调用 show()：显示
        panel.show();
        assert!(panel.is_visible());

        // 再次调用 show()：仍显示
        panel.show();
        assert!(panel.is_visible());

        // 调用 toggle()：隐藏
        panel.toggle();
        assert!(!panel.is_visible());

        // 再次调用 toggle()：显示
        panel.toggle();
        assert!(panel.is_visible());

        // 调用 hide()：隐藏
        panel.hide();
        assert!(!panel.is_visible());

        // 再次调用 hide()：仍隐藏
        panel.hide();
        assert!(!panel.is_visible());

        // 调用 toggle()：显示
        panel.toggle();
        assert!(panel.is_visible());

        // 再次调用 toggle()：隐藏
        panel.toggle();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_multiple_panels_independent() {
        let mut panel1 = ShortcutsHelpPanel::new();
        let mut panel2 = ShortcutsHelpPanel::new();

        panel1.show();
        assert!(panel1.is_visible());
        assert!(!panel2.is_visible());

        panel2.toggle();
        assert!(panel1.is_visible());
        assert!(panel2.is_visible());

        panel1.hide();
        assert!(!panel1.is_visible());
        assert!(panel2.is_visible());
    }

    #[test]
    fn test_rapid_toggle() {
        let mut panel = ShortcutsHelpPanel::new();

        for _ in 0..100 {
            panel.toggle();
        }

        // 100 次 toggle 后应该回到初始状态（隐藏）
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_alternating_show_hide() {
        let mut panel = ShortcutsHelpPanel::new();

        for i in 0..20 {
            if i % 2 == 0 {
                panel.show();
                assert!(panel.is_visible());
            } else {
                panel.hide();
                assert!(!panel.is_visible());
            }
        }
    }
}
