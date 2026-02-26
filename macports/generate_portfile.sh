#!/bin/bash

set -euo pipefail

### Generate a Portfile for poof and test it locally.
### Run with: ./generate_portfile.sh <version>

# --- Configuration ---
NAME="poof"
GH_USER="pirafrank"
REPO="poof"
CATEGORY="sysutils"
MAINTAINER_NAME="@pirafrank"
MAINTAINER_EMAIL="fpira.com:dev"
OPENMAINTAINER="true"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_PORTS_PATH="$HOME/pirafrank/ports"

# --- Version and Source ---
VERSION=${1:-$(git tag  | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' | sort -rV | head -n 1 | sed 's/v//')}
SRC_TARBALL="${NAME}-v${VERSION}.tar.gz"
MAN_PAGE="${NAME}.1"

SRC_URL="https://github.com/$GH_USER/$REPO/archive/refs/tags/v${VERSION}.tar.gz"
MAN_PAGE_URL="https://github.com/$GH_USER/$REPO/releases/download/v$VERSION/$MAN_PAGE"

# --- Helper functions ---

# 3. Helper functions
get_checksums() {
    local url=$1
    local file=$2
    curl -fsSL "$url" -o "$file"
    local size
    local sha256
    local rmd160
    size=$(stat -f%z "$file")
    sha256=$(shasum -a 256 "$file" | awk '{print $1}')
    rmd160=$(openssl dgst -rmd160 "$file" | awk '{print $2}')
    echo "$file $rmd160 $sha256 $size"
    # We keep the file briefly for cargo-generate-portfile if it's the source
}

extract_cargo_lock() {
    local tarball=$1
    local temp_dir
    temp_dir=$(mktemp -d)
    tar -xzf "$tarball" -C "$temp_dir" --strip-components=1
    if [ -f "$temp_dir/Cargo.lock" ]; then
        cp "$temp_dir/Cargo.lock" "$SCRIPT_DIR/Cargo.lock"
    else
        echo "❌ Error: Cargo.lock not found in the source tarball."
        exit 1
    fi
    rm -rf "$temp_dir"
}

cleanup_local_dir() {
    rm -f "$SCRIPT_DIR/poof.1"
    rm -f "$SCRIPT_DIR"/*.tar.gz
    rm -f "$SCRIPT_DIR/Portfile"
    rm -f "$SCRIPT_DIR/Cargo.lock"
}

cleanup_local_tree() {
    local TARGET_DIR="$LOCAL_PORTS_PATH/$CATEGORY/$NAME"
    if [ -d "$TARGET_DIR" ]; then
        echo "🧹 Cleaning up old Portfile in $TARGET_DIR..."
        rm -rf "$TARGET_DIR"
    fi
    mkdir -p "$TARGET_DIR"
}

# --- Main Execution ---

# validate version
if [ -z "$VERSION" ]; then
    echo "❌ Error: Could not determine version. Provide one explicitly."
    exit 1
fi

# 1. Cleanup local dir and prepare
echo "🧹 Cleaning up local directory..."
cleanup_local_dir
echo "🚀 Preparing Source + Manpage Portfile for $NAME v$VERSION..."

# 2. Fetch checksums
echo "📥 Fetching Source & Asset for checksumming..."
read -r -a SRC_CHKS <<< "$(get_checksums "$SRC_URL" "$SCRIPT_DIR/$SRC_TARBALL")"
read -r -a MAN_CHKS <<< "$(get_checksums "$MAN_PAGE_URL" "$SCRIPT_DIR/$MAN_PAGE")"

# 4. Generate Cargo Dependency Block
echo "📦 Generating Cargo crate list from Cargo.lock..."
extract_cargo_lock "$SCRIPT_DIR/$SRC_TARBALL"
# Check for Cargo.lock
if [ ! -f "$SCRIPT_DIR/Cargo.lock" ]; then
    echo "❌ Error: Cargo.lock not found. Cannot generate crate list."
    exit 1
fi
if ! command -v cargo2ports &> /dev/null; then
    echo "❌ Error: cargo2ports is not installed. Please install it to generate the Cargo dependency block."
    exit 1
fi
# Parse Cargo.lock to extract crate names and versions
CRATES_BLOCK=$(cargo2ports "$SCRIPT_DIR/Cargo.lock")

# 5. Write the Portfile
cat <<EOF > "$SCRIPT_DIR/Portfile"
# -*- coding: utf-8; mode: tcl; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- vim:fenc=utf-8:ft=tcl:et:sw=4:ts=4:sts=4

PortSystem          1.0
PortGroup           cargo 1.0

name                $NAME
version             $VERSION
categories          $CATEGORY
license             MIT
homepage            https://poof.fpira.com
maintainers         {${MAINTAINER_EMAIL} ${MAINTAINER_NAME}}$([ "${OPENMAINTAINER}" = "true" ] && printf " openmaintainer")

description         Magic package manager of pre-built software.
long_description    Install and manage awesome tools from GitHub Releases\
 in one command. No manifests, formulae, ports, or repositories required.

master_sites        https://github.com/$GH_USER/$REPO/archive/refs/tags/v\${version}/:source \\
                    https://github.com/$GH_USER/$REPO/releases/download/v\${version}/poof.1?dummy=\${version}:asset

distname            \${name}-\${version}
distfiles           \${distname}.tar.gz:source \\
                    \${name}-\${version}.1:asset

# Only extract the main source tarball, leave the man page file as-is in
extract.only        \${distname}.tar.gz

checksums           \${distname}.tar.gz \\
                    rmd160  "${SRC_CHKS[1]}" \\
                    sha256  "${SRC_CHKS[2]}" \\
                    size    "${SRC_CHKS[3]}" \\
                    \${name}-\${version}.1 \\
                    rmd160  "${MAN_CHKS[1]}" \\
                    sha256  "${MAN_CHKS[2]}" \\
                    size    "${MAN_CHKS[3]}"

${CRATES_BLOCK}

# The 'cargo' PortGroup builds the binary; we explicitly install it and the man page.
destroot {
    xinstall -d \${destroot}\${prefix}/share/man/man1
    xinstall -m 0755 \${worksrcpath}/target/[option triplet.\${muniversal.build_arch}]/release/\${name} \${destroot}\${prefix}/bin/
    xinstall -m 0644 \${distpath}/\${name}-\${version}.1 \${destroot}\${prefix}/share/man/man1/\${name}.1
}
EOF

# Cleanup local tree before copying the new Portfile to local ports directory
cleanup_local_tree
mkdir -p "$LOCAL_PORTS_PATH/$CATEGORY/$NAME"
cp "$SCRIPT_DIR/Portfile" "$LOCAL_PORTS_PATH/$CATEGORY/$NAME/Portfile"
# Update permissions for the new Portfile
sudo find "$LOCAL_PORTS_PATH" -type d -exec chmod 755 {} +
sudo find "$LOCAL_PORTS_PATH" -type f -exec chmod 644 {} +
# Note: user:macports is the default group for MacPorts,
#       it really means the user named 'macports', not the group.
#       This user is created during MacPorts installation and
#       is used for file permissions.
sudo chmod +a "user:macports allow search" "/Users/$USER"
sudo chmod +a "user:macports allow search" "/Users/$USER/pirafrank"
sudo chmod +a "user:macports allow search" "/Users/$USER/pirafrank/ports"
echo "✅ Portfile is ready at $LOCAL_PORTS_PATH/$CATEGORY/$NAME/Portfile"

echo "🌞 Linting the Portfile..."
(cd "$LOCAL_PORTS_PATH/$CATEGORY/$NAME" && port lint --nitpick)
echo "🔨 Testing the Portfile locally (this will build the package, it may take a while)..."
(cd "$LOCAL_PORTS_PATH/$CATEGORY/$NAME" && sudo port test)

# 6. Update Index
echo "🔄 Updating MacPorts PortIndex..."
(cd "$LOCAL_PORTS_PATH" && sudo portindex)
echo "✅ MacPorts PortIndex updated."
echo "🔄 Getting package info..."
sudo port -v info $NAME && echo "✅ Package info retrieved successfully."
echo "👉 Run 'sudo port -vst install $NAME' to test the new version."
