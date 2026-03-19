# 图片查看器 UI/UX 规格文档

> 基于 Rust + egui 的跨平台图片查看器界面规范

## 1. 界面结构

### 1.1 整体布局

```
┌─────────────────────────────────────────────────────────────┐
│  标题栏 (Title Bar)                                          │
│  [图标] 图片查看器                    [最小化] [最大化] [关闭] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                      主区域 (Main Area)                      │
│                                                             │
│              ┌───────────────────────────┐                  │
│              │                           │                  │
│              │      Gallery / Viewer     │                  │
│              │                           │                  │
│              └───────────────────────────┘                  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  状态栏 (Status Bar)                                         │
│  [路径] /home/user/images    [2/50]    [100%]    [1280x720] │
└─────────────────────────────────────────────────────────────┘
```

**布局参数：**
- 标题栏高度：32px
- 状态栏高度：24px
- 主区域：自适应剩余空间
- 最小窗口尺寸：400x300px

### 1.2 Gallery 视图布局

Gallery 模式用于浏览文件夹中的图片缩略图。

```
┌─────────────────────────────────────────────────────────────┐
│  [🖼️] Gallery - /home/user/images                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐           │
│  │ 📷  │ │ 📷  │ │ 📷  │ │ 📷  │ │ 📷  │ │ 📷  │           │
│  │img1 │ │img2 │ │img3 │ │img4 │ │img5 │ │img6 │           │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘           │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐           │
│  │ 📷  │ │ 📷  │ │ 📷  │ │ 📷  │ │ 📷  │ │ 📷  │           │
│  │img7 │ │img8 │ │ ✅  │ │     │ │     │ │     │           │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘           │
│  ...                                                        │
├─────────────────────────────────────────────────────────────┤
│  📁 /home/user/images                    [9 items] [选中 3]  │
└─────────────────────────────────────────────────────────────┘
```

