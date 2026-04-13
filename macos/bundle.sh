#!/usr/bin/env bash
#
# Build Lexito.app macOS bundle.
#
# Usage:
#   ./macos/bundle.sh          # build release + bundle
#   ./macos/bundle.sh --skip-build  # bundle only (binary must exist)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Lexito"
BUNDLE_DIR="$PROJECT_ROOT/target/release/${APP_NAME}.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# ── Build ────────────────────────────────────────────────────────────
if [[ "${1:-}" != "--skip-build" ]]; then
    echo "Building release binary..."
    cargo build --release -p lexito --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi

BINARY="$PROJECT_ROOT/target/release/lexito"
if [[ ! -f "$BINARY" ]]; then
    echo "Error: release binary not found at $BINARY" >&2
    exit 1
fi

# ── Create bundle structure ──────────────────────────────────────────
rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

cp "$BINARY" "$MACOS_DIR/lexito"
cp "$SCRIPT_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"

# ── Generate .icns from SVG ──────────────────────────────────────────
ICON_SVG="$PROJECT_ROOT/crates/desktop/assets/icon.svg"
ICONSET_DIR=$(mktemp -d)/AppIcon.iconset

if command -v rsvg-convert &>/dev/null; then
    RASTERIZE="rsvg-convert"
elif command -v resvg &>/dev/null; then
    RASTERIZE="resvg"
else
    RASTERIZE=""
fi

if [[ -n "$RASTERIZE" && -f "$ICON_SVG" ]]; then
    echo "Generating AppIcon.icns..."
    mkdir -p "$ICONSET_DIR"
    for SIZE in 16 32 128 256 512; do
        DOUBLE=$((SIZE * 2))
        if [[ "$RASTERIZE" == "rsvg-convert" ]]; then
            rsvg-convert -w "$SIZE" -h "$SIZE" "$ICON_SVG" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"
            rsvg-convert -w "$DOUBLE" -h "$DOUBLE" "$ICON_SVG" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
        else
            resvg --width "$SIZE" --height "$SIZE" "$ICON_SVG" "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"
            resvg --width "$DOUBLE" --height "$DOUBLE" "$ICON_SVG" "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
        fi
    done
    iconutil -c icns "$ICONSET_DIR" -o "$RESOURCES_DIR/AppIcon.icns"
    rm -rf "$(dirname "$ICONSET_DIR")"
    echo "AppIcon.icns created."
else
    echo "Warning: rsvg-convert or resvg not found; skipping .icns generation."
    echo "Install with: brew install librsvg"
fi

# ── Done ─────────────────────────────────────────────────────────────
echo ""
echo "Bundle created: $BUNDLE_DIR"
echo "Run with: open \"$BUNDLE_DIR\""
