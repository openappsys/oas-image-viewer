# 图片查看器 (Image Viewer)

一款使用 Rust 和 egui 构建的现代、快速图片查看器。

[![CI](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 功能特性

- 🖼️ **多格式支持**：PNG、JPEG、GIF、WebP、TIFF、BMP
- 📁 **画廊视图**：以缩略图网格形式浏览图片
- 🔍 **缩放与平移**：鼠标滚轮平滑缩放，拖拽平移
- ⚡ **高性能**：使用 Rust 构建，性能卓越
- 🎨 **现代界面**：由 egui 驱动的简洁界面
- 🔧 **可配置**：通过配置文件进行自定义
- 🖥️ **跨平台**：支持 Windows、macOS 和 Linux

## 安装

### 环境要求

- [Rust](https://rust-lang.org/) 1.93 或更高版本
- 系统依赖（仅 Linux）：
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

  # Fedora
  sudo dnf install gtk3-devel libxcb-devel

  # Arch
  sudo pacman -S gtk3 libxcb
  ```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# 构建发布版本
cargo build --release

# 运行
./target/release/image-viewer
```

### 预编译二进制文件

从 [Releases](https://github.com/yourusername/image-viewer/releases) 页面下载预编译的二进制文件。

## 使用方法

### 基本用法

```bash
# 打开图片查看器
image-viewer

# 打开指定图片
image-viewer /path/to/image.png

# 打开文件夹
image-viewer /path/to/images/
```

### 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + O` | 打开文件 |
| `Ctrl + +` | 放大 |
| `Ctrl + -` | 缩小 |
| `Ctrl + 0` | 重置缩放 |
| `← / →` | 上一张/下一张图片 |
| `F11` | 切换全屏模式 |
| `Esc` | 退出全屏/关闭查看器 |

### 鼠标控制

- **滚轮**：放大/缩小
- **拖拽**：缩放时平移
- **双击**：切换全屏模式
- **右键**：上下文菜单

## 配置说明

配置文件存储在平台特定的位置：

- **Linux**：`~/.config/image-viewer/config.toml`
- **macOS**：`~/Library/Application Support/com.imageviewer.image-viewer/config.toml`
- **Windows**：`%APPDATA%\\image-viewer\\config\\config.toml`

### 配置示例

```toml
[window]
width = 1200.0
height = 800.0
maximized = false

[gallery]
thumbnail_size = 150
items_per_row = 4

[viewer]
background_color = [30, 30, 30]
fit_to_window = true
show_info_panel = true
```

## 开发指南

### 项目结构

```
image-viewer/
├── src/
│   ├── main.rs          # 应用程序入口
│   ├── config.rs        # 配置管理
│   ├── app/             # 主应用程序逻辑
│   ├── viewer/          # 图片查看器组件
│   ├── gallery/         # 画廊网格组件
│   ├── decoder/         # 图片解码
│   └── utils/           # 工具（错误处理、线程）
├── assets/              # 静态资源
├── .github/workflows/   # CI/CD 配置
└── Cargo.toml          # 依赖项
```

### 开发命令

```bash
# 开发模式运行
cargo run

# 运行测试
cargo test

# 检查代码格式
cargo fmt -- --check

# 运行 clippy
cargo clippy -- -D warnings

# 构建发布版本
cargo build --release
```

### 代码质量

本项目强制执行严格的代码质量：

- **rustfmt**：一致的代码格式（`rustfmt.toml`）
- **clippy**：严格的代码检查（`.clippy.toml`）
- **EditorConfig**：一致的编辑器设置（`.editorconfig`）

## 架构设计

### 技术栈

- **GUI 框架**：[egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **图片解码**：[image](https://github.com/image-rs/image) crate
- **错误处理**：[anyhow](https://github.com/dtolnay/anyhow) + [thiserror](https://github.com/dtolnay/thiserror)
- **日志记录**：[tracing](https://github.com/tokio-rs/tracing)
- **配置管理**：[serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml)
- **多线程**：[rayon](https://github.com/rayon-rs/rayon) 用于并行处理

### 模块概览

| 模块 | 用途 |
|------|------|
| `app` | 主应用程序状态和事件循环 |
| `viewer` | 带缩放/平移的全尺寸图片显示 |
| `gallery` | 用于浏览的缩略图网格 |
| `decoder` | 图片格式解码 |
| `utils` | 错误类型和线程工具 |

## 参与贡献

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

请确保：
- 代码已使用 `cargo fmt` 格式化
- Clippy 检查通过 `cargo clippy -- -D warnings`
- 所有测试通过 `cargo test`

## 开发路线图

- [ ] 幻灯片模式
- [ ] EXIF 元数据显示
- [ ] 基本图片编辑（旋转、裁剪）
- [ ] 自定义主题
- [ ] 插件系统
- [ ] RAW 图片支持

## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

- [egui](https://github.com/emilk/egui) - 优秀的即时模式 GUI 库
- [image-rs](https://github.com/image-rs) - 图片解码库
- Rust 生态系统的所有贡献者
