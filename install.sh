#!/bin/bash
set -e

REPO="yourusername/cortex"
BINARY_NAME="cortex"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
    linux) PLATFORM="linux" ;;
    darwin) PLATFORM="macos" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

VERSION="${VERSION:-latest}"
if [ "$VERSION" = "latest" ]; then
    VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
fi

FILENAME="${BINARY_NAME}-${VERSION}-${PLATFORM}-${ARCH}"
URL="https://github.com/$REPO/releases/download/$VERSION/$FILENAME"

echo "📥 Installing Cortex $VERSION for $PLATFORM/$ARCH..."
echo "   From: $URL"

mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -sSL "$URL" -o "$INSTALL_DIR/$BINARY_NAME"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$INSTALL_DIR/$BINARY_NAME"
else
    echo "❌ Need curl or wget"
    exit 1
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_RC=""
    if [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    fi
    
    if [ -n "$SHELL_RC" ]; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_RC"
        echo "✅ Added $INSTALL_DIR to PATH in $SHELL_RC"
        echo "   Run: source $SHELL_RC"
    fi
fi

echo ""
echo "🧠 Cortex installed!"
echo "   Binary: $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "Quick start:"
echo "   1. cortex ingest --path ./my-project"
echo "   2. cortex watch --path ./my-project"
echo "   3. cortex ask \"what does this do?\""
echo ""
echo "For semantic search, install Ollama:"
echo "   https://ollama.com → ollama pull nomic-embed-text"
