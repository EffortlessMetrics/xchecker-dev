#!/bin/bash
# xchecker installer for Unix systems (Linux/macOS)
# Usage: curl -fsSL https://raw.githubusercontent.com/EffortlessMetrics/xchecker/main/scripts/install.sh | bash

set -euo pipefail

VERSION="${XCHECKER_VERSION:-}"
INSTALL_DIR="${XCHECKER_INSTALL_DIR:-$HOME/.local/bin}"
REPO="EffortlessMetrics/xchecker"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os-$arch" in
        linux-x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64) echo "aarch64-unknown-linux-gnu" ;;
        darwin-x86_64) echo "x86_64-apple-darwin" ;;
        darwin-arm64)  echo "aarch64-apple-darwin" ;;
        *)
            error "Unsupported platform: $os-$arch"
            ;;
    esac
}

# Get latest version from GitHub releases
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name":' \
        | sed -E 's/.*"v([^"]+)".*/\1/' \
        || error "Failed to fetch latest version"
}

# Main installation
main() {
    info "Installing xchecker..."

    # Detect platform
    local target
    target=$(detect_platform)
    info "Detected platform: $target"

    # Get version
    if [[ -z "$VERSION" ]]; then
        info "Fetching latest version..."
        VERSION=$(get_latest_version)
    fi
    info "Version: $VERSION"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    local url="https://github.com/$REPO/releases/download/v${VERSION}/xchecker-${target}.tar.gz"
    info "Downloading from: $url"

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    curl -fsSL "$url" -o "$tmp_dir/xchecker.tar.gz" \
        || error "Failed to download xchecker"

    # Verify checksum if available
    local checksum_url="${url}.sha256"
    if curl -fsSL "$checksum_url" -o "$tmp_dir/checksum.sha256" 2>/dev/null; then
        info "Verifying checksum..."
        (cd "$tmp_dir" && sha256sum -c checksum.sha256) \
            || error "Checksum verification failed"
    else
        warn "Checksum file not available, skipping verification"
    fi

    # Extract
    tar -xzf "$tmp_dir/xchecker.tar.gz" -C "$tmp_dir"

    # Install
    install -m 755 "$tmp_dir/xchecker" "$INSTALL_DIR/xchecker"

    info "xchecker installed to $INSTALL_DIR/xchecker"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi

    # Verify installation
    if "$INSTALL_DIR/xchecker" --version >/dev/null 2>&1; then
        info "Installation successful!"
        "$INSTALL_DIR/xchecker" --version
    else
        error "Installation verification failed"
    fi
}

main "$@"
