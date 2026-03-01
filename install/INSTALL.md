# Image-Viewer Installation Guide

This guide explains how to install Image-Viewer on different platforms and register it as the default image viewer.

## Table of Contents

- [Windows](#windows)
- [macOS](#macos)
- [Linux](#linux)

---

## Windows

### 系统要求

- Windows 7 / 8 / 10 / 11 (64位)
- 无需管理员权限即可运行绿色版

### 方式一：绿色版/便携版（推荐）

最简单的方式，无需安装，解压即用：

1. 下载绿色版压缩包 `image-viewer-windows-x64.zip`
2. 解压到任意文件夹（如 `D:\Tools\Image-Viewer`）
3. 直接运行 `image-viewer.exe`

#### 绿色版特点
- ✅ 无需管理员权限
- ✅ 不写注册表
- ✅ 可放在 U 盘随身携带
- ✅ 配置保存在程序目录下

### 方式二：安装版（可选）

如需系统集成（右键菜单、默认程序）：

1. 下载安装版 `image-viewer-windows-x64-setup.exe`
2. 运行安装程序，按提示完成安装
3. 可选：勾选"添加到右键菜单"

### 方式三：从源码构建

1. 安装 Rust 工具链（https://rustup.rs/）
2. 克隆仓库并构建：
   ```cmd
   cargo build --release
   ```
3. 构建完成后，可运行 `install\windows\install.bat` 进行系统集成

### 右键菜单注册（可选）

运行 `install\windows\register-context-menu.bat` 添加"使用 Image-Viewer 打开"到右键菜单：

```cmd
# 在项目目录下运行
install\windows\register-context-menu.bat
```

**说明**：
- 此脚本仅修改当前用户注册表，**无需管理员权限**
- 支持 Windows 7/8/10/11
- 可通过 `unregister-context-menu.bat` 卸载

### 设置为默认图片查看器

#### 方法 1：通过设置应用（Windows 10/11）
1. 打开 设置 → 应用 → 默认应用
2. 搜索 ".png"、".jpg" 等图片格式
3. 选择 Image-Viewer 作为默认应用

#### 方法 2：通过右键菜单
1. 右键点击任意图片文件
2. 选择"打开方式" → "选择其他应用"
3. 找到 Image-Viewer 并勾选"始终使用此应用打开"

### 便携版配置

绿色版/便携版的配置文件保存在程序目录下：
```
Image-Viewer/
├── image-viewer.exe
├── config/
│   └── config.toml      # 配置文件
└── cache/               # 缓存目录
```

### 卸载

#### 绿色版
直接删除程序文件夹即可，无残留。

#### 安装版
1. 运行 `uninstall.bat` 或控制面板卸载
2. 如需清理注册表，运行 `unregister-context-menu.bat`

### 支持的图片格式

- PNG (.png)
- JPEG (.jpg, .jpeg)
- GIF (.gif)
- WebP (.webp)
- TIFF (.tiff, .tif)
- BMP (.bmp)
- ICO (.ico)

## macOS

### Quick Install

1. Build the project:
   ```bash
   cargo build --release
   ```

2. Run the installer:
   ```bash
   ./install/macos/install.sh
   ```

3. The app bundle will be created in `~/Applications/Image-Viewer.app`

### Setting as Default

To set Image-Viewer as the default app for image files:

1. Right-click on any image file
2. Select **Get Info**
3. Under **"Open with:"**, select **Image-Viewer**
4. Click **"Change All..."** to apply to all files of that type

### Supported Formats

All standard macOS image formats are supported:
- PNG
- JPEG
- GIF
- WebP
- TIFF
- BMP

### Uninstall

Delete the app bundle:
```bash
rm -rf ~/Applications/Image-Viewer.app
```

---

## Linux

### Quick Install (User)

Installs to `~/.local/` (recommended):

```bash
./install/linux/install.sh
# Select option 1 (User install)
```

### System Install

Installs to `/usr/local/` (requires sudo):

```bash
./install/linux/install.sh
# Select option 2 (System install)
```

### Integration with xdg-open

After installation, you can open images using:

```bash
xdg-open /path/to/image.png
```

Or right-click on an image file in your file manager and select "Open with Image Viewer".

### Setting as Default

#### Using xdg-mime (Command Line)

```bash
# Set as default for PNG
xdg-mime default image-viewer.desktop image/png

# Set as default for JPEG
xdg-mime default image-viewer.desktop image/jpeg

# Set as default for GIF
xdg-mime default image-viewer.desktop image/gif

# Set as default for WebP
xdg-mime default image-viewer.desktop image/webp

# Set as default for TIFF
xdg-mime default image-viewer.desktop image/tiff

# Set as default for BMP
xdg-mime default image-viewer.desktop image/bmp
```

#### Using GUI (GNOME/KDE/XFCE)

1. Right-click on an image file
2. Select **Properties** or **Open With**
3. Choose **Image Viewer** from the list
4. Click **"Set as default"** or **"Set as default application"**

### Supported MIME Types

The .desktop file registers the following MIME types:
- `image/png`
- `image/jpeg`
- `image/gif`
- `image/webp`
- `image/tiff`
- `image/bmp`

### Uninstall

```bash
./install/linux/uninstall.sh
```

---

## Building from Source

### Prerequisites

- Rust 1.93 or higher
- Platform-specific dependencies:

#### Linux Dependencies

```bash
# Ubuntu/Debian
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install gtk3-devel libxcb-devel

# Arch
sudo pacman -S gtk3 libxcb
```

### Build

```bash
cargo build --release
```

The binary will be available at `target/release/image-viewer`.

---

## File Locations

### Windows
- Executable: `%PROGRAMFILES%\Image-Viewer\image-viewer.exe`
- Registry entries: `HKEY_CLASSES_ROOT\.[extension]\shell\OpenWithImageViewer`

### macOS
- App Bundle: `~/Applications/Image-Viewer.app`
- Configuration: `~/Library/Application Support/com.imageviewer.image-viewer/`

### Linux (User Install)
- Executable: `~/.local/bin/image-viewer`
- Desktop file: `~/.local/share/applications/image-viewer.desktop`
- Configuration: `~/.config/image-viewer/`

### Linux (System Install)
- Executable: `/usr/local/bin/image-viewer`
- Desktop file: `/usr/share/applications/image-viewer.desktop`
- Configuration: `~/.config/image-viewer/`

---

## Troubleshooting

### Windows: "Windows cannot find the file"
Make sure the executable path in the .reg file matches your actual installation location.

### macOS: App won't open
Right-click the app and select "Open" to bypass Gatekeeper for the first time.

### Linux: Command not found
Add `~/.local/bin` to your PATH:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Linux: Desktop file not showing
Run:
```bash
update-desktop-database ~/.local/share/applications/
```

---

## License

See the main [LICENSE](../LICENSE) file for details.
