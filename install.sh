#!/usr/bin/env bash
#
# Lexito installer / updater.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/gdarko/lexito/main/install.sh | bash
#
set -euo pipefail

REPO="gdarko/lexito"
APP_NAME="Lexito"
TMP_DIR=""
trap 'if [[ -n "$TMP_DIR" ]]; then rm -rf "$TMP_DIR"; fi' EXIT

# ── Detect platform ──────────────────────────────────────────────────

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  OS="linux" ;;
        Darwin) OS="macos" ;;
        *)      echo "Error: unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             echo "Error: unsupported architecture: $arch" >&2; exit 1 ;;
    esac
}

# ── Fetch latest release tag ─────────────────────────────────────────

fetch_latest_version() {
    LATEST_VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"

    if [[ -z "$LATEST_VERSION" ]]; then
        echo "Error: could not determine latest version." >&2
        exit 1
    fi
}

# ── Installed version detection ──────────────────────────────────────

get_installed_version() {
    if [[ "$OS" == "linux" ]]; then
        local version_file="${HOME}/.local/share/lexito/version"
        if [[ -f "$version_file" ]]; then
            cat "$version_file"
        fi
    else
        local app_path=""
        if [[ -d "/Applications/Lexito.app" ]]; then
            app_path="/Applications/Lexito.app"
        elif [[ -d "${HOME}/Applications/Lexito.app" ]]; then
            app_path="${HOME}/Applications/Lexito.app"
        fi
        if [[ -n "$app_path" ]]; then
            local ver
            ver="$(/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" \
                "$app_path/Contents/Info.plist" 2>/dev/null || true)"
            if [[ -n "$ver" ]]; then
                echo "v${ver}"
            fi
        fi
    fi
}

check_installed_version() {
    local installed
    installed="$(get_installed_version)"

    if [[ "$installed" == "$LATEST_VERSION" ]]; then
        echo "${APP_NAME} ${LATEST_VERSION} is already installed and up to date."
        exit 0
    fi

    if [[ -n "$installed" ]]; then
        echo "${APP_NAME} ${installed} is currently installed. Latest is ${LATEST_VERSION}."
        printf "Upgrade? [Y/n] "
        read -r answer </dev/tty
        case "$answer" in
            [nN]*) echo "Cancelled."; exit 0 ;;
        esac
    fi
}

# ── Linux install ────────────────────────────────────────────────────

install_linux() {
    local download_url
    TMP_DIR="$(mktemp -d)"

    download_url="https://github.com/${REPO}/releases/download/${LATEST_VERSION}/lexito-${LATEST_VERSION}-linux-${ARCH}.tar.gz"

    echo "Downloading lexito-${LATEST_VERSION}-linux-${ARCH}.tar.gz..."
    curl -fsSL "$download_url" -o "$TMP_DIR/lexito.tar.gz"
    tar xzf "$TMP_DIR/lexito.tar.gz" -C "$TMP_DIR"

    # Binary
    mkdir -p "${HOME}/.local/bin"
    install -m 755 "$TMP_DIR/lexito" "${HOME}/.local/bin/lexito"
    echo "  Installed binary to ~/.local/bin/lexito"

    # Icon (SVG from repo at the release tag)
    local icon_dir="${HOME}/.local/share/icons/hicolor/scalable/apps"
    mkdir -p "$icon_dir"
    curl -fsSL "https://raw.githubusercontent.com/${REPO}/${LATEST_VERSION}/crates/desktop/assets/icon.svg" \
        -o "$icon_dir/lexito.svg"
    echo "  Installed icon to ${icon_dir}/lexito.svg"

    # Desktop entry
    local desktop_dir="${HOME}/.local/share/applications"
    mkdir -p "$desktop_dir"
    cat > "$desktop_dir/lexito.desktop" << 'DESKTOP'
[Desktop Entry]
Type=Application
Name=Lexito
Comment=Desktop gettext translator with AI-powered translation
Exec=lexito
Icon=lexito
Categories=Development;Translation;
Terminal=false
StartupWMClass=lexito
DESKTOP
    echo "  Installed desktop entry to ${desktop_dir}/lexito.desktop"

    # Version marker
    local data_dir="${HOME}/.local/share/lexito"
    mkdir -p "$data_dir"
    echo "$LATEST_VERSION" > "$data_dir/version"

    # PATH check
    case ":$PATH:" in
        *":${HOME}/.local/bin:"*) ;;
        *)
            echo ""
            echo "NOTE: ~/.local/bin is not in your PATH."
            echo "Add it with:  export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
    esac
}

# ── macOS install ────────────────────────────────────────────────────

install_macos() {
    local download_url install_dir
    TMP_DIR="$(mktemp -d)"

    download_url="https://github.com/${REPO}/releases/download/${LATEST_VERSION}/lexito-${LATEST_VERSION}-macos-${ARCH}.tar.gz"

    echo "Downloading lexito-${LATEST_VERSION}-macos-${ARCH}.tar.gz..."
    curl -fsSL "$download_url" -o "$TMP_DIR/lexito.tar.gz"
    tar xzf "$TMP_DIR/lexito.tar.gz" -C "$TMP_DIR"

    install_dir="/Applications"
    if [[ ! -w "$install_dir" ]]; then
        install_dir="${HOME}/Applications"
        mkdir -p "$install_dir"
    fi

    rm -rf "${install_dir}/Lexito.app"
    cp -R "$TMP_DIR/Lexito.app" "${install_dir}/Lexito.app"

    # Clear quarantine (binary is not notarized)
    xattr -rd com.apple.quarantine "${install_dir}/Lexito.app" 2>/dev/null || true

    echo "  Installed Lexito.app to ${install_dir}/"
}

# ── Main ─────────────────────────────────────────────────────────────

main() {
    echo "Lexito installer"
    echo ""

    detect_platform
    echo "Platform: ${OS} ${ARCH}"

    fetch_latest_version
    echo "Latest:   ${LATEST_VERSION}"
    echo ""

    check_installed_version

    if [[ "$OS" == "linux" ]]; then
        install_linux
    else
        install_macos
    fi

    echo ""
    echo "${APP_NAME} ${LATEST_VERSION} installed successfully!"
}

main
