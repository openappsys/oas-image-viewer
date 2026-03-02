# Image-Viewer 架构文档

**版本**: v0.3.0  
**日期**: 2026-03-03  
**作者**: Image-Viewer Team

---

## 目录

1. [概述](#概述)
2. [架构设计原则](#架构设计原则)
3. [分层架构](#分层架构)
4. [模块详细说明](#模块详细说明)
5. [模块依赖图](#模块依赖图)
6. [数据流向](#数据流向)
7. [设计模式](#设计模式)
8. [测试策略](#测试策略)

---

## 概述

Image-Viewer 采用 **Clean Architecture**（整洁架构）设计，将应用程序分为清晰的层次结构，每层有明确的职责和依赖关系。这种架构确保了：

- **可维护性**：代码组织清晰，易于理解和修改
- **可测试性**：业务逻辑与框架解耦，便于单元测试
- **可扩展性**：新功能可以通过添加模块实现，不影响现有代码
- **灵活性**：技术实现细节（如 UI 框架）可以替换而不影响核心业务逻辑

---

## 架构设计原则

### 依赖规则

```
UI 层 ↑ 依赖
Application 层 ↑ 依赖
Domain 层 ↑ 依赖
Infrastructure 层（最底层，不依赖其他层）
```

**核心原则**：
1. 内层不依赖外层
2. 依赖方向向内指向 Domain 层
3. 通过接口/traits 解耦层间依赖

### 分层职责

| 层级 | 职责 | 变化频率 |
|------|------|----------|
| Domain | 核心业务逻辑、实体、规则 | 低 |
| Application | 用例实现、业务流程编排 | 中 |
| Infrastructure | 技术细节、外部服务、I/O | 高 |
| UI | 用户界面、输入输出呈现 | 最高 |

---

## 分层架构

### 1. Domain 层（领域层）

**文件位置**: `src/app/`, `src/config.rs`

**职责**:
- 定义核心业务实体和规则
- 管理应用程序状态
- 协调各 Application 组件
- 独立于任何外部框架

**核心组件**:

```rust
// app/mod.rs
pub struct ImageViewerApp {
    config: Config,                    // 配置
    gallery: Gallery,                  // 图库组件
    viewer: Viewer,                    // 查看器组件
    current_view: View,                // 当前视图状态
    image_list: Vec<PathBuf>,          // 图像列表
    current_index: usize,              // 当前索引
    decoder: ImageDecoder,             // 解码器
    // ... 其他状态
}
```

**状态管理**:
```
ImageViewerApp
├── View 枚举: Gallery | Viewer
├── 图像列表管理
├── 配置持久化
└── 事件协调
```

### 2. Application 层（应用层）

**文件位置**: `src/viewer/`, `src/gallery/`, `src/decoder/`

**职责**:
- 实现具体用例
- 编排 Domain 层对象完成业务功能
- 不包含业务规则，只协调工作流程

#### 2.1 Viewer 模块

**文件**: `src/viewer/mod.rs`

**职责**: 图像查看和交互

```rust
pub struct Viewer {
    config: ViewerConfig,
    current_image: Option<ViewImage>,
    scale: f32,                        // 缩放比例
    offset: Vec2,                      // 平移偏移
    info_panel: InfoPanel,
    clipboard: ClipboardManager,
}
```

**功能**:
- 图像渲染
- 缩放/平移控制
- 右键菜单（复制、在文件夹中显示）
- 信息面板集成

#### 2.2 Gallery 模块

**文件**: `src/gallery/mod.rs`

**职责**: 缩略图网格浏览

```rust
pub struct Gallery {
    config: GalleryConfig,
    images: Vec<GalleryImage>,
    selected_index: Option<usize>,
    thumbnail_loader: Option<ThumbnailLoader>,
}

pub struct ThumbnailLoader {
    sender: Sender<ThumbnailRequest>,
    receiver: Receiver<ThumbnailResult>,
}
```

**功能**:
- 异步缩略图加载
- 网格布局渲染
- 键盘导航
- 选择管理

#### 2.3 Decoder 模块

**文件**: `src/decoder/mod.rs`

**职责**: 图像格式解码

```rust
pub struct ImageDecoder;

pub enum ImageFormat {
    Png, Jpeg, Gif, Webp, Tiff, Bmp,
}
```

**功能**:
- 多格式解码（PNG/JPEG/GIF/WebP/TIFF/BMP）
- 自动格式检测（magic number）
- 双重容错解码机制

### 3. Infrastructure 层（基础设施层）

**文件位置**: `src/utils/`, `src/dnd/`, `src/clipboard/`

**职责**:
- 提供技术能力支持
- 封装外部服务访问
- 工具函数和通用功能

#### 3.1 Utils 模块

**文件**: `src/utils/mod.rs`, `src/utils/errors.rs`, `src/utils/threading.rs`

**功能**:
```rust
// 工具函数
pub fn format_file_size(size: u64) -> String;
pub fn file_name_from_path(path: &Path) -> String;
pub fn is_image_file(path: &Path) -> bool;

// 错误类型
pub enum AppError { ... }
pub enum DecoderError { ... }
pub enum GalleryError { ... }
```

#### 3.2 DnD 模块（拖放）

**文件**: `src/dnd/mod.rs`

**功能**:
```rust
pub fn extract_image_files(raw_files: &[DroppedFile]) -> Vec<PathBuf>;
pub fn is_drag_hovering(ctx: &Context) -> bool;
pub fn get_drag_preview_text(ctx: &Context) -> Option<String>;
```

#### 3.3 Clipboard 模块

**文件**: `src/clipboard/mod.rs`

**功能**:
```rust
impl ClipboardManager {
    pub fn copy_text(&mut self, text: &str) -> Result<()>;
    pub fn copy_image(&mut self, data: &[u8], width: usize, height: usize) -> Result<()>;
    pub fn copy_image_path(&mut self, path: &Path) -> Result<()>;
    pub fn show_in_folder(path: &Path) -> Result<()>;
}
```

### 4. UI 层（用户界面层）

**文件位置**: `src/main.rs`, `src/info_panel.rs`, `src/shortcuts_help.rs`

**职责**:
- 处理用户输入
- 渲染界面
- 调用 Application 层完成功能

#### 4.1 Main 模块

**文件**: `src/main.rs`

**职责**:
- 应用程序入口
- 字体配置（中文字体支持）
- 窗口初始化
- 启动 ImageViewerApp

#### 4.2 Info Panel 模块

**文件**: `src/info_panel.rs`

**功能**: 显示图像元数据信息面板

#### 4.3 Shortcuts Help 模块

**文件**: `src/shortcuts_help.rs`

**功能**: 快捷键帮助面板

---

## 模块详细说明

### 模块职责矩阵

| 模块 | 所属层 | 主要职责 | 测试数量 |
|------|--------|----------|----------|
| `app` | Domain | 应用状态、事件协调 | 90+ |
| `config` | Domain | 配置管理 | 30+ |
| `viewer` | Application | 图像查看、缩放平移 | 60+ |
| `gallery` | Application | 缩略图、网格浏览 | 70+ |
| `decoder` | Application | 图像解码 | 50+ |
| `dnd` | Infrastructure | 拖放处理 | 40+ |
| `clipboard` | Infrastructure | 剪贴板操作 | 30+ |
| `utils` | Infrastructure | 工具函数、错误 | 40+ |
| `info_panel` | UI | 信息面板 | 20+ |
| `shortcuts_help` | UI | 快捷键帮助 | 15+ |

### 关键数据结构

```rust
// 视图状态
enum View {
    Gallery,
    Viewer,
}

// 图像信息
struct ViewImage {
    path: PathBuf,
    texture: Option<TextureHandle>,
    dimensions: Option<(u32, u32)>,
}

// 图库图像
struct GalleryImage {
    path: PathBuf,
    thumbnail: Option<TextureHandle>,
    is_loading: bool,
}

// 导航动作
enum NavAction {
    None,
    SelectAndOpen(usize),
}
```

---

## 模块依赖图

### 完整依赖图

```
                           ┌─────────────────┐
                           │   main.rs       │
                           │   (UI 入口)     │
                           └────────┬────────┘
                                    │
                                    ▼
                           ┌─────────────────┐
                           │  ImageViewerApp │◄─────────────────────┐
                           │   (Domain)      │                      │
                           └────────┬────────┘                      │
                                    │                               │
           ┌────────────────────────┼────────────────────────┐      │
           │                        │                        │      │
           ▼                        ▼                        ▼      │
    ┌─────────────┐         ┌─────────────┐         ┌─────────────┐ │
    │   Viewer    │         │   Gallery   │         │   Decoder   │ │
    │(Application)│         │(Application)│         │(Application)│ │
    └──────┬──────┘         └──────┬──────┘         └─────────────┘ │
           │                       │                                │
           │    ┌──────────────────┘                                │
           │    │                                                   │
           ▼    ▼                                                   │
    ┌─────────────────────────────────────┐                        │
    │           Infrastructure            │                        │
    │  ┌─────────┐ ┌─────────┐ ┌────────┐ │                        │
    │  │   dnd   │ │clipboard│ │ utils  │ │                        │
    │  └─────────┘ └─────────┘ └────────┘ │                        │
    └─────────────────────────────────────┘                        │
                                                                   │
    ┌─────────────────────────────────────┐                        │
    │              UI                     │────────────────────────┘
    │  ┌─────────────┐ ┌────────────────┐ │   (App 控制 UI)
    │  │ info_panel  │ │shortcuts_help  │ │
    │  └─────────────┘ └────────────────┘ │
    └─────────────────────────────────────┘
```

### 依赖关系表

| 模块 | 依赖的模块 | 被依赖的模块 |
|------|-----------|-------------|
| main.rs | app, config | - |
| app | viewer, gallery, decoder, config, dnd, utils, clipboard | main.rs |
| viewer | config, clipboard, info_panel, utils | app |
| gallery | config, utils | app |
| decoder | utils | app |
| dnd | utils | app |
| clipboard | - | viewer |
| utils | - | app, viewer, gallery, decoder, dnd |
| info_panel | - | viewer |
| shortcuts_help | - | app |

---

## 数据流向

### 1. 打开图像流程

```
用户输入 (Ctrl+O / 拖放)
    │
    ▼
UI 层接收事件
    │
    ▼
App::show_open_dialog() / App::handle_drops()
    │
    ▼
App::open_image(path)
    │
    ├──► Decoder::decode_from_file() ──► 图像数据
    │
    ▼
egui::Context::load_texture() ──► TextureHandle
    │
    ▼
Viewer::set_image_with_texture()
    │
    ▼
UI 渲染更新
```

### 2. 缩略图加载流程

```
Gallery::add_image(path)
    │
    ▼
ThumbnailLoader::request(index, path)
    │
    ▼
后台线程 ──► image::open() ──► resize ──► texture
    │
    ▼
channel ──► Gallery::process_async_results()
    │
    ▼
UI 更新缩略图
```

### 3. 用户交互流程

```
用户输入 (键盘/鼠标)
    │
    ▼
eframe 事件循环
    │
    ▼
App::update()
    │
    ├──► handle_shortcuts()
    │    ├──► 导航控制 (prev/next)
    │    ├──► 视图切换 (Gallery/Viewer)
    │    └──► 功能调用 (zoom, fullscreen)
    │
    ├──► viewer.ui() / gallery.ui()
    │
    └──► 渲染输出
```

---

## 设计模式

### 1. 分层架构模式 (Layered Architecture)

将应用程序分为 Domain、Application、Infrastructure、UI 四层，每层有明确的职责。

### 2. 组件模式 (Component Pattern)

```rust
// 每个功能模块作为独立组件
pub struct Gallery { ... }
pub struct Viewer { ... }
pub struct ImageDecoder { ... }

// 组件通过组合集成到 App
pub struct ImageViewerApp {
    gallery: Gallery,
    viewer: Viewer,
    decoder: ImageDecoder,
}
```

### 3. 状态机模式 (State Machine)

```rust
enum View {
    Gallery,    // 图库视图
    Viewer,     // 查看器视图
}

// 状态转换
fn toggle_view(&mut self) {
    match self.current_view {
        View::Gallery => self.current_view = View::Viewer,
        View::Viewer => self.current_view = View::Gallery,
    }
}
```

### 4. 生产者-消费者模式 (Producer-Consumer)

```rust
// 缩略图异步加载
struct ThumbnailLoader {
    sender: Sender<ThumbnailRequest>,      // 生产者
    receiver: Receiver<ThumbnailResult>,   // 消费者
}

// 后台线程生产，主线程消费
```

### 5. 防抖模式 (Debounce Pattern)

```rust
// 配置保存防抖
pub struct DebouncedConfigSaver {
    last_request: Option<Instant>,
    debounce_duration: Duration,
}
```

### 6. 错误处理模式

```rust
// 使用 thiserror 定义错误类型
#[derive(Error, Debug)]
pub enum DecoderError {
    #[error("不支持的图像格式")]
    UnsupportedFormat,
    #[error("解码图像失败: {0}")]
    DecodeFailed(String),
}

// 使用 anyhow 进行错误传播
pub fn open_image(&self, path: &Path) -> Result<DynamicImage> {
    image::open(path)
        .map_err(|e| DecoderError::DecodeFailed(e.to_string()))?
}
```

---

## 测试策略

### 测试金字塔

```
       ┌─────────┐
       │   E2E   │  集成测试 (tests/)
       │  (10%)  │
      ┌┴─────────┴┐
      │ Integration│  模块集成
      │   (20%)   │
     ┌┴───────────┴┐
     │    Unit      │  单元测试 (#[cfg(test)])
     │    (70%)     │  451+ 测试用例
     └─────────────┘
```

### 单元测试分布

| 模块 | 测试文件 | 测试数量 | 覆盖率重点 |
|------|---------|---------|-----------|
| app | `app/mod.rs` (tests) | 90+ | 状态管理、导航、事件处理 |
| viewer | `viewer/mod.rs` (tests) | 60+ | 缩放、渲染、图像操作 |
| gallery | `gallery/mod.rs` (tests) | 70+ | 缩略图、导航、选择 |
| decoder | `decoder/mod.rs` (tests) | 50+ | 格式检测、解码容错 |
| dnd | `dnd/mod.rs` (tests) | 40+ | 文件过滤、拖放处理 |
| clipboard | `clipboard/mod.rs` (tests) | 30+ | 复制操作、错误处理 |
| utils | `utils/*.rs` (tests) | 40+ | 工具函数、错误类型 |

### 测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_select_navigation() {
        let mut gallery = Gallery::new(GalleryConfig::default());
        
        // 添加测试图像
        for i in 0..5 {
            gallery.add_image(PathBuf::from(format!("img{}.png", i)));
        }
        
        // 测试导航
        assert!(gallery.select_image(0));
        assert!(gallery.select_next());
        assert_eq!(gallery.selected_index(), Some(1));
        assert!(gallery.select_prev());
        assert_eq!(gallery.selected_index(), Some(0));
    }

    #[test]
    fn test_viewer_zoom_boundaries() {
        let mut viewer = Viewer::new(ViewerConfig::default());
        
        // 测试缩放边界
        for _ in 0..100 {
            viewer.zoom_in();
        }
        assert!(viewer.scale() <= 20.0); // 最大限制
        
        viewer.reset_zoom();
        
        for _ in 0..100 {
            viewer.zoom_out();
        }
        assert!(viewer.scale() >= 0.1); // 最小限制
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --lib app::tests
cargo test --lib viewer::tests

# 运行集成测试
cargo test --test decoder_test
cargo test --test dnd_integration_test

# 生成覆盖率报告
cargo tarpaulin --out Html
```

---

## 总结

Image-Viewer v0.3.0 通过 Clean Architecture 实现了：

1. **清晰的代码组织**：四层架构，职责分明
2. **高可测试性**：451+ 单元测试，核心逻辑全覆盖
3. **技术无关性**：业务逻辑独立于 UI 框架
4. **易于扩展**：新功能通过添加模块实现
5. **易于维护**：依赖关系清晰，修改影响范围可控

这种架构设计确保了项目的长期健康发展，为后续功能扩展奠定了坚实基础。
