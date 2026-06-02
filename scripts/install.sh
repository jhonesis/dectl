#!/usr/bin/env bash
set -euo pipefail

REPO="jhonesis/dectl"
VERSION=""
INSTALL_DIR="/usr/local/bin"
DRY_RUN=false

usage() {
    cat <<EOF
Usage: curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.sh | bash

Options:
  --version vX.Y.Z    Install a specific version (default: latest)
  --to <dir>          Install to a custom directory (default: /usr/local/bin)
  --dry-run           Preview without installing
  --help              Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --to) INSTALL_DIR="$2"; shift 2 ;;
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
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": "\(.*\)",/\1/'
}

main() {
    local target version url tarball

    target="$(detect_target)"
    echo "→ Detected target: $target"

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
    "$INSTALL_DIR/dectl" version
}

main
