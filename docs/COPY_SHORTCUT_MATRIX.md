# 复制快捷键行为矩阵

本文档定义图片复制、路径复制与文本复制的行为基线，用于回归测试与问题排查。

## 平台按键映射

- macOS：主修饰键为 `Cmd`
- Linux / Windows：主修饰键为 `Ctrl`
- 语义映射：
  - `Cmd+C` ↔ `Ctrl+C`
  - `Cmd+Shift+C` ↔ `Ctrl+Shift+C`

## 场景矩阵

| 判断项 | Cmd+C（无文本选中） | Cmd+Shift+C（无文本选中） | Cmd+C（信息面板有文本选中） | Cmd+Shift+C（信息面板有文本选中） |
|---|---|---|---|---|
| 应该复制图片 | √ | x | x | x |
| 不应该复制图片 | x | √ | √ | √ |
| 应该复制图片路径 | x | √ | x | x |
| 不应该复制图片路径 | √ | x | √ | √ |
| 应该复制信息面板里选中的文字 | x | x | √ | x |
| 不应该复制信息面板里选中的文字 | √ | √ | x | √ |
| 应该提示：复制图片 | √ | x | x | x |
| 不应该提示：复制图片 | x | √ | √ | √ |
| 应该提示：复制图片路径 | x | √ | x | x |
| 不应该提示：复制图片路径 | √ | x | √ | √ |

### Linux / Windows 对应矩阵

| 判断项 | Ctrl+C（无文本选中） | Ctrl+Shift+C（无文本选中） | Ctrl+C（信息面板有文本选中） | Ctrl+Shift+C（信息面板有文本选中） |
|---|---|---|---|---|
| 应该复制图片 | √ | x | x | x |
| 不应该复制图片 | x | √ | √ | √ |
| 应该复制图片路径 | x | √ | x | x |
| 不应该复制图片路径 | √ | x | √ | √ |
| 应该复制信息面板里选中的文字 | x | x | √ | x |
| 不应该复制信息面板里选中的文字 | √ | √ | x | √ |
| 应该提示：复制图片 | √ | x | x | x |
| 不应该提示：复制图片 | x | √ | √ | √ |
| 应该提示：复制图片路径 | x | √ | x | x |
| 不应该提示：复制图片路径 | √ | x | √ | √ |

## 回归测试入口

- 决策矩阵单测：`src/adapters/egui/app.rs` 中 `matrix_cmd_*` 与 `matrix_ctrl_*` 用例。
- 信号与决策测试：`src/adapters/egui/app/copy_shortcuts.rs` 中 `integration_*` 用例。
- 事件前置拦截实现：`src/adapters/egui/app/shortcuts.rs` 中 `suppress_shift_copy_shortcut_before_widgets`。

## 建议执行命令

```bash
cargo test matrix_cmd_c_no_text_selected
cargo test matrix_cmd_shift_c_no_text_selected
cargo test matrix_cmd_c_with_text_selected
cargo test matrix_cmd_shift_c_with_text_selected
cargo test matrix_ctrl_c_no_text_selected
cargo test matrix_ctrl_shift_c_no_text_selected
cargo test matrix_ctrl_c_with_text_selected
cargo test matrix_ctrl_shift_c_with_text_selected
cargo test integration_cmd_shift_c_with_focused_widget_does_not_copy_path
cargo test integration_copy_event_with_shift_and_focused_widget_is_consumed
```

## 变更准入规则

- 涉及复制快捷键逻辑的改动，必须在本矩阵 4 个核心场景下人工回归。
- 同时必须通过上述单测与全量测试：`cargo test`。
