# 使用文档

本文档详细介绍 Image Viewer 的使用方法。

## 目录

- [快速开始](#快速开始)
- [详细功能说明](#详细功能说明)
- [常见问题](#常见问题)
- [故障排除](#故障排除)

---

## 快速开始

### 安装

#### 方式一：下载预编译二进制文件

1. 访问 [Releases](https://github.com/openappsys/oas-image-viewer/releases) 页面
2. 下载适合您系统的版本
3. 解压并运行

#### 方式二：从源码构建

```bash
# 1. 确保已安装 Rust 1.94+
rustc --version

# 2. 克隆仓库
git clone https://github.com/openappsys/oas-image-viewer.git
cd oas-image-viewer

# 3. 构建
cargo build --release

# 4. 运行
./target/release/oas-image-viewer
```

### 首次运行

```bash
# 直接打开
oas-image-viewer

# 打开指定图片
oas-image-viewer photo.png

# 打开文件夹
oas-image-viewer ~/Pictures/
```

---

## 详细功能说明

### 1. 图片查看模式

#### 基本导航

| 操作 | 效果 |
|------|------|
| `←` / `→` | 上一张/下一张图片 |
| 鼠标滚轮 | 放大/缩小 |
| 拖拽 | 平移图片（放大后） |
| 双击 | 切换全屏 |
| 右键菜单 | 复制路径、在文件夹中显示 |

#### 缩放控制

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + +` | 放大 |
| `Ctrl + -` | 缩小 |
| `Ctrl + 0` | 适应窗口 |
| `Ctrl + 1` | 重置为原始大小 |

### 2. 画廊模式

#### 进入画廊

- 按 `G` 键切换画廊/查看器模式
- 或者在查看器中点击"画廊"按钮

#### 画廊操作

| 操作 | 效果 |
|------|------|
| 点击缩略图 | 在查看器中打开该图片 |
| 滚轮 | 滚动缩略图列表 |
| `Ctrl + 滚轮` | 调整缩略图大小 |
| `G` | 切换画廊/查看器模式 |

### 3. 文件操作

| 快捷键 | 功能 |
|--------|------|
| `Cmd/Ctrl + O` | 打开文件对话框 |
| `Cmd/Ctrl + Shift + O` | 打开文件夹 |
| `Cmd + C`（macOS）/ `Ctrl + C`（Windows/Linux） | 复制当前图片到剪贴板 |
| `Cmd + Shift + C`（macOS）/ `Ctrl + Shift + C`（Windows/Linux） | 复制当前图片路径 |
| `←` / `→` | 导航上一张/下一张 |
| `Esc` | 退出全屏/关闭浮层 |

### 4. 显示选项

| 快捷键 | 功能 |
|--------|------|
| `F11` | 切换全屏模式 |
| `F` | 切换信息面板 |
| `?` | 显示快捷键帮助 |
| `G` | 切换画廊模式 |
| `H` | 水平翻转 |
| `V` | 垂直翻转 |
| `R` | 顺时针旋转 90° |
| `Shift + R` | 逆时针旋转 90° |

### 5. 配置自定义

配置文件位置：

- **Linux**: `~/.config/oas-image-viewer/config.toml`
- **macOS**: `~/Library/Application Support/com.openappsys.oas-image-viewer/config.toml`
- **Windows**: `%APPDATA%\\openappsys\\oas-image-viewer\\config\\config.toml`

#### 完整配置示例

```toml
# 窗口设置
[window]
width = 1200.0          # 窗口宽度
height = 800.0          # 窗口高度
maximized = false       # 启动时最大化

# 画廊设置
[gallery]
thumbnail_size = 150            # 缩略图大小（像素）
items_per_row = 0               # 0 表示自动计算每行数量
grid_spacing = 12.0             # 缩略图间距
show_filenames = true           # 显示文件名

# 查看器设置
[viewer]
background_color = [30, 30, 30]     # 背景色 RGB
fit_to_window = true                # 默认适应窗口
show_info_panel = false             # 显示信息面板
smooth_scroll = true                # 平滑滚动
zoom_step = 1.25                    # 缩放步长（倍率）
min_scale = 0.1                     # 最小缩放比例
max_scale = 20.0                    # 最大缩放比例
```

---

## 常见问题

### Q: 支持哪些图片格式？

**A:** 当前支持以下格式：

- PNG（包括透明通道）
- JPEG/JPG
- GIF（静态和动画）
- WebP
- TIFF
- BMP

计划支持的格式：
- RAW（CR2, NEF, ARW 等）
- HEIC/HEIF
- SVG（基础支持）
- AVIF

### Q: 如何设置默认程序？

**A:**

#### Windows
1. 右键点击任意图片文件
2. 选择"打开方式" → "选择其他应用"
3. 找到 oas-image-viewer 并勾选"始终使用此应用打开"

#### macOS
1. 右键点击任意图片文件
2. 选择"显示简介"
3. 在"打开方式"下选择 Image Viewer
4. 点击"全部更改"

#### Linux (GNOME)
1. 右键点击任意图片文件
2. 选择"使用其他应用程序打开"
3. 选择 Image Viewer 并设为默认

### Q: 可以编辑图片吗？

**A:** 当前版本不提供图片编辑能力（暂不支持旋转、翻转、裁剪、亮度/对比度调整）。编辑能力仍在规划中：

- 基础旋转和翻转
- 裁剪
- 调整亮度/对比度

### Q: 如何清除最近打开的文件列表？

**A:** 当前版本没有独立的“最近文件列表”配置段。如需重置相关状态，可直接删除配置文件 `config.toml`。

### Q: 支持网络图片吗？

**A:** 当前版本仅支持本地文件。URL 支持已在路线图中。

---

## 故障排除

### 问题：程序无法启动

#### 症状
双击程序无反应，或命令行返回错误。

#### 解决方案

1. **检查 Rust 版本**
   ```bash
   rustc --version  # 需要 1.94+
   ```

2. **检查系统依赖**（Linux）
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
     libxcb-xfixes0-dev
   
   # 检查缺失的库
   ldd ./target/release/oas-image-viewer
   ```

3. **查看详细错误**
   ```bash
   RUST_LOG=debug ./target/release/oas-image-viewer
   ```

### 问题：无法打开某些图片

#### 症状
特定图片文件显示"无法加载"或空白。

#### 解决方案

1. **检查文件格式**
   ```bash
   file problem-image.png
   ```

2. **验证文件完整性**
   ```bash
   # 尝试用其他程序打开
   # 如果其他程序也无法打开，文件可能已损坏
   ```

3. **启用调试日志**
   ```bash
   RUST_LOG=image_viewer=debug oas-image-viewer problem-image.png
   ```

4. **检查文件权限**
   ```bash
   ls -la problem-image.png
   ```

### 问题：性能缓慢

#### 症状
打开大图片或包含大量图片的文件夹时卡顿。

#### 解决方案

1. **使用发布模式构建**
   ```bash
   cargo build --release
   ```

2. **减少缩略图大小**
   编辑配置文件：
   ```toml
   [gallery]
   thumbnail_size = 100  # 减小缩略图尺寸
   ```

3. **限制同时加载的图片数量**
   在 `config.toml` 中：
   ```toml
   [gallery]
   max_concurrent_loads = 4
   ```

4. **检查系统资源**
   ```bash
   # 查看内存使用
   free -h
   
   # 查看 CPU 使用
   htop
   ```

### 问题：界面显示异常

#### 症状
文字模糊、界面元素错位或黑屏。

#### 解决方案

1. **更新显卡驱动**

2. **禁用 GPU 加速**（如果需要）
   ```bash
   # Linux: 使用软件渲染
   LIBGL_ALWAYS_SOFTWARE=1 oas-image-viewer
   ```

3. **调整缩放设置**（HiDPI 显示器）
   ```bash
   # Linux
   export WINIT_X11_SCALE_FACTOR=1.5
   oas-image-viewer
   ```

### 问题：配置文件错误

#### 症状
启动时报配置解析错误。

#### 解决方案

1. **重置配置文件**
   ```bash
   # Linux
   mv ~/.config/oas-image-viewer/config.toml ~/.config/oas-image-viewer/config.toml.bak
   
   # macOS
   mv ~/Library/Application\ Support/com.openappsys.oas-image-viewer/config.toml \
      ~/Library/Application\ Support/com.openappsys.oas-image-viewer/config.toml.bak
   
   # Windows (PowerShell)
   Move-Item "$env:APPDATA\\oas-image-viewer\\config\\config.toml" `
             "$env:APPDATA\\oas-image-viewer\\config\\config.toml.bak"
   ```

2. **验证 TOML 语法**
   使用在线 TOML 验证器检查配置文件。

### 获取帮助

如果以上方案无法解决您的问题：

1. 查看 [Issues](https://github.com/openappsys/oas-image-viewer/issues) 是否已有类似报告
2. 开启调试模式收集日志：
   ```bash
   RUST_LOG=debug oas-image-viewer 2> log.txt
   ```
3. 创建新 Issue，附上：
   - 操作系统和版本
   - Image Viewer 版本
   - 问题描述
   - 复现步骤
   - 调试日志（如有）

---

## 快捷键速查表

### 导航
| 快捷键 | 功能 |
|--------|------|
| `←` / `→` | 上一张/下一张 |
| `↑` / `↓` | 放大/缩小 |
| `Home` | 第一张图片 |
| `End` | 最后一张图片 |

### 查看
| 快捷键 | 功能 |
|--------|------|
| `F11` | 全屏切换 |
| `G` | 画廊/查看器切换 |
| `Ctrl + 0` | 原始大小 |
| `Ctrl + 1` | 适应窗口 |

### 文件
| 快捷键 | 功能 |
|--------|------|
| `Ctrl + O` | 打开文件 |
| `Ctrl + Shift + O` | 打开文件夹 |
| `Ctrl + C` | 复制图片 |
| `Delete` | 删除文件 |

### 其他
| 快捷键 | 功能 |
|--------|------|
| `F1` | 显示帮助 |
| `?` | 显示快捷键帮助 |
| `Ctrl + ,` | 打开设置 |
| `Ctrl + Q` | 退出程序 |
