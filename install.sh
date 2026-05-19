#!/usr/bin/env bash
# ============================================================
# dectl installer — https://github.com/jhonesis/dectl
# Supports: Linux, macOS, WSL (Windows Subsystem for Linux)
# ============================================================
set -e

REPO="https://github.com/jhonesis/dectl.git"
INSTALL_DIR="${HOME}/.local/bin"
DECTL_SOURCE="${HOME}/.local/src/dectl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()   { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()   { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()  { echo -e "${RED}[ERROR]${NC} $1" >&2; }
step()   { echo -e "${BLUE}[STEP]${NC} $1"; }

# Banner
banner() {
    echo ""
    echo "  ██████╗ ██╗███████╗███████╗"
    echo "  ██╔══██╗██║██╔════╝██╔════╝"
    echo "  ██████╔╝██║███████╗███████╗"
    echo "  ██╔══██╗██║╚════██║╚════██║"
    echo "  ██████╔╝██║███████║███████║"
    echo "  ╚═════╝ ╚═╝╚══════╝╚══════╝"
    echo ""
    echo "  Dev Environment Control — Model-agnostic developer life OS"
    echo ""
}

# Platform detection
detect_platform() {
    local os="$(uname -s)"
    case "$os" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        *MINGW*|*CYGWIN*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# Check if running on WSL
is_wsl() {
    if grep -qiE 'microsoft|wsl' /proc/version 2>/dev/null; then
        return 0
    fi
    # Also check WSL_DISTRO_NAME env var
    if [ -n "$WSL_DISTRO_NAME" ]; then
        return 0
    fi
    return 1
}

# Check if command exists
has_command() {
    command -v "$1" &>/dev/null
}

# Check Rust installation
check_rust() {
    if has_command rustc; then
        info "Rust found: $(rustc --version 2>/dev/null | cut -d' ' -f1-2)"
        return 0
    fi
    return 1
}

# Install Rust via rustup
install_rust() {
    warn "Rust is not installed."

    local confirm
    echo ""
    read -p "Install Rust via rustup? This will download ~100MB. [y/N]: " confirm
    echo ""

    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        error "Rust is required to build dectl. Aborting installation."
        exit 1
    fi

    step "Installing Rust..."

    if has_command curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif has_command wget; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        error "Neither curl nor wget found. Cannot install Rust."
        exit 1
    fi

    # Source cargo env
    if [ -f "${HOME}/.cargo/env" ]; then
        source "${HOME}/.cargo/env"
    fi

    info "Rust installed: $(rustc --version | cut -d' ' -f1-2)"
}

# Check if dectl is already installed
check_existing() {
    if has_command dectl; then
        local version=$(dectl --version 2>/dev/null | head -1 || echo "unknown")
        info "dectl already installed: $version"
        return 0
    fi
    return 1
}

# Update existing installation
update_installation() {
    step "Updating dectl..."

    if [ ! -d "$DECTL_SOURCE" ]; then
        error "dectl source not found at $DECTL_SOURCE"
        error "Run installation without --update to install first."
        exit 1
    fi

    cd "$DECTL_SOURCE"

    # Fetch latest changes
    if has_command git; then
        info "Fetching latest changes..."
        git fetch origin main
        git pull origin main
    else
        error "git not found. Cannot update."
        exit 1
    fi

    # Rebuild
    step "Building dectl (this may take a few minutes on first run)..."
    cd dectl

    if ! cargo build --release 2>&1; then
        error "Build failed. Check errors above."
        exit 1
    fi

    # Reinstall
    cp target/release/dectl "$INSTALL_DIR/dectl"
    chmod +x "$INSTALL_DIR/dectl"

    info "dectl updated: $(dectl --version)"
    info "Restart your terminal or run 'hash -r' to update command hash."
}

# Main installation
install() {
    local platform=$(detect_platform)

    if [ "$platform" = "unknown" ]; then
        error "Unsupported platform: $(uname -s)"
        exit 1
    fi

    if is_wsl; then
        warn "Running on WSL (Windows Subsystem for Linux)"
        warn "dectl will work in your WSL terminal, but not in Windows cmd/PowerShell."
    fi

    info "Detected platform: $platform"

    # Check for cargo
    if ! has_command cargo; then
        install_rust
    fi

    # Verify cargo is available
    if ! has_command cargo; then
        error "cargo not available after installation. Please restart your terminal."
        exit 1
    fi

    # Create directories
    step "Setting up directories..."
    mkdir -p "$INSTALL_DIR"
    mkdir -p "${HOME}/.local/src"

    # Clone or update repository
    if [ -d "$DECTL_SOURCE" ]; then
        warn "dectl source directory found at $DECTL_SOURCE"
        read -p "Update existing installation? [y/N]: " confirm
        if [[ "$confirm" =~ ^[Yy]$ ]]; then
            cd "$DECTL_SOURCE"
            if has_command git; then
                git pull origin main 2>/dev/null || git pull origin main
            fi
        else
            info "Using existing source at $DECTL_SOURCE"
        fi
    else
        step "Cloning dectl from GitHub..."
        git clone "$REPO" "$DECTL_SOURCE"
        cd "$DECTL_SOURCE"
    fi

    # Build
    step "Building dectl (this may take a few minutes on first run)..."

    if [ ! -f "dectl/Cargo.toml" ]; then
        error "Invalid repository structure. Expected dectl/ directory."
        exit 1
    fi

    cd dectl

    if ! cargo build --release 2>&1; then
        error "Build failed. Check errors above."
        exit 1
    fi

    # Install binary
    step "Installing dectl..."
    cp target/release/dectl "$INSTALL_DIR/dectl"
    chmod +x "$INSTALL_DIR/dectl"

    info "dectl installed: $(dectl --version)"
    echo ""
    info "Add '$INSTALL_DIR' to your PATH if not already present:"
    info "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    info ""
    info "Run 'dectl --help' to get started."
    echo ""
}

# Check for updates
check_update() {
    if [ ! -d "$DECTL_SOURCE" ]; then
        return 1
    fi

    cd "$DECTL_SOURCE"

    if ! has_command git; then
        return 1
    fi

    git fetch origin main 2>/dev/null

    local local_hash=$(git rev-parse HEAD 2>/dev/null)
    local remote_hash=$(git rev-parse origin/main 2>/dev/null)

    if [ "$local_hash" != "$remote_hash" ]; then
        return 0
    fi
    return 1
}

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --update, -u    Update existing dectl installation"
    echo "  --check, -c     Check for updates"
    echo "  --help, -h      Show this help message"
    echo ""
    echo "Examples:"
    echo "  curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/install.sh | bash"
    echo "  curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/install.sh | bash -s -- --update"
}

# Main
main() {
    banner

    local update_mode=false
    local check_mode=false

    for arg in "$@"; do
        case $arg in
            --update|-u) update_mode=true ;;
            --check|-c)  check_mode=true ;;
            --help|-h)   usage; exit 0 ;;
        esac
    done

    if [ "$check_mode" = true ]; then
        if check_update; then
            info "Update available. Run with --update to install."
        else
            info "dectl is up to date."
        fi
        exit 0
    fi

    if [ "$update_mode" = true ]; then
        update_installation
    else
        if check_existing; then
            echo ""
            read -p "dectl is already installed. Update? [y/N]: " confirm
            if [[ "$confirm" =~ ^[Yy]$ ]]; then
                update_installation
            else
                info "Keeping existing installation."
            fi
        else
            install
        fi
    fi
}

main "$@"