# OAS Image Viewer

A modern, high-performance image viewer built with Rust and egui.

[![CI](https://github.com/openappsys/oas-image-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/openappsys/oas-image-viewer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[中文文档](README.zh-CN.md) | [English Documentation](README.md)

## Features

- 🖼️ **Multi-format Support**: PNG, JPEG, GIF, WebP, TIFF, BMP
- 📁 **Gallery View**: Thumbnail grid browsing
- 🔍 **Zoom & Pan**: Mouse wheel zoom, drag to pan
- ⚡ **High Performance**: Built with Rust for ultimate performance
- 🎨 **Modern UI**: Clean interface powered by egui
- 🔧 **Configurable**: Customize via configuration file
- 🖥️ **Cross-platform**: Windows, macOS, Linux support
- 🧪 **Quality-focused Testing**: unit + integration tests for core workflows

## Architecture

This project adopts **Layered Architecture** design:

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Entry Layer                               │
│  ┌──────────────────────┐  ┌────────────────────────────────────┐  │
│  │       main.rs        │  │               lib.rs               │  │
│  │ startup/args/window  │  │ module exports and public API      │  │
│  └──────────────────────┘  └────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                        Adapters Layer                               │
│  ┌────────────────────────────┐ ┌────────────────────────────────┐  │
│  │       adapters/egui        │ │       adapters/platform        │  │
│  │ UI rendering/menu/input    │ │ Linux/macOS/Windows integration│  │
│  └────────────────────────────┘ └────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                           Core Layer                                │
│  ┌────────────────────┐ ┌────────────────────┐ ┌─────────────────┐  │
│  │    core/domain     │ │     core/ports     │ │ core/use_cases  │  │
│  │ entities/value obj │ │ interface contracts│ │ app services    │  │
│  └────────────────────┘ └────────────────────┘ └─────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                     Infrastructure Layer                             │
│  ┌────────────────────┐ ┌────────────────────┐ ┌─────────────────┐  │
│  │    JsonStorage     │ │   FsImageSource    │ │ filesystem/I-O  │  │
│  │ config persistence │ │ image source impl  │ │ technical detail│  │
│  └────────────────────┘ └────────────────────┘ └─────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Layer Dependencies

```
Entry Layer (main.rs / lib.rs)
        ↓
Adapters Layer (adapters/egui + adapters/platform)
        ↓
Core Layer (core/domain + core/ports + core/use_cases)
        ↓
Infrastructure Layer (infrastructure/)
```

## Installation

### Prerequisites

- Rust 1.94 or higher
- Linux system dependencies:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

  # Fedora
  sudo dnf install gtk3-devel libxcb-devel

  # Arch
  sudo pacman -S gtk3 libxcb
  ```

### Build from Source

```bash
# Clone repository
git clone git@github.com:openappsys/oas-image-viewer.git
cd oas-image-viewer

# Build release version
cargo build --release

# Run
./target/release/oas-image-viewer
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/openappsys/oas-image-viewer/releases) page.

#### ⚠️ Security Notice (First Run)

Since the current version is not code-signed yet, the system may show security warnings on first run. Please follow these steps:

**Windows:**
1. If you see "Windows protected your PC"
2. Click **"More info"**
3. Click **"Run anyway"**
4. Next time it won't prompt

> 💡 Or: Right-click program → Properties → Check "Unblock" → Apply

**macOS:**
1. If you see "Cannot verify developer"
2. Click **"Cancel"**
3. Open **System Settings → Privacy & Security**
4. Find "OAS Image Viewer" under Security, click **"Open Anyway"**
5. Click **"Open"** again to confirm

> 💡 Or: Right-click app → Select "Open" → Click "Open"
> 
> 💡 Terminal command (quick fix):
> ```bash
> xattr -d com.apple.quarantine /Applications/Image\ Viewer.app
> ```

**Linux:**
```bash
# Grant execute permission and run
chmod +x ./oas-image-viewer
./oas-image-viewer
```

> 💡 **Recommended**: Use AppImage format, no installation needed, double-click to run

**Why this prompt?**
- Code signing certificates require annual fees and organization verification
- We are preparing official signing, will be resolved in v1.0
- All source code is open and auditable, please use with confidence

## Usage

### Basic Usage

```bash
# Launch image viewer
oas-image-viewer

# Open specific image
oas-image-viewer /path/to/image.png

# Open directory
oas-image-viewer /path/to/images/

# Open with debug logging
RUST_LOG=debug oas-image-viewer /path/to/image.png
```

### Keyboard Shortcuts

| Key | Function | Mode |
|-----|----------|------|
| `→` | Next image | All |
| `←` | Previous image | All |
| `Cmd/Ctrl + O` | Open file dialog | All |
| `Cmd/Ctrl + Shift + O` | Open folder dialog | All |
| `Cmd + C` (macOS) / `Ctrl + C` (Windows/Linux) | Copy current image | Viewer |
| `Cmd + Shift + C` (macOS) / `Ctrl + Shift + C` (Windows/Linux) | Copy current image path | Viewer |
| `Cmd/Ctrl + + / Cmd/Ctrl + -` | Zoom in/out | Viewer |
| `Cmd/Ctrl + 0` | Fit to window | Viewer |
| `Cmd/Ctrl + 1` | Actual size (100%) | Viewer |
| `Cmd/Ctrl + 2` | Fit to width | Viewer |
| `Cmd/Ctrl + 3` | Fit to height | Viewer |
| `B` | Cycle background (black/gray/white) | Viewer |
| `S` | Start/Pause slideshow | Viewer |
| `R` / `Shift + R` | Rotate 90° clockwise/counterclockwise (read-only) | Viewer |
| `H` / `V` | Flip horizontal/vertical (read-only) | Viewer |
| `F11` | Toggle fullscreen | All |
| `G` | Toggle gallery mode | All |
| `F` | Show/hide info panel | Viewer |
| `?` | Toggle shortcuts help panel | All |
| `Esc` | Exit fullscreen / Close overlays | All |

### Mouse Operations

- **Click**: Select image / Open image
- **Double-click**: Toggle fullscreen
- **Scroll**: Zoom in/out (in viewer mode) / Scroll gallery (in gallery mode)
- **Drag**: Pan image (in viewer mode)
- **Right-click**: Context menu

### Configuration

Configuration file location is resolved by `directories::ProjectDirs`:
- **Linux**: `~/.config/oas-image-viewer/config.toml`
- **macOS**: `~/Library/Application Support/com.openappsys.oas-image-viewer/config.toml`
- **Windows**: `%APPDATA%\openappsys\oas-image-viewer\config\config.toml`

Example configuration:
```toml
[window]
# Window settings
width = 1200.0
height = 800.0
maximized = false

[gallery]
# Gallery settings
thumbnail_size = 120
items_per_row = 0
grid_spacing = 12.0
show_filenames = true

[viewer]
# Viewer settings
background_color = [30, 30, 30]
fit_to_window = true
show_info_panel = false
min_scale = 0.1
max_scale = 20.0
zoom_step = 1.25
smooth_scroll = true
```

## Development

### Project Structure

```
oas-image-viewer/
├── Cargo.toml           # Project configuration
├── config.example.toml  # Configuration template
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Library root
│   ├── adapters/       # Adapter layer (UI + platform integration)
│   │   ├── egui/       # egui UI adapter
│   │   ├── platform/   # OS integration (linux/macos/windows)
│   │   └── macos_file_open.rs
│   ├── core/           # Core business layer (Domain + Use Cases)
│   │   ├── domain/     # Entities, value objects
│   │   ├── ports/      # Interface definitions
│   │   └── use_cases/  # Use case implementations
│   └── infrastructure/ # Infrastructure layer
│       ├── mod.rs
│       ├── fs_image_source.rs
│       ├── storage.rs
│       ├── async_image_source.rs
│       ├── file_dialog.rs
│       └── tests.rs
├── tests/              # Integration tests
├── docs/               # Documentation
├── assets/             # Icons, resources
└── scripts/            # Build scripts
```

### Build & Test

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Check code
cargo clippy --all-targets -- -D warnings
```

### Tech Stack

- **GUI Framework**: [egui](https://github.com/emilk/egui) - Immediate mode GUI
- **Image Decoding**: [image](https://github.com/image-rs/image) - Rust image library
- **Parallel Processing**: [rayon](https://github.com/rayon-rs/rayon) - Parallel iterator
- **Config**: [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml) - Config serialization
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) - Structured logging

## Roadmap

### v0.3.x (Current)
- ✅ Multi-format image support
- ✅ Gallery view
- ✅ Metadata panel and clipboard workflows
- ✅ Drag & drop support
- ✅ Keyboard shortcuts

### v0.4.0 (In Progress)
- [x] Slideshow mode (play/pause, interval, end behavior)
- [x] Read-only transform preview (R/Shift+R/H/V)
- [x] Full EXIF display (camera/lens/exposure/GPS)
- [ ] Image editing (crop, adjust brightness/contrast)
- [ ] Batch processing

### v1.0.0 (Release)
- [ ] Code signing (Windows/macOS)
- [ ] Auto-update
- [ ] Cloud sync (optional)

## Contributing

We welcome contributions! Please read our [Contributing Guide](docs/DEVELOPMENT.md) first.

### Quick Start

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Create Pull Request

Please ensure:
- Code follows `rustfmt` style
- All tests pass: `cargo test`
- No clippy warnings: `cargo clippy --all-targets -- -D warnings`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [egui](https://github.com/emilk/egui) - Excellent Rust GUI framework
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - egui's official framework integration
- [image](https://github.com/image-rs/image) - Rust image processing library

## Support

- 💬 [Discussions](https://github.com/openappsys/oas-image-viewer/discussions)
- 🐛 [Issues](https://github.com/openappsys/oas-image-viewer/issues)
- 📧 Email: team@openappsys.com

---

**Made with ❤️ using Rust**
