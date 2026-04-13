#!/usr/bin/env bash
#
# Build Lexito.app macOS bundle.
#
# Usage:
#   ./macos/bundle.sh                              # build release + bundle
#   ./macos/bundle.sh --skip-build                  # bundle only (binary must exist)
#   ./macos/bundle.sh --target aarch64-apple-darwin # cross-compile for target
#   ./macos/bundle.sh --version v0.2.0              # stamp version into Info.plist
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Lexito"

TARGET=""
SKIP_BUILD=false
VERSION=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build) SKIP_BUILD=true; shift ;;
        --target)     TARGET="$2"; shift 2 ;;
        --version)    VERSION="$2"; shift 2 ;;
        *)            echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
done

if [[ -n "$TARGET" ]]; then
    TARGET_DIR="$PROJECT_ROOT/target/$TARGET/release"
else
    TARGET_DIR="$PROJECT_ROOT/target/release"
fi

BUNDLE_DIR="$TARGET_DIR/${APP_NAME}.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# ── Build ────────────────────────────────────────────────────────────
if ! $SKIP_BUILD; then
    echo "Building release binary..."
    if [[ -n "$TARGET" ]]; then
        cargo build --release -p lexito --target "$TARGET" --manifest-path "$PROJECT_ROOT/Cargo.toml"
    else
        cargo build --release -p lexito --manifest-path "$PROJECT_ROOT/Cargo.toml"
    fi
fi

BINARY="$TARGET_DIR/lexito"
if [[ ! -f "$BINARY" ]]; then
    echo "Error: release binary not found at $BINARY" >&2
    exit 1
fi

# ── Create bundle structure ──────────────────────────────────────────
rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

cp "$BINARY" "$MACOS_DIR/lexito"
cp "$SCRIPT_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"

# ── Stamp version into Info.plist ────────────────────────────────────
if [[ -n "$VERSION" ]]; then
    VER="${VERSION#v}"
    sed -i.bak "s|__VERSION__|${VER}|g" "$CONTENTS_DIR/Info.plist"
    rm -f "$CONTENTS_DIR/Info.plist.bak"
    echo "Stamped version ${VER} into Info.plist."
fi

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
