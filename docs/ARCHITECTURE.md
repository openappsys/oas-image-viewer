# OAS-Image-Viewer 架构文档

**版本**: v0.4.0  
**日期**: 2026-03-04  
**作者**: OAS-Image-Viewer Team

---

## 目录

1. [概述](#概述)
2. [架构设计原则](#架构设计原则)
3. [分层架构](#分层架构)
4. [模块详细说明](#模块详细说明)
5. [模块依赖图](#模块依赖图)
6. [数据流向](#数据流向)
7. [商业化架构扩展](#商业化架构扩展)
8. [测试策略](#测试策略)

---

## 概述

OAS-Image-Viewer 采用 **分层架构（轻量级 DDD）** 设计，将应用程序分为清晰的层次结构，每层有明确的职责和依赖关系。

### 核心架构特点

- **依赖反转**：内层定义接口（Ports），外层实现（Adapters）
- **Core 层纯净**：零外部依赖，纯业务逻辑
- **框架无关**：业务逻辑不依赖任何 UI 框架
- **高可测试性**：250+ 单元测试，核心模块覆盖率 >80%

### 文档状态约定

| 标记 | 含义 |
|------|------|
| **已实现** | 在 `src/` 中已有可运行实现 |
| **规划中** | 设计方向已确定，但尚未落地到 `src/` |
| **历史方案** | 方案讨论记录，用于决策追溯，不代表当前落地实现 |

### 架构目标

| 目标 | 说明 |
|------|------|
| **可维护性** | 代码组织清晰，易于理解和修改 |
| **可测试性** | 业务逻辑与框架解耦，便于单元测试 |
| **可扩展性** | 新功能通过添加模块实现，不影响现有代码 |
| **商业化支持** | 支持功能分层、账号系统、云同步等商业化需求 |

---

## 架构设计原则

### 依赖规则

```
┌─────────────────────────────────────────────────────────────┐
│                    依赖方向向内                              │
│                                                             │
│    Adapters (egui)  ──────────►  Core (Domain/Ports/UseCases)│
│    Infrastructure  ──────────►  Core                        │
│                                                             │
│    Core 层零外部依赖，完全独立                               │
└─────────────────────────────────────────────────────────────┘
```

**核心原则**：
1. 内层不依赖外层
2. 依赖方向向内指向 Core 层
3. 通过 Ports (traits) 解耦层间依赖

### 分层职责

| 层级 | 目录 | 职责 | 变化频率 |
|------|------|------|----------|
| **Core** | `src/core/` | 核心业务逻辑、实体、规则、用例 | 低 |
| **Infrastructure** | `src/infrastructure/` | 技术细节、外部服务、I/O 实现 | 高 |
| **Adapters** | `src/adapters/` | UI 框架适配、外部接口适配 | 中 |
| **Entry** | `src/main.rs` | 应用入口、依赖注入 | 低 |

---

## 分层架构

### 架构全景图

```
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs (入口)                          │
│                    依赖注入、应用启动                            │
├─────────────────────────────────────────────────────────────────┤
│                      Adapters (适配器层)                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   adapters/egui/                        │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │   │
│  │  │   EguiApp    │  │ ViewerWidget │  │GalleryWidget │  │   │
│  │  │  (协调层)    │  │  (UI 组件)    │  │  (UI 组件)    │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                        Core (核心层)                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  domain/         │  ports/          │  use_cases/       │   │
│  │  ┌────────────┐  │  ┌────────────┐  │  ┌─────────────┐ │   │
│  │  │ Image      │  │  │ImageSource │  │  │ViewImage    │ │   │
│  │  │ Gallery    │  │  │Storage     │  │  │NavigateGallery│  │
│  │  │ Scale      │  │  │UiPort      │  │  │ManageConfig │ │   │
│  │  │ Position   │  │  │ClipboardPort│ │  │OASImageViewerSvc│  │
│  │  │ ViewState │  │  │FileDialogPort│ │  └─────────────┘ │   │
│  │  └────────────┘  │  └────────────┘  │                   │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                   Infrastructure (基础设施层)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐      │
│  │FsImageSource │  │ JsonStorage  │  │  RfdFileDialog   │      │
│  │ (实现 ports)  │  │ (实现 ports)  │  │   (实现 ports)    │      │
│  └──────────────┘  └──────────────┘  └──────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                      UI 组件 (根目录)                           │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐     │
│   │  clipboard/  │  │ info_panel.rs│  │ shortcuts_help.rs│     │
│   └──────────────┘  └──────────────┘  └──────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

### 1. Core 层（核心层）

**文件位置**: `src/core/`

```
src/core/
├── mod.rs           # 模块入口、Result 类型、CoreError
├── domain/          # 领域实体和值对象
│   ├── mod.rs
│   ├── image.rs     # Image, Gallery, ImageMetadata, ImageFormat
│   └── types.rs     # Scale, Position, Color, ViewMode, GalleryLayout
├── ports/           # 端口接口（traits）
│   └── mod.rs       # ImageSource, Storage, UiPort, ClipboardPort, FileDialogPort
└── use_cases/       # 业务用例
    ├── mod.rs       # 用例聚合导出
    ├── service.rs    # 应用服务入口（按职责拆分子模块）
    └── service/      # lifecycle/viewer/gallery/config/ui_state
```

**职责**:
- 定义核心业务实体和规则
- 定义端口接口（Ports）
- 实现业务用例（Use Cases）
- **零外部依赖，完全独立**

**核心实体**:

```rust
// domain/image.rs
pub struct Image {
    id: String,
    path: PathBuf,
    metadata: ImageMetadata,
}

pub struct Gallery {
    images: Vec<Image>,
    selected_index: Option<usize>,
}

// domain/types.rs
pub struct Scale { value: f32 }
pub struct Position { x: f32, y: f32 }
pub enum ViewMode { Gallery, Viewer }
```

**端口接口**:

```rust
// ports/mod.rs
pub trait ImageSource: Send + Sync {
    fn load_metadata(&self, path: &Path) -> Result<ImageMetadata>;
    fn load_image_data(&self, path: &Path) -> Result<(u32, u32, Vec<u8>)>;
    fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>>;
}

pub trait Storage: Send + Sync {
    fn load_config(&self) -> Result<AppConfig>;
    fn save_config(&self, config: &AppConfig) -> Result<()>;
}
```

**用例**:

```rust
// use_cases/mod.rs
pub struct ViewImageUseCase {
    image_source: Arc<dyn ImageSource>,
}

impl ViewImageUseCase {
    pub fn new(image_source: Arc<dyn ImageSource>, _storage: Arc<dyn Storage>) -> Self;
    pub fn open_image(&self, path: &Path, state: &mut ViewState, ...) -> Result<()>;
    pub fn zoom(&self, state: &mut ViewState, factor: f32, ...);
    pub fn reset_zoom(&self, state: &mut ViewState);
}

// use_cases/service.rs
pub struct OASImageViewerService {
    view_use_case: ViewImageUseCase,
    navigate_use_case: NavigateGalleryUseCase,
    config_use_case: ManageConfigUseCase,
    state: Mutex<AppState>,
}

// use_cases/service/
// - lifecycle.rs  (初始化/状态入口)
// - viewer.rs     (查看器行为与查询)
// - gallery.rs    (图库行为与查询)
// - config.rs     (配置读写与持久化)
// - ui_state.rs   (UI 辅助状态)
```

### 2. Infrastructure 层（基础设施层）

**文件位置**: `src/infrastructure/`

**职责**:
- 实现 Core 层定义的 Ports
- 封装外部服务访问
- 提供技术能力支持

**实现**:

```rust
// infrastructure/mod.rs
mod fs_image_source;
mod storage;
mod async_image_source;
mod file_dialog;

pub use fs_image_source::FsImageSource; // ImageSource
pub use storage::JsonStorage;           // Storage
pub use file_dialog::RfdFileDialog;     // FileDialogPort
```

### 3. Adapters 层（适配器层）

**文件位置**: `src/adapters/egui/`

**职责**:
- 将 egui 事件转换为 Core 用例调用
- 将 Core 状态转换为 egui 显示
- 隔离 UI 框架细节

```
src/adapters/
├── mod.rs
├── egui/
│   ├── app.rs
│   ├── app/               # copy_shortcuts/handlers/lifecycle/menu(menu_file/menu_view/menu_image/menu_help/popup/sections)/render/shortcuts/state_sync/types/utils
│   ├── thumbnail_loader.rs
│   └── widgets/
└── platform/              # linux/macos/windows 平台集成
```

**核心适配器**:

```rust
// adapters/egui/app.rs
pub struct EguiApp {
    service: Arc<OASImageViewerService>,  // Core 服务
    viewer_widget: ViewerWidget,
    gallery_widget: GalleryWidget,
    // ...
}

impl EguiApp {
    pub fn new(cc: &CreationContext, service: Arc<OASImageViewerService>) -> Self;
    fn handle_open_dialog(&mut self);        // 调用 FileDialogPort
    fn process_pending_files(&mut self);     // 调用 ViewImageUseCase
}
```

### 4. UI 组件（根目录）

**文件位置**: `src/`

```
src/
├── adapters/
│   ├── clipboard.rs
│   └── egui/
│       ├── info_panel.rs
│       ├── info_panel/    # helpers/metadata/receiver/tests
│       └── shortcuts_help.rs
└── utils/
    ├── mod.rs
    └── threading.rs
```

---

## 模块详细说明

### 模块职责矩阵

| 模块 | 所属层 | 主要职责 |
|------|--------|----------|
| `core/domain` | Core | 实体、值对象、错误类型 |
| `core/ports` | Core | 端口接口定义 |
| `core/use_cases` | Core | 业务用例、状态管理 |
| `infrastructure` | Infrastructure | Ports 实现、配置持久化 |
| `adapters/egui` | Adapters | UI 适配 |
| `clipboard` | Infrastructure | 剪贴板 |
| `utils` | Infrastructure | 工具函数 |

> **总计**: 250+ 单元测试，Core 层覆盖率 >80%

### 关键数据结构

```rust
// 应用状态
pub struct AppState {
    pub view: ViewState,       // 查看器状态
    pub gallery: GalleryState, // 画廊状态
    pub config: AppConfig,     // 配置
}

pub struct ViewState {
    pub current_image: Option<Image>,
    pub scale: Scale,
    pub offset: Position,
    pub view_mode: ViewMode,
}

pub struct GalleryState {
    pub gallery: Gallery,
    pub layout: GalleryLayout,
}
```

---

## 模块依赖图

### 完整依赖图

```
                          ┌─────────────────┐
                          │    main.rs      │
                          │   (应用入口)    │
                          └────────┬────────┘
                                   │ 创建所有依赖
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Infrastructure                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐       │
│  │FsImageSource │  │ JsonStorage  │  │  RfdFileDialog   │       │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘       │
└─────────┼─────────────────┼───────────────────┼─────────────────┘
          │                 │                   │
          │    实现 Ports    │                   │
          ▼                 ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Core                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ ports/         ◄─────── 定义接口 ────────►  use_cases/  │   │
│  │ ImageSource              │              ViewImageUseCase │   │
│  │ Storage                  │              NavigateGallery  │   │
│  │ UiPort                   │              OASImageViewerService│  │
│  └──────────────────────────┼───────────────────────────────┘   │
│                             │                                   │
│                             ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      domain/                            │   │
│  │  Image, Gallery, Scale, Position, ViewMode, ...        │   │
│  └─────────────────────────────────────────────────────────┘   │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Adapters/Egui                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐       │
│  │   EguiApp    │  │ ViewerWidget │  │  GalleryWidget   │       │
│  │              │  │              │  │                  │       │
│  └──────────────┘  └──────────────┘  └──────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### 依赖关系表

| 模块 | 依赖的模块 | 被依赖的模块 |
|------|-----------|-------------|
| `main.rs` | core, infrastructure, adapters | - |
| `core/domain` | - | core/ports, core/use_cases |
| `core/ports` | core/domain | core/use_cases, infrastructure |
| `core/use_cases` | core/domain, core/ports | adapters |
| `infrastructure` | core/ports | main.rs |
| `adapters/egui` | core | main.rs |

---

## 数据流向

### 1. 打开图像流程

```
用户输入 (Ctrl+O / 拖放)
    │
    ▼
EguiApp::handle_open_dialog()
    │
    ├──► RfdFileDialog::open_files()
    │
    ▼
EguiApp::process_pending_files()
    │
    ├──► OASImageViewerService::update_state()
    │    │
    │    └──► ViewImageUseCase::open_image()
    │         │
    │         ├──► ImageSource::load_metadata()
    │         └──► ImageSource::load_image_data()
    │
    ▼
egui::Context::load_texture()
    │
    ▼
UI 渲染更新
```

### 2. 缩略图加载流程

```
GalleryWidget::ui()
    │
    ├──► 检查缩略图状态
    │
    ├──► 未加载 → ThumbnailLoader::request()
    │              │
    │              ▼
    │         后台线程
    │         ├──► ImageSource::generate_thumbnail()
    │         └──► channel 发送结果
    │
    └──► ThumbnailLoader::process_results()
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
EguiApp::update()
    │
    ├──► handle_shortcuts()
    │    ├──► 导航控制 (prev/next)
    │    ├──► 视图切换 (Gallery/Viewer)
    │    └──► 功能调用 (zoom, fullscreen)
    │
    ├──► viewer_widget.ui() / gallery_widget.ui()
    │
    └──► 渲染输出
```

---

## 商业化架构扩展

### 功能分层架构

基于业务需求，架构支持三层功能模型：

```
┌─────────────────────────────────────────────────────────────────┐
│                    功能分层架构                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  🆓 Tier 1: 基础版（免费，无需注册）                            │
│  ├─ 所有基础看图功能                                            │
│  ├─ PNG/JPEG/GIF/WebP/BMP/TIFF 支持                            │
│  └─ 缩放、平移、全屏                                            │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  📧 Tier 2: 高级版（免费，需注册）                              │
│  ├─ RAW 格式支持                                                │
│  ├─ 批量处理                                                    │
│  ├─ EXIF 显示                                                   │
│  └─ 需要 FeatureFlags::advanced_features()                     │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  💎 Tier 3: 增值版（付费）                                      │
│  ├─ 配置云同步                                                  │
│  ├─ 优先技术支持                                                │
│  └─ 需要 License::tier() >= LicenseTier::Pro                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 需要扩展的模块

| 模块 | 位置 | 说明 |
|------|------|------|
| `FeatureFlags` | `core/domain/` | 功能开关值对象 |
| `License` | `core/domain/` | 许可证实体 |
| `User` | `core/domain/` | 用户实体 |
| `AuthPort` | `core/ports/` | 认证端口 |
| `AuthService` | `core/use_cases/` | 认证用例 |
| `CloudSyncPort` | `core/ports/` | 云同步端口（规划中） |

### 规划项与现状映射

| 设计项 | 文档状态 | 当前落地方式 | 说明 |
|------|------|------|------|
| `DomainEvent` | 历史方案 | `OASImageViewerService::read_state/update_state` + 用例编排 | 当前采用同步状态驱动而非事件溯源 |
| `ImageCommand` | 规划中 | `ImageTransform` + `EditImageUseCase` + `BatchUseCase` | 当前已覆盖导出/批处理基础能力 |
| `CommandHistory` | 规划中 | 暂无 | 建议在 v0.7.0 撤销/重做需求明确后引入 |
| `FeatureFlags` / `License` | 规划中 | 暂无 | 商业化分层后续按产品验证推进 |

### 功能开关设计

```rust
// core/domain/feature_flags.rs (待实现)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeatureFlags {
    pub advanced_features: bool,  // Tier 2
    pub cloud_sync: bool,         // Tier 3
    pub priority_support: bool,   // Tier 3
}

impl FeatureFlags {
    pub fn free() -> Self { ... }
    pub fn registered() -> Self { ... }
    pub fn pro() -> Self { ... }
}
```

### 许可证验证设计

```rust
// core/domain/license.rs (待实现)
#[derive(Debug, Clone)]
pub struct License {
    pub tier: LicenseTier,
    pub expires: Option<DateTime<Utc>>,
    pub signature: String,
}

pub enum LicenseTier {
    Free,
    Pro,
    Enterprise,
}
```

### 图像编辑扩展 (v0.7.0)

```rust
// core/domain/command.rs (规划中，当前未实现)
pub trait ImageCommand {
    fn execute(&self, image: &mut Image) -> Result<()>;
    fn undo(&self, image: &mut Image) -> Result<()>;
    fn description(&self) -> &str;
}

pub struct RotateCommand { degrees: f32 }
pub struct CropCommand { rect: Rect }
pub struct FlipCommand { horizontal: bool }

// 命令历史管理
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn ImageCommand>>,
    redo_stack: Vec<Box<dyn ImageCommand>>,
}
```

当前代码中的编辑链路为：

- 变换语义：`core/domain/editing.rs::ImageTransform`
- 编辑用例：`core/use_cases/edit.rs::EditImageUseCase`
- 批处理用例：`core/use_cases/batch.rs::BatchUseCase`
- 服务聚合入口：`core/use_cases/service.rs::OASImageViewerService`

### v0.3.5 ~ v0.7.0 版本适配评估

| 版本 | 目标能力 | 当前架构适配度 | 主要缺口 |
|------|----------|----------------|----------|
| v0.3.5 | 背景色切换、适应宽/高 | 高 | 仅需补充缩放策略函数与快捷键映射 |
| v0.4.0 | 幻灯片、EXIF完整、旋转翻转只读 | 中高 | 缺统一播放状态机；旋转翻转目前无独立只读变换层 |
| v0.5.0 | 可保存编辑、格式互转、批量重命名 | 中 | 缺编辑命令流水线、导出端口、批处理任务编排 |
| v0.6.0 | RAW 解码 | 中低 | 缺解码抽象层与多后端策略（当前 ImageSource 偏单路径） |
| v0.7.0 | 性能优化 | 中 | 缺系统化性能观测指标与任务调度分级 |

建议演进顺序：

1. 在 `core/domain` 增加只读变换模型（旋转/翻转）与查看状态解耦；
2. 在 `core/ports` 增加导出端口（格式转换/质量参数）和批处理端口；
3. 在 `infrastructure` 引入解码后端抽象（标准格式与 RAW 分离）；
4. 在 `adapters/egui` 增加任务进度与取消机制，支撑批处理与性能优化闭环。

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
    │    (70%)     │  250+ 测试用例
     └─────────────┘
```

### 单元测试分布

| 模块 | 测试方式 | 覆盖率重点 |
|------|---------|-----------|
| core/domain | 内嵌 tests | 实体行为、值对象验证 |
| core/ports | 内嵌 tests | 接口契约 |
| core/use_cases | 内嵌 tests | 业务逻辑、状态转换 |
| infrastructure | 内嵌 tests | Ports 实现、错误处理 |
| adapters/egui | 内嵌 tests | 事件处理、状态同步 |

> **总计**: 250+ 单元测试

### Core 层测试优势

由于 Core 层零外部依赖，可以：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // 1. 无需 mock 框架，直接实现 trait
    struct MockImageSource;
    impl ImageSource for MockImageSource {
        fn load_metadata(&self, _: &Path) -> Result<ImageMetadata> {
            Ok(ImageMetadata::default())
        }
        // ...
    }
    
    // 2. 纯单元测试，无需启动 UI
    #[test]
    fn test_view_image_use_case() {
        let source = Arc::new(MockImageSource);
        let use_case = ViewImageUseCase::new(source, ...);
        
        let mut state = ViewState::default();
        use_case.open_image(Path::new("test.png"), &mut state).unwrap();
        
        assert!(state.current_image.is_some());
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --lib core::domain::tests
cargo test --lib core::use_cases::tests

# 运行集成测试
cargo test --test decoder_test
cargo test --test dnd_integration_test

# 生成覆盖率报告
cargo tarpaulin --out Html
```

---

## 总结

OAS-Image-Viewer v0.4.0 通过 Clean Architecture 实现了：

1. **清晰的代码组织**：Core / Infrastructure / Adapters 三层架构
2. **高可测试性**：250+ 单元测试，Core 层可纯单元测试
3. **技术无关性**：业务逻辑独立于 UI 框架
4. **易于扩展**：新功能通过添加 UseCase/Port 实现
5. **商业化就绪**：架构支持功能分层、账号系统、云同步等商业化需求
6. **易于维护**：依赖关系清晰，修改影响范围可控

这种架构设计确保了项目的长期健康发展，为后续功能扩展和商业化奠定了坚实基础。
