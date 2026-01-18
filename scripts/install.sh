#!/bin/sh
set -e

# --- Configuration ---
GITHUB_REPO="suwakei/golta"
BINARY_NAME="golta"
INSTALL_DIR="$HOME/.golta/bin"
# ---------------------

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)     OS_TYPE="linux"; EXT=".tar.gz" ;;
    Darwin*)    OS_TYPE="macos"; EXT=".tar.gz" ;;
    MINGW*|MSYS*) OS_TYPE="windows"; EXT=".zip" ;;
    *)          echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64) ARCH_TYPE="amd64" ;;
    arm64|aarch64) ARCH_TYPE="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

echo "Installing $BINARY_NAME latest for $OS_TYPE-$ARCH_TYPE..."

# Construct download URL
ASSET_NAME="${BINARY_NAME}-${OS_TYPE}-${ARCH_TYPE}${EXT}"
DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/latest/download/$ASSET_NAME"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
TEMP_DIR=$(mktemp -d)
echo "Downloading $DOWNLOAD_URL..."
curl -L --fail "$DOWNLOAD_URL" -o "$TEMP_DIR/$ASSET_NAME"

if [ "$OS_TYPE" = "windows" ]; then
    unzip -o "$TEMP_DIR/$ASSET_NAME" -d "$INSTALL_DIR"
else
    tar -xzf "$TEMP_DIR/$ASSET_NAME" -C "$INSTALL_DIR"
fi

rm -rf "$TEMP_DIR"

echo "Installed to $INSTALL_DIR"

# Configure PATH (Add environment variable)
SHELL_CONFIG=""
case "$SHELL" in
    */zsh) SHELL_CONFIG="$HOME/.zshrc" ;;
    */bash) SHELL_CONFIG="$HOME/.bashrc" ;;
    *) echo "Warning: Could not detect shell config file. Please add $INSTALL_DIR to your PATH manually." ;;
esac

if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG"; then
        echo "" >> "$SHELL_CONFIG"
        echo "# Added by $BINARY_NAME installer" >> "$SHELL_CONFIG"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
        echo "Added $INSTALL_DIR to $SHELL_CONFIG."
        echo "To start using $BINARY_NAME immediately, run:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
fi

# Try to update current session PATH (works if script is sourced)
export PATH="$INSTALL_DIR:$PATH"