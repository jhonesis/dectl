#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
DRY_RUN=false
NATIVE_ONLY=false

usage() {
    echo "Usage: $0 vX.Y.Z [--native] [--dry-run]"
    echo ""
    echo "  vX.Y.Z       Version tag (required)"
    echo "  --native     Build only for the native target (skip cross-compilation)"
    echo "  --dry-run    Preview without building or publishing"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --native) NATIVE_ONLY=true; shift ;;
        --dry-run) DRY_RUN=true; shift ;;
        v*) [[ -z "$VERSION" ]] && VERSION="$1" || usage; shift ;;
        *) usage ;;
    esac
done

if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    usage
fi

VERSION_NUM="${VERSION#v}"
RELEASE_DIR="$(mktemp -d)"
TARGETS=()

if $NATIVE_ONLY || ! command -v cargo-zigbuild &>/dev/null; then
    TARGETS+=( "$(rustc -vV | grep host | awk '{print $2}')" )
    if ! $NATIVE_ONLY && ! command -v cargo-zigbuild &>/dev/null; then
        echo "  Warning: cargo-zigbuild not found. Building only for native target."
        echo "  Install: cargo install cargo-zigbuild"
    fi
else
    TARGETS=(
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
    )
fi

echo "==> dectl $VERSION release"
echo "    Targets: ${TARGETS[*]}"
echo ""

for TARGET in "${TARGETS[@]}"; do
    echo "    Building for $TARGET..."

    if ! $DRY_RUN; then
        if command -v cargo-zigbuild &>/dev/null; then
            cargo zigbuild --release --target "$TARGET"
        else
            cargo build --release --target "$TARGET"
        fi

        BIN_NAME="dectl-${VERSION_NUM}-${TARGET}"
        ARCHIVE_DIR="${RELEASE_DIR}/${BIN_NAME}"
        mkdir -p "$ARCHIVE_DIR"
        cp "target/$TARGET/release/dectl" "$ARCHIVE_DIR/"
        cp README.md "$ARCHIVE_DIR/" 2>/dev/null || true
        cp LICENSE "$ARCHIVE_DIR/" 2>/dev/null || true

        tar czf "${RELEASE_DIR}/${BIN_NAME}.tar.gz" -C "$RELEASE_DIR" "$BIN_NAME"
        echo "    -> ${BIN_NAME}.tar.gz"
        rm -rf "$ARCHIVE_DIR"
    else
        echo "    [DRY-RUN] Would build for $TARGET"
    fi
done

if ! $DRY_RUN; then
    echo ""
    echo "==> Generating checksums..."
    if command -v sha256sum &>/dev/null; then
        sha256sum "${RELEASE_DIR}"/*.tar.gz > "${RELEASE_DIR}/SHA256SUMS.txt"
    else
        shasum -a 256 "${RELEASE_DIR}"/*.tar.gz > "${RELEASE_DIR}/SHA256SUMS.txt"
    fi
    echo "    -> SHA256SUMS.txt"
fi

echo ""
echo "==> Release artifacts:"
if $DRY_RUN; then
    echo "    (dry run — no artifacts produced)"
else
    ls -lh "$RELEASE_DIR"/*.tar.gz "$RELEASE_DIR"/SHA256SUMS.txt
fi

echo ""
if command -v gh &>/dev/null && ! $DRY_RUN; then
    echo "==> Creating GitHub release..."
    echo "    gh release create $VERSION \\"
    echo "        --title \"dectl $VERSION\" \\"
    echo "        --notes \"See CHANGELOG for details.\" \\"
    echo "        \"$RELEASE_DIR\"/*.tar.gz \"$RELEASE_DIR\"/SHA256SUMS.txt"
    echo ""
    echo -n "    Proceed? [Y/n] "
    read -r CONFIRM
    if [[ "$CONFIRM" =~ ^[Nn] ]]; then
        echo "    Skipped. Artifacts at: $RELEASE_DIR"
    else
        gh release create "$VERSION" \
            --title "dectl $VERSION" \
            --notes "See CHANGELOG for details." \
            "$RELEASE_DIR"/*.tar.gz "$RELEASE_DIR"/SHA256SUMS.txt
        echo "    -> Published: https://github.com/jhonesis/dectl/releases/tag/$VERSION"
    fi
elif ! $DRY_RUN; then
    echo "==> To publish:"
    echo "    gh release create $VERSION \\"
    echo "        --title \"dectl $VERSION\" \\"
    echo "        \"$RELEASE_DIR\"/*.tar.gz \"$RELEASE_DIR\"/SHA256SUMS.txt"
    echo ""
    echo "    Artifacts at: $RELEASE_DIR"
fi
