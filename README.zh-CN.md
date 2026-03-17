# OAS Image Viewer

一个使用 Rust 和 egui 构建的现代化、高性能图片查看器。

[![CI](https://github.com/openappsys/oas-image-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/openappsys/oas-image-viewer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English Documentation](README.md) | [中文文档](README.zh-CN.md)
## 特性

- 🖼️ **多格式支持**：PNG、JPEG、GIF、WebP、TIFF、BMP
- 📁 **画廊视图**：缩略图网格浏览
- 🔍 **缩放与平移**：鼠标滚轮缩放，拖拽平移
- ⚡ **高性能**：Rust 构建，极致性能
- 🎨 **现代化 UI**：基于 egui 的简洁界面
- 🔧 **可配置**：通过配置文件自定义
- 🖥️ **跨平台**：支持 Windows、macOS、Linux
- 🧪 **质量导向测试**：核心流程包含单元测试与集成测试

## 架构说明

本项目采用 **分层架构** 架构设计，将代码组织为清晰的分层结构：

```
┌─────────────────────────────────────────────────────────────────────┐
│                         入口层 (Entry)                              │
│  ┌──────────────────────┐  ┌────────────────────────────────────┐  │
│  │       main.rs        │  │               lib.rs               │  │
│  │ 启动流程/参数/窗口配置 │  │ 模块导出与公共 API 组织            │  │
│  └──────────────────────┘  └────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                      适配器层 (Adapters)                             │
│  ┌────────────────────────────┐ ┌────────────────────────────────┐  │
│  │       adapters/egui        │ │       adapters/platform        │  │
│  │ UI渲染/菜单/输入/状态同步   │ │ Linux/macOS/Windows 系统集成  │  │
│  └────────────────────────────┘ └────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                         核心层 (Core)                                │
│  ┌────────────────────┐ ┌────────────────────┐ ┌─────────────────┐  │
│  │    core/domain     │ │     core/ports     │ │ core/use_cases  │  │
│  │   实体与值对象     │ │    抽象接口契约    │ │ 用例与应用服务  │  │
│  └────────────────────┘ └────────────────────┘ └─────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                    基础设施层 (Infrastructure)                        │
│  ┌────────────────────┐ ┌────────────────────┐ ┌─────────────────┐  │
│  │     JsonStorage    │ │   FsImageSource    │ │ 文件系统与I/O   │  │
│  │     配置存储       │ │      图片来源      │ │ 与技术实现细节  │  │
│  └────────────────────┘ └────────────────────┘ └─────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 层间依赖

```
入口层（main.rs / lib.rs）
        ↓
适配器层（adapters/egui + adapters/platform）
        ↓
核心层（core/domain + core/ports + core/use_cases）
        ↓
基础设施层（infrastructure/）
```

## 安装

### 前提条件

- Rust 1.94 或更高版本
- Linux 系统依赖：
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
git clone git@github.com:openappsys/oas-image-viewer.git
cd oas-image-viewer

# 构建 release 版本
cargo build --release

# 运行
./target/release/oas-image-viewer
```

### 预构建二进制文件

从 [Releases](https://github.com/openappsys/oas-image-viewer/releases) 页面下载预构建的二进制文件。

#### ⚠️ 安全提示（首次运行）

由于当前版本暂未进行代码签名，首次运行时系统可能会显示安全警告，请按以下步骤操作：

**Windows:**
1. 运行程序时若显示 "Windows 已保护你的电脑"
2. 点击 **"更多信息"**（More info）
3. 点击 **"仍要运行"**（Run anyway）
4. 下次打开将不再提示

> 💡 或者：右键点击程序 → 属性 → 勾选"解除锁定"（Unblock）→ 应用

**macOS:**
1. 首次打开时若显示 "无法验证开发者"
2. 点击 **"取消"**
3. 打开 **系统设置 → 隐私与安全性**
4. 在"安全性"下方找到 "OAS Image Viewer"，点击 **"仍要打开"**
5. 再次点击 **"打开"** 确认

> 💡 或者：右键点击应用 → 选择"打开" → 点击"打开"确认
> 
> 💡 终端命令（快速解除）：
> ```bash
> xattr -d com.apple.quarantine /Applications/Image\ Viewer.app
> ```

**Linux:**
```bash
# 赋予执行权限后运行
chmod +x ./oas-image-viewer
./oas-image-viewer
```

> 💡 **推荐**: 使用 AppImage 格式，无需安装，双击即可运行

**为什么有这个提示？**
- 代码签名证书需要年费和公司资质审核
- 我们正在准备正式签名，将在 v1.0 版本解决
- 所有源码开源可审计，请放心使用

## 使用

### 基础用法

```bash
# 启动图片查看器
oas-image-viewer

# 打开指定图片
oas-image-viewer /path/to/image.png

# 打开目录
oas-image-viewer /path/to/images/

# 调试日志方式打开
RUST_LOG=debug oas-image-viewer /path/to/image.png
```

### 快捷键

| 按键 | 功能 | 模式 |
|-----|------|------|
| `→` | 下一张图片 | 全部 |
| `←` | 上一张图片 | 全部 |
| `Ctrl + O` | 打开文件对话框 | 全部 |
| `Ctrl + Shift + O` | 打开文件夹对话框 | 全部 |
| `Ctrl + C` | 复制当前图片 | 查看器 |
| `Ctrl + Shift + C` | 复制当前图片路径 | 查看器 |
| `Ctrl + + / Ctrl + -` | 放大/缩小 | 查看器 |
| `Ctrl + 0` | 适应窗口 | 查看器 |
| `Ctrl + 1` | 原始尺寸（100%） | 查看器 |
| `F11` | 切换全屏 | 全部 |
| `G` | 切换画廊模式 | 全部 |
| `F` | 显示/隐藏信息面板 | 查看器 |
| `?` | 显示/隐藏快捷键帮助 | 全部 |
| `Esc` | 退出全屏 / 关闭浮层 | 全部 |

### 鼠标操作

- **单击**：选择图片 / 打开图片
- **双击**：切换全屏
- **滚轮**：缩放（查看器模式）/ 滚动（画廊模式）
- **拖拽**：平移图片（查看器模式）
- **右键**：上下文菜单

## 配置

配置文件位置由 `directories::ProjectDirs` 自动解析：

- **Linux**: `~/.config/oas-image-viewer/config.toml`
- **macOS**: `~/Library/Application Support/com.openappsys.oas-image-viewer/config.toml`
- **Windows**: `%APPDATA%\openappsys\oas-image-viewer\config\config.toml`

配置示例：

```toml
[window]
# 窗口设置
width = 1200.0
height = 800.0
maximized = false

[gallery]
# 画廊设置
thumbnail_size = 120
items_per_row = 0
grid_spacing = 12.0
show_filenames = true

[viewer]
# 查看器设置
background_color = [30, 30, 30]
fit_to_window = true
show_info_panel = false
min_scale = 0.1
max_scale = 20.0
zoom_step = 1.25
smooth_scroll = true
```

## 开发

### 项目结构

```
oas-image-viewer/
├── Cargo.toml           # 项目配置
├── config.example.toml  # 配置模板
├── src/
│   ├── main.rs         # 入口点
│   ├── lib.rs          # 库根
│   ├── adapters/       # 适配器层（UI + 平台集成）
│   │   ├── egui/       # egui UI 适配器
│   │   ├── platform/   # 系统集成（linux/macos/windows）
│   │   └── macos_file_open.rs
│   ├── core/           # 核心业务层（Domain + Use Cases）
│   │   ├── domain/     # 实体、值对象
│   │   ├── ports/      # 接口定义
│   │   └── use_cases/  # 用例实现
│   └── infrastructure/ # 基础设施层
│       └── mod.rs
├── tests/              # 集成测试
├── docs/               # 文档
├── assets/             # 图标、资源
└── scripts/            # 构建脚本
```

### 构建与测试

```bash
# 开发构建
cargo build

# Release 构建
cargo build --release

# 运行测试
cargo test

# 带日志运行
RUST_LOG=debug cargo run

# 代码检查
cargo clippy -- -D warnings
```

### 技术栈

- **GUI 框架**: [egui](https://github.com/emilk/egui) - 即时模式 GUI
- **图像解码**: [image](https://github.com/image-rs/image) - Rust 图像库
- **并行处理**: [rayon](https://github.com/rayon-rs/rayon) - 并行迭代
- **配置**: [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml) - 配置序列化
- **日志**: [tracing](https://github.com/tokio-rs/tracing)
- **剪贴板**: [arboard](https://github.com/1Password/arboard) - 系统剪贴板操作

## 路线图

### v0.3.x（当前）
- ✅ 多格式图片支持
- ✅ 画廊视图
- ✅ 基础图片编辑（旋转、翻转）
- ✅ 拖拽支持
- ✅ 键盘快捷键

### v0.4.0（计划中）
- [ ] 图片编辑（裁剪、亮度/对比度调整）
- [ ] 批量处理
- [ ] 幻灯片模式
- [ ] 自定义主题
- [ ] 插件系统

### v1.0.0（正式版）
- [ ] 代码签名（Windows/macOS）
- [ ] 自动更新
- [ ] 云同步（可选）

完整路线图见 [ROADMAP.md](docs/ROADMAP.md)。

## 参与贡献

欢迎贡献！开始前请先阅读 [开发指南](docs/DEVELOPMENT.md)。

### 快速开始

1. Fork 本仓库
2. 创建功能分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'Add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 创建 Pull Request

请确保：
- 代码符合 `rustfmt` 风格
- 所有测试通过：`cargo test`
- 无 clippy 警告：`cargo clippy`

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [egui](https://github.com/emilk/egui) - 优秀的 Rust GUI 框架
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - egui 官方框架集成
- [image](https://github.com/image-rs/image) - Rust 图像处理库

## 支持

- 💬 [Discussions](https://github.com/openappsys/oas-image-viewer/discussions)
- 🐛 [Issues](https://github.com/openappsys/oas-image-viewer/issues)
- 📧 邮箱: team@openappsys.com

---

**Made with ❤️ using Rust**
