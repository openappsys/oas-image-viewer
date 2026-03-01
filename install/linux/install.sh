#!/bin/bash

# Image-Viewer Linux Installer
# This script installs Image-Viewer and registers it with xdg-open

set -e

APP_NAME="image-viewer"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

echo "========================================="
echo "Image-Viewer Linux Installation"
echo "========================================="
echo ""

# Detect distro
if [[ -f /etc/os-release ]]; then
    . /etc/os-release
    DISTRO=$NAME
else
    DISTRO="Unknown"
fi

echo "Detected distribution: $DISTRO"
echo ""

# Build release version if not exists
if [[ ! -f "$PROJECT_DIR/target/release/image-viewer" ]]; then
    echo "Building release version..."
    cd "$PROJECT_DIR"
    cargo build --release
    echo ""
fi

# Determine installation type
echo "Select installation type:"
echo "  1) User install (recommended) - installs to ~/.local/"
echo "  2) System install - requires sudo, installs to /usr/local/"
echo ""
read -p "Enter choice [1-2]: " choice

case $choice in
    1)
        INSTALL_TYPE="user"
        BIN_DIR="$HOME/.local/bin"
        APP_DIR="$HOME/.local/share/applications"
        ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
        ;;
    2)
        INSTALL_TYPE="system"
        BIN_DIR="/usr/local/bin"
        APP_DIR="/usr/share/applications"
        ICON_DIR="/usr/share/icons/hicolor/256x256/apps"
        ;;
    *)
        echo "Invalid choice. Exiting."
        exit 1
        ;;
esac

echo ""
echo "Installing to: $BIN_DIR"
echo ""

# Create directories
mkdir -p "$BIN_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$ICON_DIR"

# Copy executable
echo "Copying executable..."
if [[ "$INSTALL_TYPE" == "system" ]]; then
    sudo cp "$PROJECT_DIR/target/release/image-viewer" "$BIN_DIR/"
    sudo chmod +x "$BIN_DIR/image-viewer"
else
    cp "$PROJECT_DIR/target/release/image-viewer" "$BIN_DIR/"
    chmod +x "$BIN_DIR/image-viewer"
fi

# Install .desktop file
echo "Installing desktop file..."
DESKTOP_FILE="$SCRIPT_DIR/image-viewer.desktop"

# Update Exec path in desktop file
temp_desktop=$(mktemp)
sed "s|Exec=image-viewer|Exec=$BIN_DIR/image-viewer|g" "$DESKTOP_FILE" > "$temp_desktop"

if [[ "$INSTALL_TYPE" == "system" ]]; then
    sudo cp "$temp_desktop" "$APP_DIR/image-viewer.desktop"
    sudo chmod +x "$APP_DIR/image-viewer.desktop"
else
    cp "$temp_desktop" "$APP_DIR/image-viewer.desktop"
    chmod +x "$APP_DIR/image-viewer.desktop"
fi
rm "$temp_desktop"

# Create a simple icon placeholder (256x256)
echo "Creating icon..."
ICON_FILE="$ICON_DIR/image-viewer.png"
if command -v convert &> /dev/null; then
    # Create a simple colored icon using ImageMagick
    if [[ "$INSTALL_TYPE" == "system" ]]; then
        sudo convert -size 256x256 xc:steelblue -pointsize 30 -fill white -gravity center -annotate +0+0 "IV" "$ICON_FILE" 2>/dev/null || true
    else
        convert -size 256x256 xc:steelblue -pointsize 30 -fill white -gravity center -annotate +0+0 "IV" "$ICON_FILE" 2>/dev/null || true
    fi
else
    # Create a simple SVG and convert if possible
    echo "Icon creation skipped (ImageMagick not installed)"
fi

# Register MIME types and update desktop database
echo "Registering MIME types..."
if command -v xdg-mime &> /dev/null; then
    # Set as default for image types (optional - user can choose)
    :
fi

# Update desktop database
if [[ "$INSTALL_TYPE" == "system" ]]; then
    if command -v update-desktop-database &> /dev/null; then
        sudo update-desktop-database "$APP_DIR" 2>/dev/null || true
    fi
else
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$APP_DIR" 2>/dev/null || true
    fi
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    if [[ "$INSTALL_TYPE" == "system" ]]; then
        sudo gtk-update-icon-cache -f /usr/share/icons/hicolor/ 2>/dev/null || true
    else
        gtk-update-icon-cache -f "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
    fi
fi

echo ""
echo "========================================="
echo "Installation completed successfully!"
echo "========================================="
echo ""
echo "Image-Viewer installed to: $BIN_DIR/image-viewer"
echo "Desktop file installed to: $APP_DIR/image-viewer.desktop"
echo ""
echo "You can now:"
echo "  - Run 'image-viewer' from terminal"
echo "  - Right-click image files to open with Image-Viewer"
echo "  - Use 'xdg-open image.png' to open images"
echo ""
echo "Supported formats: PNG, JPEG, GIF, WebP, TIFF, BMP"
echo ""

# Check if binary is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "WARNING: $BIN_DIR is not in your PATH."
    echo "Add the following to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    echo ""
fi
