#!/bin/sh
#
# poof install script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/pirafrank/poof/main/install.sh | sh
#
set -e

OWNER="pirafrank"
REPO="poof"
BIN_NAME="poof"
INSTALL_DIR="${HOME}/.local/bin"

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
    # Use ldd to detect libc
    LDD_OUTPUT=$(ldd --version 2>&1 || true)
    if echo "$LDD_OUTPUT" | grep -qi "glibc\|gnu libc"; then
        LIBC_TYPE="gnu"
    else
        # Fallback to musl
        LIBC_TYPE="musl"
    fi
fi

# Determine Target based on matrix.jsonc support
TARGET=""
case "$OS" in
    Linux)
        # Determine suffix based on architecture and libc availability
        case "$ARCH" in
            powerpc64le)
                # Only glibc available in matrix.jsonc
                TARGET="${ARCH}-unknown-linux-gnu"
                ;;
            s390x)
                # Only glibc available in matrix.jsonc
                TARGET="${ARCH}-unknown-linux-gnu"
                ;;
            *)
                # Other architectures have both glibc and musl
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

# Get latest version
echo "Fetching latest version..."
LATEST_URL="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
RELEASE_JSON=$(curl -sL "$LATEST_URL")
TAG_NAME=$(echo "$RELEASE_JSON" | grep -m 1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
# Remove 'v' prefix for filename construction
VERSION="${TAG_NAME#v}"

if [ -z "$TAG_NAME" ] || [ "$TAG_NAME" = "null" ]; then
    echo "Error: Could not determine latest release version."
    echo ""
    echo "Please download poof manually from https://github.com/${OWNER}/${REPO}/releases"
    echo "then move it to $INSTALL_DIR/$BIN_NAME:"
    echo "  mv poof-${VERSION}-${TARGET}.tar.gz $INSTALL_DIR/$BIN_NAME"
    echo "and make it executable:"
    echo "  chmod +x $INSTALL_DIR/$BIN_NAME"
    echo ""
    echo "then run the following command to add it to your PATH:"
    echo ""
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo "  source ~/.bashrc or ~/.zshrc"
    exit 1
fi

echo "Latest release: $TAG_NAME"

# Construct Filename
# Format matches release.yml: poof-<version>-<target>.tar.gz
FILENAME="${BIN_NAME}-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/${TAG_NAME}/${FILENAME}"

# Download and Install
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading $DOWNLOAD_URL..."
if ! curl -fL "$DOWNLOAD_URL" -o "$TMP_DIR/$FILENAME"; then
    echo "Error: Download failed. Please check your internet connection and try again."
    exit 1
fi

echo "Extracting..."
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

# Verify binary exists (it should be at root of archive)
if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
    echo "Error: Binary not found in archive."
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
if [ -f "$INSTALL_DIR/$BIN_NAME" ]; then
    echo "Removing existing binary..."
    rm "$INSTALL_DIR/$BIN_NAME"
fi
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "Successfully installed $BIN_NAME to $INSTALL_DIR/$BIN_NAME"
echo ""
echo "Run $BIN_NAME --help to get started."
echo ""

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "Warning: $INSTALL_DIR is not in your PATH."
        echo "To use $BIN_NAME, add the directory to your PATH:"
        echo ""
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "You can add this to your shell config (e.g., ~/.zshrc or ~/.bashrc)."
        ;;
esac

