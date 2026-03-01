# Image Viewer

A modern, fast image viewer built with Rust and egui.

[![CI](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/image-viewer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🖼️ **Multiple Format Support**: PNG, JPEG, GIF, WebP, TIFF, BMP
- 📁 **Gallery View**: Browse images in a thumbnail grid
- 🔍 **Zoom & Pan**: Smooth zooming with mouse wheel and drag to pan
- ⚡ **Fast**: Built with Rust for maximum performance
- 🎨 **Modern UI**: Clean interface powered by egui
- 🔧 **Configurable**: Customize through config file
- 🖥️ **Cross-Platform**: Windows, macOS, and Linux support

## Installation

### Prerequisites

- [Rust 1.93 or higher
- System dependencies (Linux only):
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
# Clone the repository
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# Build release version
cargo build --release

# Run
./target/release/image-viewer
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/yourusername/image-viewer/releases) page.

## Usage

### Basic Usage

```bash
# Open the image viewer
image-viewer

# Open with a specific image
image-viewer /path/to/image.png

# Open a directory
image-viewer /path/to/images/
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + O` | Open file |
| `Ctrl + +` | Zoom in |
| `Ctrl + -` | Zoom out |
| `Ctrl + 0` | Reset zoom |
| `← / →` | Previous/Next image |
| `F11` | Toggle fullscreen |
| `Esc` | Exit fullscreen / Close viewer |
| `F` | Toggle file info panel |
| `?` | Show keyboard shortcuts help |

### Mouse Controls

- **Scroll**: Zoom in/out
- **Drag**: Pan when zoomed
- **Double-click**: Toggle fullscreen
- **Right-click**: Context menu

## Configuration

Configuration is stored in platform-specific locations:

- **Linux**: `~/.config/image-viewer/config.toml`
- **macOS**: `~/Library/Application Support/com.imageviewer.image-viewer/config.toml`
- **Windows**: `%APPDATA%\image-viewer\config\config.toml`

### Example Configuration

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

## Development

### Project Structure

```
image-viewer/
├── src/
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration management
│   ├── app/             # Main application logic
│   ├── viewer/          # Image viewer component
│   ├── gallery/         # Gallery grid component
│   ├── decoder/         # Image decoding
│   └── utils/           # Utilities (errors, threading)
├── assets/              # Static assets
├── .github/workflows/   # CI/CD configuration
└── Cargo.toml          # Dependencies
```

### Development Commands

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Build release
cargo build --release
```

### Code Quality

This project enforces strict code quality:

- **rustfmt**: Consistent code formatting (`rustfmt.toml`)
- **clippy**: Linting with strict rules (`.clippy.toml`)
- **EditorConfig**: Consistent editor settings (`.editorconfig`)

## Architecture

### Tech Stack

- **GUI Framework**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **Image Decoding**: [image](https://github.com/image-rs/image) crate
- **Error Handling**: [anyhow](https://github.com/dtolnay/anyhow) + [thiserror](https://github.com/dtolnay/thiserror)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing)
- **Configuration**: [serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml)
- **Threading**: [rayon](https://github.com/rayon-rs/rayon) for parallel processing

### Module Overview

| Module | Purpose |
|--------|---------|
| `app` | Main application state and event loop |
| `viewer` | Full-size image display with zoom/pan |
| `gallery` | Thumbnail grid for browsing |
| `decoder` | Image format decoding |
| `utils` | Error types and threading utilities |

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- Code is formatted with `cargo fmt`
- Clippy passes with `cargo clippy -- -D warnings`
- All tests pass with `cargo test`

## Roadmap

- [ ] Slideshow mode
- [ ] EXIF metadata display
- [ ] Basic image editing (rotate, crop)
- [ ] Custom themes
- [ ] Plugin system
- [ ] RAW image support

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [egui](https://github.com/emilk/egui) for the excellent immediate-mode GUI library
- [image-rs](https://github.com/image-rs) for the image decoding library
- All contributors to the Rust ecosystem
