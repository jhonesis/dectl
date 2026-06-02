#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Usage: $0 vX.Y.Z"
    exit 1
fi

VERSION_NUM="${VERSION#v}"
RELEASE_DIR="$(mktemp -d)"
TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
)

echo "==> Building dectl $VERSION"

BUILD_CMD="cargo build --release"
if command -v cargo-zigbuild &>/dev/null; then
    BUILD_CMD="cargo zigbuild --release"
    echo "    Using cargo-zigbuild for cross-compilation"
else
    echo "    Warning: cargo-zigbuild not found; building only for native target"
    TARGETS=( "$(rustc -vV | grep host | awk '{print $2}')" )
fi

for TARGET in "${TARGETS[@]}"; do
    echo "    Building for $TARGET..."
    $BUILD_CMD --target "$TARGET"

    BIN_NAME="dectl-${VERSION_NUM}-${TARGET}"
    ARCHIVE_DIR="${RELEASE_DIR}/${BIN_NAME}"
    mkdir -p "$ARCHIVE_DIR"

    cp "target/$TARGET/release/dectl" "$ARCHIVE_DIR/"
    cp README.md "$ARCHIVE_DIR/" 2>/dev/null || true
    cp LICENSE "$ARCHIVE_DIR/" 2>/dev/null || true

    cd "$RELEASE_DIR"
    tar czf "${BIN_NAME}.tar.gz" "$BIN_NAME"
    cd - >/dev/null

    echo "    -> ${BIN_NAME}.tar.gz"
done

cd "$RELEASE_DIR"
echo "==> Generating checksums..."
sha256sum *.tar.gz > SHA256SUMS.txt 2>/dev/null || shasum -a 256 *.tar.gz > SHA256SUMS.txt
echo "    -> SHA256SUMS.txt"
cd - >/dev/null

echo ""
echo "==> Release artifacts in: $RELEASE_DIR"
ls -lh "$RELEASE_DIR"/*.tar.gz "$RELEASE_DIR"/SHA256SUMS.txt

if command -v gh &>/dev/null; then
    echo ""
    echo "==> Creating GitHub release..."
    gh release create "$VERSION" \
        --title "dectl $VERSION" \
        --notes "See [CHANGELOG](../../CHANGELOG.md) for details." \
        "$RELEASE_DIR"/*.tar.gz "$RELEASE_DIR"/SHA256SUMS.txt
    echo "    -> Published: https://github.com/jhonesis/dectl/releases/tag/$VERSION"
else
    echo ""
    echo "==> gh not installed. To publish:"
    echo "    gh release create $VERSION --title \"dectl $VERSION\" $RELEASE_DIR/*.tar.gz $RELEASE_DIR/SHA256SUMS.txt"
fi
