# Image-Viewer Installation Guide

This guide explains how to install Image-Viewer on different platforms and register it as the default image viewer.

## Table of Contents

- [Windows](#windows)
- [macOS](#macos)
- [Linux](#linux)

---

## Windows

### Quick Install

1. Build the project:
   ```cmd
   cargo build --release
   ```

2. Run the installer as Administrator:
   ```cmd
   install\windows\install.bat
   ```

### Manual Registration

If you prefer to manually register the context menu:

1. Double-click `install\windows\register-context-menu.reg`
2. Click "Yes" when prompted by Registry Editor
3. The "Open with Image-Viewer" option will appear in the right-click menu for supported image files

### Uninstall

1. Run the uninstaller:
   ```cmd
   install\windows\unregister-context-menu.reg
   ```

2. Delete the installation directory:
   ```cmd
   rmdir /s "%PROGRAMFILES%\Image-Viewer"
   ```

### Supported Formats

- PNG (.png)
- JPEG (.jpg, .jpeg)
- GIF (.gif)
- WebP (.webp)
- TIFF (.tiff, .tif)
- BMP (.bmp)
- ICO (.ico) - optional

---

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

- Rust 1.90 or higher
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
