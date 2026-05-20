#!/usr/bin/env bash
# ============================================================
# dectl installer — https://github.com/jhonesis/dectl
# Usage: curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/install.sh | bash
# ============================================================
set -e

REPO="jhonesis/dectl"
RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
INSTALL_DIR="${HOME}/.local/bin"
TMP_DIR=$(mktemp -d)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()   { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()   { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()  { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Platform detection
detect_platform() {
    local os="$(uname -s)"
    local arch="$(uname -m)"
    
    case "$os" in
        Linux*)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-musl" ;;
                aarch64) echo "aarch64-unknown-linux-musl" ;;
                armv7l)  echo "arm-unknown-linux-musleabihf" ;;
                *)       echo "unknown" ;;
            esac
            ;;
        Darwin*)
            case "$arch" in
                x86_64)  echo "x86_64-apple-darwin" ;;
                arm64)   echo "aarch64-apple-darwin" ;;
                *)       echo "unknown" ;;
            esac
            ;;
        *MINGW*|*CYGWIN*|*MSYS*)
            echo "x86_64-pc-windows-msvc"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Download file with fallback
download() {
    local url=$1
    local dest=$2
    
    if command -v curl &>/dev/null; then
        curl -fSL "$url" -o "$dest" || return 1
    elif command -v wget &>/dev/null; then
        wget -qO "$dest" "$url" || return 1
    else
        error "Neither curl nor wget found"
        return 1
    fi
}

# Get download URL for platform
get_download_url() {
    local platform=$1
    local json=$(curl -sfSL "$RELEASE_URL" 2>/dev/null) || {
        error "Failed to fetch release info"
        return 1
    }
    
    local name
    case "$platform" in
        x86_64-unknown-linux*)  name="dectl-linux-x86_64" ;;
        aarch64-apple-darwin)   name="dectl-macos-arm64" ;;
        x86_64-apple-darwin)   name="dectl-macos-x86_64" ;;
        *)                      name="dectl-${platform}" ;;
    esac
    
    # Try to find matching asset
    echo "$json" | grep -o "\"browser_download_url\": *\"\([^\"]*${name}[^\"]*\)\"" | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/' || echo "$json" | grep -o "\"browser_download_url\": *\"\([^\"]*dectl[^\"]*\)\"" | grep -i "$(uname -s | tr '[:upper:]' '[:lower:]')" | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/'
}

# Main installation
install() {
    local platform=$(detect_platform)
    
    if [ "$platform" = "unknown" ]; then
        error "Unsupported platform: $(uname -s)/$(uname -m)"
        exit 1
    fi
    
    info "Detected platform: $platform"
    
    # Get latest version
    local version=$(curl -sfSL "$RELEASE_URL" 2>/dev/null | grep '"tag_name"' | sed 's/.*"v\?\([^"]*\)".*/\1/' || echo "latest")
    info "Installing dectl $version..."
    
    # Get download URL
    local download_url=$(get_download_url "$platform")
    
    if [ -z "$download_url" ]; then
        error "No pre-built binary for $platform"
        info "Try building from source: https://github.com/${REPO}#building-from-source"
        exit 1
    fi
    
    info "Downloading from: $download_url"
    
    # Download binary
    local binary="${TMP_DIR}/dectl"
    if ! download "$download_url" "$binary"; then
        error "Download failed"
        exit 1
    fi
    
    # Install
    mkdir -p "$INSTALL_DIR"
    mv "$binary" "$INSTALL_DIR/dectl"
    chmod +x "$INSTALL_DIR/dectl"
    
    # Cleanup
    rm -rf "$TMP_DIR"
    
    info "dectl installed: $(dectl --version 2>/dev/null | head -1 || echo "installed")"
    echo ""
    info "Add '$INSTALL_DIR' to your PATH:"
    info "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
}

# Update
update() {
    install
}

# Check version
check() {
    local current=$(dectl --version 2>/dev/null | head -1 || echo "not installed")
    local latest=$(curl -sfSL "$RELEASE_URL" 2>/dev/null | grep '"tag_name"' | sed 's/.*"v\?\([^"]*\)".*/\1/' || echo "unknown")
    
    info "Current: $current"
    info "Latest:  $latest"
    
    if [ "$current" != "$latest" ] && [ "$latest" != "unknown" ]; then
        echo ""
        info "Update available. Run: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash"
    fi
}

# Usage
usage() {
    echo "Usage: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --update, -u    Update to latest version"
    echo "  --check, -c     Check for updates"
    echo "  --help, -h      Show this help"
    echo ""
    echo "Examples:"
    echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash"
    echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash -s -- --update"
}

# Main
main() {
    local mode="install"
    
    for arg in "$@"; do
        case $arg in
            --update|-u)  mode="update" ;;
            --check|-c)   mode="check" ;;
            --help|-h)    usage; exit 0 ;;
        esac
    done
    
    case $mode in
        install) install ;;
        update)  update ;;
        check)   check ;;
    esac
}

main "$@"