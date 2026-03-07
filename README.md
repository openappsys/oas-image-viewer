# Image Viewer

A modern, high-performance image viewer built with Rust and egui.

[![CI](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[中文文档](README.zh-CN.md)

## Features

- 🖼️ **Multi-format Support**: PNG, JPEG, GIF, WebP, TIFF, BMP
- 📁 **Gallery View**: Thumbnail grid browsing
- 🔍 **Zoom & Pan**: Mouse wheel zoom, drag to pan
- ⚡ **High Performance**: Built with Rust for ultimate performance
- 🎨 **Modern UI**: Clean interface powered by egui
- 🔧 **Configurable**: Customize via configuration file
- 🖥️ **Cross-platform**: Windows, macOS, Linux support
- 🧪 **High Test Coverage**: 451+ unit tests ensuring quality

## Architecture

This project adopts **Layered Architecture** design:

```
┌─────────────────────────────────────────────────────────────┐
│                        UI Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │   main.rs   │ │ info_panel  │ │   shortcuts_help    │   │
│  │  (entry)    │ │(info panel) │ │  (shortcuts help)   │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                     Domain Layer                            │
│  ┌─────────────────────────┐  ┌─────────────────────────┐   │
│  │      app/mod.rs         │  │       config.rs         │   │
│  │   (ImageViewerApp)      │  │    (config mgmt)        │   │
│  │   app state, event loop │  │  config load & persist  │   │
│  └─────────────────────────┘  └─────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                   Application Layer                         │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐    │
│  │    viewer    │ │    gallery   │ │     decoder      │    │
│  │ (img viewer) │ │   (gallery)  │ │  (img decoder)   │    │
│  │ zoom/pan/ren │ │ thumb/grid   │ │  multi fmt dec   │    │
│  └──────────────┘ └──────────────┘ └──────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                   Infrastructure Layer                      │
│  ┌──────────┐ ┌──────────┐ ┌────────────────────────────┐  │
│  │   dnd    │ │ clipboard│ │          utils             │  │
│  │ (drag)   │ │(clipboard)│ │  errors / threading / ...  │  │
│  └──────────┘ └──────────┘ └────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Core Library Layer                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Zero external deps: Image, Gallery, Cache, Events  │   │
│  │  Implements: ImageSource, ImageOperations, Gallery  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Layer Dependencies

```
UI Layer (adapters/)
        ↓
Domain Layer (app/) ←→ Application Layer (use_cases/)
        ↓
Infrastructure Layer (infrastructure/)
        ↓
Core Library Layer (core/)
```

## Installation

### Prerequisites

- Rust 1.93 or higher
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
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# Build release version
cargo build --release

# Run
./target/release/image-viewer
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/yourusername/image-viewer/releases) page.

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
4. Find "Image Viewer" under Security, click **"Open Anyway"**
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
chmod +x ./image-viewer
./image-viewer
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
image-viewer

# Open specific image
image-viewer /path/to/image.png

# Open directory
image-viewer /path/to/images/

# Show help
image-viewer --help
```

### Keyboard Shortcuts

| Key | Function | Mode |
|-----|----------|------|
| `Space` / `→` | Next image | All |
| `←` | Previous image | All |
| `↑` / `↓` | Scroll gallery | Gallery |
| `+` / `-` | Zoom in/out | Viewer |
| `0` | Fit to window | Viewer |
| `1` | Actual size (100%) | Viewer |
| `F` | Toggle fullscreen | All |
| `G` | Toggle gallery mode | All |
| `I` | Show/hide info panel | Viewer |
| `Delete` | Delete current image | All |
| `Cmd/Ctrl + C` | Copy image | All |
| `Esc` | Exit fullscreen / Close image | - |

### Mouse Operations

- **Click**: Select image / Open image
- **Double-click**: Toggle between gallery and viewer modes
- **Scroll**: Zoom in/out (in viewer mode) / Scroll gallery (in gallery mode)
- **Drag**: Pan image (in viewer mode) / Drag selection (in gallery mode)
- **Right-click**: Context menu

### Configuration

Configuration file location:
- **Windows**: `%APPDATA%\image-viewer\config.toml`
- **macOS**: `~/Library/Application Support/image-viewer/config.toml`
- **Linux**: `~/.config/image-viewer/config.toml`

Example configuration:
```toml
[app]
# Window settings
window_width = 1200
window_height = 800
maximized = false

[gallery]
# Gallery settings
thumbnail_size = 120
items_per_row = 4
spacing = 10

[viewer]
# Viewer settings
background_color = [0.1, 0.1, 0.1]  # RGB
default_zoom = "fit"  # or "actual"
show_info_panel = true
smooth_scroll = true

[shortcuts]
# Custom keyboard shortcuts
next_image = "Space"
prev_image = "Left"
```

## Development

### Project Structure

```
image-viewer/
├── Cargo.toml           # Project configuration
├── config.example.toml  # Configuration template
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Library root
│   ├── core/           # Core library (domain + logic)
│   ├── adapters/       # Adapters (egui UI, infrastructure)
│   └── infrastructure/ # Infrastructure implementations
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
make check       # or: cargo check && cargo clippy
```

### Tech Stack

- **GUI Framework**: [egui](https://github.com/emilk/egui) - Immediate mode GUI
- **Image Decoding**: [image](https://github.com/image-rs/image) - Rust image library
- **Async Runtime**: [tokio](https://tokio.rs/) - Async runtime
- **Config**: [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml) - Config serialization
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) - Structured logging

## Roadmap

### v0.3.x (Current)
- ✅ Multi-format image support
- ✅ Gallery view
- ✅ Basic image editing (rotate, flip)
- ✅ Drag & drop support
- ✅ Keyboard shortcuts

### v0.4.0 (Planned)
- [ ] Image editing (crop, adjust brightness/contrast)
- [ ] Batch processing
- [ ] Slideshow mode
- [ ] Custom themes
- [ ] Plugin system

### v1.0.0 (Release)
- [ ] Code signing (Windows/macOS)
- [ ] Auto-update
- [ ] Multi-language support
- [ ] Cloud sync (optional)

See [ROADMAP.md](docs/ROADMAP.md) for full roadmap.

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
- No clippy warnings: `cargo clippy`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [egui](https://github.com/emilk/egui) - Excellent Rust GUI framework
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - egui's official framework integration
- [image](https://github.com/image-rs/image) - Rust image processing library

## Support

- 💬 [Discussions](https://github.com/yourusername/image-viewer/discussions)
- 🐛 [Issues](https://github.com/yourusername/image-viewer/issues)
- 📧 Email: your.email@example.com

---

**Made with ❤️ using Rust**
