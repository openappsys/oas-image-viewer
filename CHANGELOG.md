# Changelog

## 0.3.0 - 2026-03-03

### 🏗️ 架构重构 - Clean Architecture

本次版本进行了重大架构重构，采用 Clean Architecture 设计理念，将代码组织为清晰的分层结构，提高可维护性、可测试性和可扩展性。

#### 📁 新目录结构

```
src/
├── main.rs              # 应用程序入口点
├── config.rs            # 配置管理（Domain层）
├── app/                 # 核心应用逻辑（Domain层）
│   └── mod.rs           # ImageViewerApp 主控制器
├── viewer/              # 图像查看器（Application层）
│   └── mod.rs           # Viewer 组件
├── gallery/             # 图库模块（Application层）
│   └── mod.rs           # Gallery 组件
├── decoder/             # 图像解码器（Application层）
│   └── mod.rs           # ImageDecoder
├── dnd/                 # 拖放处理（Infrastructure层）
│   └── mod.rs           # 拖放事件处理
├── clipboard/           # 剪贴板操作（Infrastructure层）
│   └── mod.rs           # ClipboardManager
├── info_panel.rs        # 信息面板（UI层）
├── shortcuts_help.rs    # 快捷键帮助（UI层）
└── utils/               # 工具函数（Infrastructure层）
    ├── mod.rs           # 通用工具函数
    ├── errors.rs        # 错误类型定义
    └── threading.rs     # 线程工具
```

#### 🏛️ 架构分层说明

| 层级 | 模块 | 职责 |
|------|------|------|
| **Domain** | `app`, `config` | 核心业务逻辑、应用状态管理 |
| **Application** | `viewer`, `gallery`, `decoder` | 功能组件、图像处理、解码 |
| **Infrastructure** | `utils`, `dnd`, `clipboard` | 工具函数、系统交互、I/O操作 |
| **UI** | `main.rs`, `info_panel`, `shortcuts_help` | 界面渲染、用户交互 |

#### ✅ 测试覆盖

- **451+ 单元测试** 通过
- 测试分布：
  - `app`: 90+ 测试用例（状态管理、导航、快捷键）
  - `viewer`: 60+ 测试用例（缩放、渲染、图像操作）
  - `gallery`: 70+ 测试用例（缩略图、导航、选择）
  - `decoder`: 50+ 测试用例（格式检测、解码）
  - `dnd`: 40+ 测试用例（拖放处理、文件过滤）
  - `clipboard`: 30+ 测试用例（复制操作、错误处理）
  - `utils`: 40+ 测试用例（工具函数、错误类型）
- 集成测试：`tests/` 目录包含 decoder 和 dnd 集成测试

#### 🔄 迁移说明

**从 v0.2.x 迁移到 v0.3.0：**

1. **配置文件**：配置文件位置保持不变，自动兼容
2. **API 变更**：内部模块结构重组，对外功能不变
3. **构建命令**：无变化，`cargo build --release` 即可
4. **依赖更新**：无新增依赖，现有依赖版本保持兼容

#### 🐛 修复

- 修复了缩略图异步加载的内存泄漏问题
- 优化了图像缩放算法，提高性能
- 修复了全屏模式下菜单栏显示问题

#### ⚡ 性能优化

- 缩略图采用后台线程异步加载
- 图像解码增加双重容错机制
- 配置保存增加防抖处理

---

## 0.2.0 - 2026-02-15

### 新增功能

- 🖼️ **画廊模式**：G键切换，支持缩略图网格浏览
- 📁 **拖放支持**：支持拖拽图片文件到窗口打开
- ⌨️ **快捷键系统**：完整快捷键支持，? 键显示帮助
- ℹ️ **信息面板**：F键切换，显示图片元数据
- 📋 **剪贴板集成**：右键菜单复制图片和路径

### 改进

- 优化了图像加载性能
- 改进了中文界面显示
- 增加了更多配置选项

---

## 0.1.0 - 2026-01-20

### 初始版本

- 🚀 首次发布
- 📷 支持 PNG/JPEG/GIF/WebP/TIFF/BMP 格式
- 🔍 基础缩放和平移功能
- 🖥️ 全屏模式支持
- ⚙️ 基础配置系统
