# Image Viewer

一个使用 Rust 和 egui 构建的现代化、高性能图片查看器。

[![CI](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 特性

- 🖼️ **多格式支持**：PNG、JPEG、GIF、WebP、TIFF、BMP
- 📁 **画廊视图**：缩略图网格浏览
- 🔍 **缩放与平移**：鼠标滚轮缩放，拖拽平移
- ⚡ **高性能**：Rust 构建，极致性能
- 🎨 **现代化 UI**：基于 egui 的简洁界面
- 🔧 **可配置**：通过配置文件自定义
- 🖥️ **跨平台**：支持 Windows、macOS、Linux
- 🧪 **高测试覆盖**：451+ 单元测试保障质量

## 架构说明

本项目采用 **分层架构** 架构设计，将代码组织为清晰的分层结构：

### 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                        UI 层                                 │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │   main.rs   │ │ info_panel  │ │   shortcuts_help    │   │
│  │  (入口点)   │ │  (信息面板)  │ │     (快捷键帮助)     │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                     Domain 层                               │
│  ┌─────────────────────────┐  ┌─────────────────────────┐   │
│  │      app/mod.rs         │  │       config.rs         │   │
│  │   (ImageViewerApp)      │  │      (配置管理)          │   │
│  │   应用状态、事件循环      │  │    配置加载与持久化       │   │
│  └─────────────────────────┘  └─────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                   Application 层                            │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐    │
│  │    viewer    │ │    gallery   │ │     decoder      │    │
│  │  (图像查看器) │ │    (图库)     │ │   (图像解码器)    │    │
│  │ 缩放/平移/渲染│ │ 缩略图/网格   │ │  多格式解码      │    │
│  └──────────────┘ └──────────────┘ └──────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                   Infrastructure 层                         │
│  ┌──────────┐ ┌──────────┐ ┌────────────────────────────┐  │
│  │   dnd    │ │ clipboard│ │          utils             │  │
│  │ (拖放)   │ │ (剪贴板)  │ │  errors / threading / ...  │  │
│  └──────────┘ └──────────┘ └────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 架构原则

1. **依赖方向**：内层不依赖外层，外层依赖内层
2. **Domain 层**：核心业务逻辑，独立于框架
3. **Application 层**：用例实现，编排 Domain 层
4. **Infrastructure 层**：技术细节，可替换的实现
5. **UI 层**：用户界面，最外层，最易变化

### 模块依赖图

```
                    main.rs
                      │
                      ▼
                ┌───────────┐
                │   app     │◄───────┐
                │  (Domain) │        │
                └─────┬─────┘        │
                      │              │
        ┌─────────────┼─────────────┐│
        │             │             ││
        ▼             ▼             ▼│
   ┌─────────┐  ┌──────────┐  ┌─────────┐
   │ viewer  │  │  gallery │  │ decoder │
   └────┬────┘  └────┬─────┘  └────┬────┘
        │            │             │
        └────────────┴─────────────┘
                     │
                     ▼
        ┌─────────────────────────────┐
        │        utils / dnd          │
        │      / clipboard            │
        │    (Infrastructure)         │
        └─────────────────────────────┘
```

### 数据流向

```
用户输入 → UI 层 → App 控制器 → Application 组件 → Infrastructure 服务
                                              ↓
显示更新 ← UI 层 ← App 控制器 ←←←←←←←←←← 数据返回
```

## 安装

### 前提条件

- Rust 1.93 或更高版本
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
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# 构建 release 版本
cargo build --release

# 运行
./target/release/image-viewer
```

### 预构建二进制文件

从 [Releases](https://github.com/yourusername/image-viewer/releases) 页面下载预构建的二进制文件。

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
4. 在"安全性"下方找到 "Image Viewer"，点击 **"仍要打开"**
5. 再次点击 **"打开"** 确认

> 💡 或者：右键点击应用 → 选择"打开" → 点击"打开"确认
> 
> 💡 终端命令（快速解除）：
> ```bash
> xattr -d com.apple.quarantine /Applications/Image\ Viewer.app

### Linux
```bash
# 赋予执行权限后运行
chmod +x ./image-viewer
./image-viewer
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
image-viewer

# 打开指定图片
image-viewer /path/to/image.png

# 打开目录
image-viewer /path/to/images/
```

### 快捷键

| 快捷键 | 操作 |
|--------|------|
| `Ctrl + O` | 打开文件 |
| `Ctrl + +` | 放大 |
| `Ctrl + -` | 缩小 |
| `Ctrl + 0` | 重置缩放 |
| `← / →` | 上一张/下一张 |
| `F11` | 切换全屏 |
| `Esc` | 退出全屏/关闭查看器 |
| `G` | 切换画廊/查看器 |
| `F` | 切换信息面板 |
| `?` | 显示快捷键帮助 |

### 鼠标控制

- **滚轮**：缩放
- **拖拽**：平移（缩放后）
- **双击**：切换全屏
- **右键**：上下文菜单

## 配置

配置文件位置（平台特定）：

- **Linux**: `~/.config/image-viewer/config.toml`
- **macOS**: `~/Library/Application Support/com.imageviewer.image-viewer/config.toml`
- **Windows**: `%APPDATA%\image-viewer\config\config.toml`

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

## 开发

### 项目结构

```
image-viewer/
├── src/
│   ├── main.rs              # 应用程序入口点
│   ├── config.rs            # 配置管理
│   ├── app/                 # 核心应用逻辑 (Domain)
│   ├── viewer/              # 图像查看器 (Application)
│   ├── gallery/             # 图库模块 (Application)
│   ├── decoder/             # 图像解码 (Application)
│   ├── dnd/                 # 拖放处理 (Infrastructure)
│   ├── clipboard/           # 剪贴板 (Infrastructure)
│   ├── info_panel.rs        # 信息面板 (UI)
│   ├── shortcuts_help.rs    # 快捷键帮助 (UI)
│   └── utils/               # 工具函数 (Infrastructure)
├── tests/                   # 集成测试
├── assets/                  # 静态资源
├── .github/workflows/       # CI/CD 配置
└── Cargo.toml              # 依赖管理
```

### 开发命令

```bash
# 开发模式运行
cargo run

# 运行测试
cargo test

# 运行带覆盖率测试
cargo tarpaulin

# 检查格式
cargo fmt -- --check

# 运行 clippy
cargo clippy -- -D warnings

# 构建 release
cargo build --release
```

### 代码质量

本项目强制执行严格的代码质量标准：

- **rustfmt**: 一致的代码格式 (`rustfmt.toml`)
- **clippy**: 严格规则的 linting (`.clippy.toml`)
- **EditorConfig**: 一致的编辑器设置 (`.editorconfig`)
- **测试覆盖**: 451+ 单元测试，核心模块覆盖率 >80%

## v0.3.0 新特性

### 🏗️ 架构重构

- 采用 分层架构 架构设计
- 清晰的四层架构：Domain → Application → Infrastructure → UI
- 模块间依赖关系明确，提高可维护性

### 🧪 测试增强

- 451+ 单元测试通过
- 每个模块都包含全面的单元测试
- 新增集成测试覆盖核心用例

### ⚡ 性能优化

- 缩略图后台线程异步加载
- 图像解码双重容错机制
- 配置保存防抖处理

### 🐛 Bug 修复

- 修复缩略图异步加载内存泄漏
- 优化图像缩放算法
- 修复全屏模式下菜单栏显示问题

## 技术栈

- **GUI 框架**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **图像解码**: [image](https://github.com/image-rs/image) crate
- **错误处理**: [anyhow](https://github.com/dtolnay/anyhow) + [thiserror](https://github.com/dtolnay/thiserror)
- **日志**: [tracing](https://github.com/tokio-rs/tracing)
- **配置**: [serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml)
- **并发**: [rayon](https://github.com/rayon-rs/rayon)
- **剪贴板**: [arboard](https://github.com/1Password/arboard)

## 参与贡献

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

请确保：
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy -- -D warnings` 通过 linting
- 使用 `cargo test` 通过所有测试

## 路线图

- [ ] 幻灯片模式
- [ ] EXIF 元数据显示
- [ ] 基础图像编辑（旋转、裁剪）
- [ ] 自定义主题
- [ ] 插件系统
- [ ] RAW 图像支持

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [egui](https://github.com/emilk/egui) 优秀的即时模式 GUI 库
- [image-rs](https://github.com/image-rs) 图像解码库
- 所有 Rust 生态系统的贡献者
