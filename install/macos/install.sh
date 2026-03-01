#!/bin/bash

# Image-Viewer macOS Installer
# This script installs Image-Viewer and registers it as an image viewer

set -e

APP_NAME="Image-Viewer"
BUNDLE_ID="com.imageviewer.image-viewer"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

echo "========================================="
echo "Image-Viewer macOS Installation"
echo "========================================="
echo ""

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: This script is for macOS only."
    exit 1
fi

# Build release version if not exists
if [[ ! -f "$PROJECT_DIR/target/release/image-viewer" ]]; then
    echo "Building release version..."
    cd "$PROJECT_DIR"
    cargo build --release
fi

# Create app bundle
APP_BUNDLE="$HOME/Applications/${APP_NAME}.app"
echo "Creating app bundle at: $APP_BUNDLE"

# Remove existing bundle if present
if [[ -d "$APP_BUNDLE" ]]; then
    echo "Removing existing app bundle..."
    rm -rf "$APP_BUNDLE"
fi

# Create bundle structure
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy executable
cp "$PROJECT_DIR/target/release/image-viewer" "$APP_BUNDLE/Contents/MacOS/"

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$APP_BUNDLE/Contents/"

# Create a simple icon (optional - user can replace with real icon)
# For now, we'll skip icon creation

echo ""
echo "========================================="
echo "Installation completed successfully!"
echo "========================================="
echo ""
echo "App bundle created at: $APP_BUNDLE"
echo ""
echo "To set Image-Viewer as the default app for an image:"
echo "  1. Right-click an image file"
echo "  2. Select 'Get Info'"
echo "  3. Under 'Open with:', select Image-Viewer"
echo "  4. Click 'Change All...' to set as default"
echo ""
echo "Supported formats: PNG, JPEG, GIF, WebP, TIFF, BMP"
echo ""
