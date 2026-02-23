#!/bin/sh
#
# poof devcontainer feature install script
#
set -e

OWNER="pirafrank"
REPO="poof"
BIN_NAME="poof"
INSTALL_DIR="/usr/local/bin"

# VERSION is injected by the devcontainer feature runtime from the 'version' option.
# Default is 'latest' (set in devcontainer-feature.json).
VERSION="${VERSION:-latest}"

# Determine OS and Arch
OS="$(uname -s)"
ARCH="$(uname -m)"

# Normalize Arch
case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    i386|i586|i686) ARCH="i686" ;;
    armv7l|armv7) ARCH="armv7" ;;
    riscv64|riscv64gc) ARCH="riscv64gc" ;;
    ppc64le|powerpc64le) ARCH="powerpc64le" ;;
    s390x) ARCH="s390x" ;;
    loongarch64) ARCH="loongarch64" ;;
    *)
        echo "Error: Unsupported architecture $ARCH"
        exit 1
        ;;
esac

# Detect libc type on Linux (glibc vs musl)
LIBC_TYPE=""
if [ "$OS" = "Linux" ]; then
    LDD_OUTPUT=$(ldd --version 2>&1 || true)
    if echo "$LDD_OUTPUT" | grep -qi "glibc\|gnu libc"; then
        LIBC_TYPE="gnu"
    else
        LIBC_TYPE="musl"
    fi
fi

# Determine Target based on matrix.jsonc support
TARGET=""
case "$OS" in
    Linux)
        case "$ARCH" in
            powerpc64le|s390x)
                # Only glibc available for these architectures
                TARGET="${ARCH}-unknown-linux-gnu"
                ;;
            *)
                if [ "$LIBC_TYPE" = "gnu" ]; then
                    TARGET="${ARCH}-unknown-linux-gnu"
                else
                    TARGET="${ARCH}-unknown-linux-musl"
                fi
                ;;
        esac
        ;;
    Darwin)
        TARGET="${ARCH}-apple-darwin"
        ;;
    *)
        echo "Error: Unsupported OS $OS"
        exit 1
        ;;
esac

echo "Detected system: $OS $ARCH"
echo "Target: $TARGET"

# Check dependencies
if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required"
    exit 1
fi
if ! command -v tar >/dev/null 2>&1; then
    echo "Error: tar is required"
    exit 1
fi

# Resolve version and tag
if [ "$VERSION" = "latest" ]; then
    echo "Fetching latest release..."
    RELEASE_URL="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
    RELEASE_JSON=$(curl -sL "$RELEASE_URL")
    TAG_NAME=$(echo "$RELEASE_JSON" | grep -m 1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
else
    # Ensure the version is prefixed with 'v'
    case "$VERSION" in
        v*) TAG_NAME="$VERSION" ;;
        *)  TAG_NAME="v${VERSION}" ;;
    esac
    echo "Requested version: $TAG_NAME"
fi

# Strip 'v' prefix for filename construction
RELEASE_VERSION="${TAG_NAME#v}"

if [ -z "$TAG_NAME" ] || [ "$TAG_NAME" = "null" ]; then
    echo "Error: Could not determine release version."
    echo "Please check https://github.com/${OWNER}/${REPO}/releases for available versions."
    exit 1
fi

echo "Installing poof $TAG_NAME..."

# Construct filename matching release artifact format
FILENAME="${BIN_NAME}-${RELEASE_VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/${TAG_NAME}/${FILENAME}"

# Download and install
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading $DOWNLOAD_URL..."
if ! curl -fL "$DOWNLOAD_URL" -o "$TMP_DIR/$FILENAME"; then
    echo "Error: Download failed. Please check your internet connection and try again."
    exit 1
fi

echo "Extracting..."
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
    echo "Error: Binary not found in archive."
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "Successfully installed $BIN_NAME $TAG_NAME to $INSTALL_DIR/$BIN_NAME"
