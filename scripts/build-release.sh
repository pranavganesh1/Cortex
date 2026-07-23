#!/bin/bash
set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
TARGET_DIR="dist"
mkdir -p "$TARGET_DIR"

echo "🔨 Building Cortex v$VERSION..."

# Native build
cargo build --release

# Package native binary
cp target/release/cortex "$TARGET_DIR/cortex-$VERSION-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"

echo "✅ Native build complete"

# Cross-compilation (if you have the targets installed)
# rustup target add x86_64-unknown-linux-musl
# rustup target add aarch64-apple-darwin
# rustup target add x86_64-pc-windows-gnu

# cargo build --release --target x86_64-unknown-linux-musl
# cargo build --release --target aarch64-apple-darwin
# cargo build --release --target x86_64-pc-windows-gnu

echo "📦 Packaged to $TARGET_DIR/"
ls -la "$TARGET_DIR/"
