#!/bin/bash

set -e

# Get OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture to the name used in GitHub release page
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH" 
        exit 1
        ;;
esac

# Fix macOS system name
if [ "$OS" = "darwin" ]; then
    OS="macos"
fi

# Get latest version number
LATEST_VERSION=$(curl -s https://api.github.com/repos/ReyReyy/tcpping/releases/latest | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p')

if [ -z "$LATEST_VERSION" ]; then
    echo "Failed to get latest version number"
    exit 1
fi

# Build download URL
DOWNLOAD_URL="https://github.com/ReyReyy/tcpping/releases/download/${LATEST_VERSION}/tcpping-${LATEST_VERSION}-${OS}-${ARCH}.tar.gz"

# Create temporary directory
TEMP_DIR=$(mktemp -d -t tcpping-XXXXXX)
cd "$TEMP_DIR"

# Download file
if ! curl -L -s -o tcpping.tar.gz "$DOWNLOAD_URL"; then
    echo "Download failed"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Unzip file
if ! tar -xzf tcpping.tar.gz; then
    echo "Unzip failed"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Check if the file exists
if [ ! -f tcpping ]; then
    echo "tcpping file does not exist"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Move executable file to /usr/local/bin/
if ! sudo mv tcpping /usr/local/bin/; then
    echo "Move file failed"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Give execute permission
sudo chmod +x /usr/local/bin/tcpping

# Clean up temporary files
cd /
rm -rf "$TEMP_DIR"

echo "tcpping has been installed successfully"

# Delete script itself
rm -f "$0"