#!/bin/bash

set -e

# Store the script path for later deletion
SCRIPT_PATH=$(readlink -f "$0")

# Function to extract version number from tcpping --version output
get_version() {
    tcpping --version 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+'
}

# Check current installation and version
CURRENT_VERSION=""
OPERATION="installed"
if tcpping --version >/dev/null 2>&1; then
    CURRENT_VERSION=$(get_version)
fi

# Check if the user is root or using sudo
if [ "$EUID" -ne 0 ]; then
    echo "Please use sudo or run as root."
    exit 1
fi

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
        echo "Unsupported architecture: $ARCH."
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
    echo "Failed to get latest version number."
    exit 1
fi

# Build download URL
DOWNLOAD_URL="https://github.com/ReyReyy/tcpping/releases/download/${LATEST_VERSION}/tcpping-${LATEST_VERSION}-${OS}-${ARCH}.tar.gz"

# Create temporary directory
TEMP_DIR=$(mktemp -d -t tcpping-XXXXXX)
cd "$TEMP_DIR"

# Download file
if ! curl -L -s -o tcpping.tar.gz "$DOWNLOAD_URL"; then
    echo "Download failed."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Unzip file
if ! tar -xzf tcpping.tar.gz; then
    echo "Unzip failed."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Check if the file exists
if [ ! -f tcpping ]; then
    echo "tcpping file does not exist."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Check if /usr/local/bin directory exists, create if not
if [ ! -d /usr/local/bin ]; then
    sudo mkdir -p /usr/local/bin
fi

# Move executable file to /usr/local/bin/
if ! sudo mv tcpping /usr/local/bin/; then
    echo "Move file failed."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Give execute permission
sudo chmod +x /usr/local/bin/tcpping

# Clean up temporary files
cd /
rm -rf "$TEMP_DIR"

# Verify installation and determine operation type
NEW_VERSION=$(get_version)
if [ -z "$NEW_VERSION" ]; then
    echo "Installation failed: tcpping is not working properly."
    exit 1
fi

if [ -n "$CURRENT_VERSION" ]; then
    if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
        OPERATION="reinstalled"
    else
        OPERATION="upgraded"
    fi
fi

echo "tcpping has been ${OPERATION} successfully!"
tcpping --version

# Delete script itself
rm -f "$SCRIPT_PATH"