default := "help"

fmt:
    cargo fmt

lint:
    cargo clippy -- -D warnings

test:
    cargo test

build:
    cargo build --release

build-fast:
    cargo build

check:
    cargo check

clean:
    cargo clean

help:
    @echo "Available targets:"
    @echo "  fmt        - Format code with rustfmt"
    @echo "  lint       - Run clippy with deny warnings"
    @echo "  test       - Run tests"
    @echo "  build      - Release build"
    @echo "  build-fast - Debug build"
    @echo "  check      - Type check without building"
    @echo "  clean      - Remove build artifacts"