**缩略图网格规格：**
- 缩略图尺寸：120x120px（可配置：60-200px）
- 网格间距：12px
- 边框圆角：4px
- 选中态：2px 蓝色边框 (#3498db) + 轻微阴影

**滚动行为：**
- 垂直滚动：鼠标滚轮 / 触摸板
- 平滑滚动：启用 egui 的平滑滚动动画
- 滚动条：自定义样式，宽度 8px

**选中态样式：**
```
┌──────────────┐
│              │
│   [图片]     │ ← 2px 蓝色边框 (#3498db)
│              │    box-shadow: 0 0 8px rgba(52, 152, 219, 0.5)
│  filename    │
└──────────────┘
```

### 1.3 Viewer 视图布局

Viewer 模式用于单张图片的详细查看。

```
┌─────────────────────────────────────────────────────────────┐
│  [←] [→]  Viewer - image_003.jpg                     [🔧]  │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────┬─────────────────────────────┐ │
│  │                          │  📋 信息面板                 │ │
│  │                          ├─────────────────────────────┤ │
│  │                          │  文件名: image_003.jpg      │ │
│  │                          │  格式: JPEG                 │ │
│  │      [图片显示区域]       │  尺寸: 3840 x 2160          │ │
│  │                          │  大小: 2.4 MB               │ │
│  │                          │  拍摄时间: 2026-01-15       │ │
│  │                          │  相机: Canon EOS R5         │ │
│  │                          │  ...                        │ │
│  │                          │                             │ │
│  │                          │  🏷️ 标签                     │ │
│  │                          │  [风景] [旅行] [2026]       │ │
│  └──────────────────────────┴─────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  [⬅️] [➡️]  [🔍-] [100%] [🔍+]  [↔️适应] [🖥️原始]  [🔄旋转]  │
└─────────────────────────────────────────────────────────────┘
```

**工具栏按钮（从左到右）：**
1. ⬅️ 上一张图片
2. ➡️ 下一张图片
3. 🔍- 缩小
4. 当前缩放比例显示（可点击重置）
5. 🔍+ 放大
6. ↔️ 适应窗口
7. 🖥️ 原始大小
8. 🔄 顺时针旋转90°
9. ⛶ 全屏切换

**信息面板：**
- 宽度：固定 280px（可折叠）
- 背景：略深于主背景
- 分组：文件信息 / EXIF / 标签

## 2. 交互流程

### 2.1 启动流程

```
┌─────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   启动应用   │────▶│  加载配置/主题   │────▶│   显示主窗口    │
└─────────────┘     └─────────────────┘     └────────┬────────┘
                                                     │
                    ┌───────────────────────────────┘
                    ▼
        ┌───────────────────────┐
        │  检查启动参数/最近文件  │
        └───────────┬───────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
        ▼                       ▼
┌───────────────┐      ┌────────────────┐
│ 有文件路径参数 │      │  无参数/新启动  │
│  或最近文件    │      │                │
└───────┬───────┘      └───────┬────────┘
        │                      │
        ▼                      ▼
┌───────────────┐      ┌────────────────┐
│  加载文件夹    │      │   显示空态     │
│  进入Gallery  │      │  "打开文件"    │
└───────────────┘      └────────────────┘
```

### 2.2 打开文件流程

```
用户操作: Ctrl+O / 菜单点击
         │
         ▼
┌───────────────────┐
│   显示文件选择器   │
└─────────┬─────────┘
          │
    ┌─────┴─────┐
    │           │
    ▼           ▼
┌───────┐  ┌──────────┐
│ 选择  │  │  取消    │
│ 文件  │  │          │
└───┬───┘  └──────────┘
    │
    ▼
┌───────────────────┐
│  判断文件类型      │
└─────────┬─────────┘
          │
    ┌─────┴─────┐
    │           │
    ▼           ▼
┌───────┐  ┌──────────┐
│ 单文件 │  │  文件夹  │
└───┬───┘  └────┬─────┘
    │           │
    ▼           ▼
┌────────┐  ┌──────────┐
│进入    │  │加载目录  │
│Viewer  │  │进入Gallery│
└────────┘  └──────────┘
```

### 2.3 状态转换图

```
                    ┌─────────────────────────────────────────┐
                    │                                         │
                    ▼                                         │
┌────────┐    ┌─────────┐    ┌──────────┐    ┌────────┐      │
│  启动   │───▶│  空态   │───▶│ Gallery  │◀──▶│ Viewer │──────┘
└────────┘    └─────────┘    └──────────┘    └────────┘
                  │                ▲              │
                  │                │              │
                  │           ┌────┴────┐         │
                  │           │  返回   │◀────────┘
                  │           │  Gallery│
                  │           └─────────┘
                  │
                  ▼
            ┌──────────┐
            │  退出    │
            └──────────┘

状态转换触发条件：
────────────────────────────────────────
空态 → Gallery:  打开文件夹 / 拖拽文件夹到窗口
空态 → Viewer:   打开单个文件 / 拖拽文件到窗口
Gallery → Viewer: 双击图片 / 按 Enter / 空格键
Viewer → Gallery: 按 Esc / 点击返回按钮 / 按 Backspace
Viewer → Viewer:  左右箭头切换图片 / 鼠标滚轮
Gallery → Gallery: 方向键导航 / 鼠标点击选中
任意 → 空态:      关闭所有文件
```

## 3. 组件状态定义

### 3.1 Gallery 状态

```rust
enum GalleryState {
    /// 空态：无文件夹打开
    Empty {
        /// 提示信息
        message: &'static str,  // "拖放文件夹到此处或按 Ctrl+O 打开"
        /// 快捷操作按钮
        actions: Vec<Action>,   // [打开文件夹, 打开文件]
    },
    
    /// 加载中：正在扫描/加载缩略图
    Loading {
        /// 已加载数量
        loaded: usize,
        /// 总数量
        total: usize,
        /// 当前处理路径
        current_path: Option<PathBuf>,
    },
    
    /// 有内容：正常显示缩略图网格
    Content {
        /// 图片列表
        items: Vec<GalleryItem>,
        /// 当前选中索引
        selected_index: Option<usize>,
        /// 视图偏移（滚动位置）
        scroll_offset: Vec2,
        /// 缩略图大小
        thumbnail_size: f32,
    },
    
    /// 选中：用户已选择图片（准备打开或预览）
    Selection {
        /// 选中的项目
        selected: Vec<usize>,
        /// 最后选中（焦点）
        last_selected: usize,
        /// 是否支持多选
        multi_select: bool,
    },
    
    /// 错误：加载失败
    Error {
        /// 错误信息
        message: String,
        /// 是否可重试
        retryable: bool,
    },
}
```

**状态可视化：**

| 状态 | 视觉表现 | 用户交互 |
|------|----------|----------|
| Empty | 中央显示大图标 + 提示文字 | 点击打开按钮、拖拽文件 |
| Loading | 进度条 + "加载中..."文字 | 可取消、显示实时进度 |
| Content | 缩略图网格 | 点击选中、双击打开、右键菜单 |
| Selection | 高亮选中项、底部显示操作栏 | Enter打开、Delete删除 |
| Error | 错误图标 + 错误信息 + 重试按钮 | 点击重试、返回 |

### 3.2 Viewer 状态

```rust
enum ViewerState {
    /// 加载中：正在解码图片
    Loading {
        /// 文件路径
        path: PathBuf,
        /// 加载开始时间
        started_at: Instant,
    },
    
    /// 显示中：图片正常显示
    Displaying {
        /// 图片纹理
        texture: TextureHandle,
        /// 原始尺寸
        original_size: Vec2,
        /// 当前变换
        transform: ViewTransform,
        /// 是否显示信息面板
        info_panel_visible: bool,
    },
    
    /// 缩放中：用户正在缩放/平移
    Zooming {
        /// 基础变换
        base_transform: ViewTransform,
        /// 缩放起点（鼠标位置）
        zoom_anchor: Option<Vec2>,
        /// 缩放比例
        scale: f32,
        /// 偏移量
        offset: Vec2,
    },
    
    /// 错误：无法加载或显示
    Error {
        /// 错误类型
        error: ViewerError,
        /// 文件路径
        path: PathBuf,
    },
}

struct ViewTransform {
    /// 缩放比例 (0.1 - 10.0)
    scale: f32,
    /// 平移偏移
    offset: Vec2,
    /// 旋转角度 (0, 90, 180, 270)
    rotation: u16,
}
```

**状态可视化：**

| 状态 | 视觉表现 | 用户交互 |
|------|----------|----------|
| Loading | 中央旋转进度指示器 | 可取消、显示文件名 |
| Displaying | 图片居中显示、工具栏可用 | 拖拽平移、滚轮缩放 |
| Zooming | 实时缩放预览、显示当前比例 | 释放鼠标确认、Esc取消 |
| Error | 错误图标 + 友好提示 + 操作选项 | 重试、打开其他、报告问题 |

## 4. 快捷键/手势映射表

### 4.1 全局快捷键

| 快捷键 | 功能 | 作用范围 |
|--------|------|----------|
| `Ctrl + O` | 打开文件/文件夹 | 全局 |
| `Ctrl + Shift + O` | 打开文件夹 | 全局 |
| `Ctrl + W` | 关闭当前文件/返回 | 全局 |
| `Ctrl + Q` | 退出应用 | 全局 |
| `F11` | 全屏切换 | 全局 |
| `Ctrl + ,` | 打开设置 | 全局 |
| `F1` | 帮助/快捷键列表 | 全局 |

### 4.2 Gallery 视图快捷键

| 快捷键 | 功能 | 说明 |
|--------|------|------|
| `↑ ↓ ← →` | 导航选择 | 方向键移动选择 |
| `Enter` / `Space` | 打开选中图片 | 进入 Viewer |
| `Ctrl + A` | 全选 | 选中所有图片 |
| `Ctrl + Click` | 多选 | 添加/移除单个选择 |
| `Shift + Click` | 范围选择 | 从最后选中到当前 |
| `Delete` | 删除选中 | 移入回收站/确认对话框 |
| `+` / `-` | 调整缩略图大小 | 放大/缩小缩略图 |
| `Ctrl + 0` | 重置缩略图大小 | 恢复默认 120px |
| `Ctrl + R` | 刷新文件夹 | 重新扫描 |

### 4.3 Viewer 视图快捷键

| 快捷键 | 功能 | 说明 |
|--------|------|------|
| `←` | 上一张 | 切换到上一张图片 |
| `→` | 下一张 | 切换到下一张图片 |
| `Esc` | 返回 Gallery / 退出全屏 | 查看器退回图库或退出全屏 |
| `Cmd/Ctrl + +` / `Cmd/Ctrl + -` | 放大/缩小 | 查看器缩放 |
| `Cmd/Ctrl + 0` | 适应窗口 | 图片适应窗口 |
| `Cmd/Ctrl + 1` | 原始大小 | 1:1 显示 |
| `Cmd/Ctrl + 2` / `Cmd/Ctrl + 3` | 适应宽度/高度 | 查看器快速适配 |
| `R` / `Shift + R` | 顺/逆时针旋转 90° | 只读变换（不落盘） |
| `H` / `V` | 水平/垂直翻转 | 只读变换（不落盘） |
| `F` | 切换信息面板 | 显示/隐藏右侧信息 |
| `B` | 背景色循环 | 黑/灰/白三态切换 |
| `S` | 幻灯片播放/暂停 | 支持间隔与末尾行为配置 |

### 4.4 鼠标/触摸手势

| 手势 | 功能 | 上下文 |
|------|------|--------|
| `滚轮` | 垂直滚动 (Gallery) / 缩放 (Viewer) | 通用 |
| `滚轮 + Shift` | 水平滚动 | Gallery |
| `滚轮 + Ctrl` | 缩放 | Gallery |
| `拖拽` | 平移图片 | Viewer |
| `双击` | 打开图片 | Gallery |
| `双击` | 适应窗口 ↔ 实际大小 | Viewer |
| `右键` | 上下文菜单 | 通用 |
| `中键拖拽` | 快速平移 | Viewer |
| `触摸: 捏合` | 缩放 | Viewer |
| `触摸: 滑动` | 切换图片 | Viewer |
| `触摸: 长按` | 上下文菜单 | 通用 |

## 5. 视觉规范

### 5.1 颜色系统

基于 egui 默认风格，进行以下定制：

```rust
// 主色调
const PRIMARY: Color32 = Color32::from_rgb(52, 152, 219);    // #3498db - 蓝色
const PRIMARY_HOVER: Color32 = Color32::from_rgb(41, 128, 185); // #2980b9
const PRIMARY_ACTIVE: Color32 = Color32::from_rgb(31, 97, 141); // #1f618d

// 背景色
const BG_PRIMARY: Color32 = Color32::from_gray(30);          // 主背景
const BG_SECONDARY: Color32 = Color32::from_gray(40);        // 次级背景
const BG_TERTIARY: Color32 = Color32::from_gray(50);         // 卡片/面板背景

// 文字颜色
const TEXT_PRIMARY: Color32 = Color32::from_gray(240);       // 主要文字
const TEXT_SECONDARY: Color32 = Color32::from_gray(180);     // 次要文字
const TEXT_MUTED: Color32 = Color32::from_gray(120);         // 弱化文字

// 功能色
const SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);    // 成功
const WARNING: Color32 = Color32::from_rgb(241, 196, 15);    // 警告
const ERROR: Color32 = Color32::from_rgb(231, 76, 60);       // 错误
const INFO: Color32 = Color32::from_rgb(52, 152, 219);       // 信息
```

### 5.2 间距系统

```rust
// 基础间距单位 = 4px
const SPACING_XS: f32 = 4.0;   // 极小间距
const SPACING_SM: f32 = 8.0;   // 小间距
const SPACING_MD: f32 = 12.0;  // 中间距 (默认)
const SPACING_LG: f32 = 16.0;  // 大间距
const SPACING_XL: f32 = 24.0;  // 极大间距
const SPACING_XXL: f32 = 32.0; // 超大间距

// 组件间距
const BUTTON_PADDING: Vec2 = vec2(12.0, 8.0);
const CARD_PADDING: f32 = 16.0;
const PANEL_PADDING: f32 = 16.0;
const GRID_GAP: f32 = 12.0;
```

### 5.3 字体规范

```rust
// 字体大小
const FONT_SIZE_XS: f32 = 11.0;   // 极小：标签、辅助文字
const FONT_SIZE_SM: f32 = 12.0;   // 小：状态栏、元信息
const FONT_SIZE_MD: f32 = 14.0;   // 中：正文 (默认)
const FONT_SIZE_LG: f32 = 16.0;   // 大：按钮、标题
const FONT_SIZE_XL: f32 = 20.0;   // 极大：面板标题
const FONT_SIZE_XXL: f32 = 24.0;  // 超大：空态标题

// 字重
const FONT_WEIGHT_NORMAL: f32 = 400.0;
const FONT_WEIGHT_MEDIUM: f32 = 500.0;
const FONT_WEIGHT_BOLD: f32 = 700.0;

// 字体族（使用系统默认）
// Windows: Segoe UI
// macOS: San Francisco
// Linux: Cantarell / Ubuntu
```

### 5.4 圆角与阴影

```rust
// 圆角半径
const RADIUS_SM: f32 = 2.0;
const RADIUS_MD: f32 = 4.0;    // 默认
const RADIUS_LG: f32 = 8.0;
const RADIUS_FULL: f32 = 9999.0; // 圆形

// 阴影
const SHADOW_SM: Shadow = Shadow::small();
const SHADOW_MD: Shadow = Shadow::medium();  // 默认
const SHADOW_LG: Shadow = Shadow::big();
```

### 5.5 组件样式

**按钮：**
```rust
Button::new("打开")
    .fill(PRIMARY)
    .stroke(Stroke::NONE)
    .corner_radius(RADIUS_MD)
    .padding(BUTTON_PADDING)
```

**缩略图卡片：**
```rust
// 默认态
Frame::default()
    .fill(BG_TERTIARY)
    .corner_radius(RADIUS_MD)
    .inner_margin(4.0)

// 选中态
Frame::default()
    .fill(BG_TERTIARY)
    .stroke(Stroke::new(2.0, PRIMARY))
    .corner_radius(RADIUS_MD)
    .shadow(SHADOW_MD)
```

**信息面板：**
```rust
SidePanel::right("info_panel")
    .resizable(true)
    .default_width(280.0)
    .frame(Frame::default()
        .fill(BG_SECONDARY)
        .stroke(Stroke::new(1.0, BG_TERTIARY)))
```

## 6. 响应式适配

### 6.1 窗口缩放行为

**最小窗口尺寸：** 400x300px

**断点定义：**
```rust
enum WindowSize {
    Compact,   // < 600px 宽度
    Medium,    // 600px - 1024px
    Large,     // 1024px - 1440px
    XLarge,    // > 1440px
}
```

### 6.2 Gallery 响应式规则

| 窗口宽度 | 缩略图尺寸 | 每行数量 | 网格间距 |
|----------|------------|----------|----------|
| < 600px  | 自适应     | 3-4      | 8px      |
| 600-1024 | 100px      | 自适应   | 10px     |
| 1024-1440| 120px      | 自适应   | 12px     |
| > 1440px | 140px      | 自适应   | 16px     |

**代码实现逻辑：**
```rust
fn calculate_grid_layout(available_width: f32) -> GridLayout {
    let min_item_width = match window_size {
        WindowSize::Compact => 80.0,
        WindowSize::Medium => 100.0,
        WindowSize::Large => 120.0,
        WindowSize::XLarge => 140.0,
    };
    
    let gap = 12.0;
    let columns = ((available_width + gap) / (min_item_width + gap)) as usize;
    let columns = columns.max(1);
    
    let item_width = (available_width - gap * (columns - 1) as f32) / columns as f32;
    
    GridLayout { columns, item_width, gap }
}
```

### 6.3 Viewer 响应式规则

| 窗口宽度 | 信息面板 | 工具栏布局 |
|----------|----------|------------|
| < 600px  | 隐藏（可侧滑） | 简化图标 |
| 600-900  | 可折叠 | 图标+文字 |
| > 900px  | 固定显示 | 完整布局 |

**Viewer 缩放行为：**
- 图片始终保持在可视区域内
- 缩放时以鼠标位置为中心（或窗口中心）
- 窗口大小改变时保持相对缩放比例

### 6.4 字体缩放支持

- 遵循系统字体缩放设置
- 所有尺寸使用相对单位
- 最小保证可读性（不小于 9px）

### 6.5 高分屏适配

```rust
// 自动检测缩放比例
let scale_factor = ctx.pixels_per_point();

// 根据 DPI 调整 UI 元素
let adjusted_padding = BASE_PADDING * scale_factor;
let adjusted_font_size = BASE_FONT_SIZE * scale_factor;
```

---

## 附录

### A. 图标列表

| 图标 | 用途 | Unicode |
|------|------|---------|
| 🖼️ | Gallery 模式 | U+1F5BC |
| 🔍 | 搜索/缩放 | U+1F50D |
| ⬅️ | 上一张 | U+2B05 |
| ➡️ | 下一张 | U+27A1 |
| ↕️ | 适应窗口 | U+2195 |
| 🔄 | 旋转 | U+1F504 |
| ⛶ | 全屏 | U+26F6 |
| 📋 | 信息面板 | U+1F4CB |
| 🏷️ | 标签 | U+1F3F7 |
| 📁 | 文件夹 | U+1F4C1 |
| ⚙️ | 设置 | U+2699 |
| ❌ | 关闭/错误 | U+274C |
| ✅ | 成功/选中 | U+2705 |

### B. 文件格式支持

| 格式 | 缩略图 | 查看 | 元信息 |
|------|--------|------|--------|
| JPEG | ✓ | ✓ | EXIF |
| PNG | ✓ | ✓ | 基础 |
| GIF | ✓ | ✓ | 基础 |
| WebP | ✓ | ✓ | 基础 |
| BMP | ✓ | ✓ | 基础 |
| TIFF | ✓ | ✓ | EXIF |
| HEIC/HEIF | ✓ | ✓ | EXIF |
| AVIF | ✓ | ✓ | 基础 |
| RAW | ✗ | ✓ | EXIF |
| SVG | ✓ | ✓ | 基础 |

---

*文档版本: 1.0*  
*最后更新: 2026-02-28*  
*适用项目: oas-image-viewer (Rust + egui)*
