#!/usr/bin/env bash
set -euo pipefail

REPO="jhonesis/dectl"
VERSION=""
INSTALL_DIR="${HOME}/.local/bin"
DRY_RUN=false
BUILD_FROM_SOURCE=false

usage() {
    cat <<EOF
Usage: curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.sh | bash

Options:
  --version vX.Y.Z    Install a specific version (default: latest)
  --to <dir>          Install to a custom directory (default: ~/.local/bin)
  --build             Force build from source instead of downloading
  --dry-run           Preview without installing
  --help              Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --to) INSTALL_DIR="$2"; shift 2 ;;
        --build) BUILD_FROM_SOURCE=true; shift ;;
        --dry-run) DRY_RUN=true; shift ;;
        --help) usage; exit 0 ;;
        *) echo "Unknown option: $1"; usage; exit 1 ;;
    esac
done

detect_target() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$os" in
        linux) os="unknown-linux-gnu" ;;
        darwin) os="apple-darwin" ;;
        *) echo "Unsupported OS: $os"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "Unsupported architecture: $arch"; exit 1 ;;
    esac

    echo "${arch}-${os}"
}

fetch_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": "\(.*\)",/\1/' 2>/dev/null || true
}

check_has_releases() {
    local code
    code=$(curl -fsSL -o /dev/null -w "%{http_code}" "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null || echo "000")
    [[ "$code" == "200" ]]
}

install_rust() {
    echo "→ Rust toolchain not found."

    if $DRY_RUN; then
        echo "[DRY-RUN] Would install Rust via rustup, then build from source"
        exit 0
    fi

    echo -n "→ Install Rust via rustup? [Y/n] "
    read -r CONFIRM
    if [[ "$CONFIRM" =~ ^[Nn] ]]; then
        echo "✗ Aborted. Install Rust manually:"
        echo "  https://rustup.rs"
        exit 1
    fi

    echo "→ Installing Rust (this may take a moment)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tail -3

    # Source the env so cargo is available immediately
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi

    if ! command -v cargo &>/dev/null; then
        echo "✗ Rust installation failed. Try manually: https://rustup.rs"
        exit 1
    fi

    echo "✓ Rust installed: $(cargo --version)"
}

install_from_source() {
    local tmpdir ver

    echo ""
    echo "→ Building from source..."

    if ! command -v cargo &>/dev/null; then
        install_rust
    fi

    echo "→ Cloning repository..."
    tmpdir="$(mktemp -d)"
    git clone --depth 1 "https://github.com/$REPO.git" "$tmpdir/dectl" 2>/dev/null || {
        echo "✗ Failed to clone repository. Check your internet connection."
        rm -rf "$tmpdir"
        exit 1
    }

    cd "$tmpdir/dectl"
    echo "→ Building (this may take a few minutes)..."
    cargo build --release -q 2>&1 | tail -5 || {
        echo "✗ Build failed."
        rm -rf "$tmpdir"
        exit 1
    }

    if $DRY_RUN; then
        echo ""
        echo "[DRY-RUN] Would install: $tmpdir/dectl/target/release/dectl → $INSTALL_DIR/dectl"
        rm -rf "$tmpdir"
        exit 0
    fi

    if [[ ! -d "$INSTALL_DIR" ]]; then
        mkdir -p "$INSTALL_DIR"
    fi

    cp "target/release/dectl" "$INSTALL_DIR/dectl"
    chmod +x "$INSTALL_DIR/dectl"
    rm -rf "$tmpdir"

    echo ""
    echo "✓ dectl built from source and installed to $INSTALL_DIR/dectl"
    "$INSTALL_DIR/dectl" version
}

main() {
    local target version url tarball

    target="$(detect_target)"
    echo "→ Detected target: $target"

    # Decide: download prebuilt binary or build from source?
    if $BUILD_FROM_SOURCE || ! check_has_releases; then
        if ! $BUILD_FROM_SOURCE && ! check_has_releases; then
            echo "→ No prebuilt release found. Falling back to source build."
        fi
        install_from_source
        exit 0
    fi

    # Download prebuilt binary
    if [[ -z "$VERSION" ]]; then
        echo "→ Fetching latest version..."
        VERSION="$(fetch_latest_version)"
        echo "→ Latest version: $VERSION"
    fi

    url="https://github.com/$REPO/releases/download/$VERSION/dectl-${VERSION#v}-${target}.tar.gz"
    tarball="dectl-${VERSION#v}-${target}.tar.gz"

    echo "→ Downloading: $url"
    echo "→ Install to: $INSTALL_DIR"

    if $DRY_RUN; then
        echo ""
        echo "[DRY-RUN] Would download: $url"
        echo "[DRY-RUN] Would extract dectl to: $INSTALL_DIR/dectl"
        exit 0
    fi

    curl -fsSL "$url" -o "/tmp/$tarball"

    echo "→ Extracting..."
    tar xzf "/tmp/$tarball" -C /tmp

    if [[ ! -d "$INSTALL_DIR" ]]; then
        mkdir -p "$INSTALL_DIR"
    fi

    cp "/tmp/dectl-${VERSION#v}-${target}/dectl" "$INSTALL_DIR/dectl"
    chmod +x "$INSTALL_DIR/dectl"
    rm -rf "/tmp/$tarball" "/tmp/dectl-${VERSION#v}-${target}"

    echo ""
    echo "✓ dectl $VERSION installed to $INSTALL_DIR/dectl"

    # Warn if install dir is not in PATH
    case ":${PATH:-}:" in
        *:"$INSTALL_DIR":*) ;;
        *) echo "⚠  $INSTALL_DIR is not in your PATH."
           echo "   Add it: export PATH=\"\$PATH:$INSTALL_DIR\""
           echo "   Or run: sudo ln -s $INSTALL_DIR/dectl /usr/local/bin/" ;;
    esac

    "$INSTALL_DIR/dectl" version
}

main
