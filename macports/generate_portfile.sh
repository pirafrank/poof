#!/bin/bash

### Generate a Portfile for poof and test it locally.
### Run with: ./generate_portfile.sh <version>

# --- Configuration ---
NAME="poof"
GH_USER="pirafrank"
REPO="poof"
CATEGORY="sysutils"
MAINTAINER="@pirafrank"
LOCAL_PORTS_PATH="$HOME/pirafrank/ports"

# --- Version and Source ---
VERSION=${1:-$(git describe --tags --abbrev=0 | sed 's/v//')}
SRC_TARBALL="v${VERSION}.tar.gz"
MAN_PAGE="poof.1"

SRC_URL="https://github.com/$GH_USER/$REPO/archive/refs/tags/$SRC_TARBALL"
MAN_PAGE_URL="https://github.com/$GH_USER/$REPO/releases/download/v$VERSION/$MAN_PAGE"

# --- Helper functions ---

# 3. Helper functions
get_checksums() {
    local url=$1
    local file=$2
    curl -L "$url" -o "$file"
    local size=$(stat -f%z "$file")
    local sha256=$(shasum -a 256 "$file" | awk '{print $1}')
    local rmd160=$(openssl dgst -rmd160 "$file" | awk '{print $2}')
    echo "$file $rmd160 $sha256 $size"
    # We keep the file briefly for cargo-generate-portfile if it's the source
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

# 1. Prepare

# Cleanup local tree
cleanup_local_tree

# Check for cargo-generate-portfile
if ! command -v cargo-generate-portfile &> /dev/null; then
    echo "❌ Error: cargo-generate-portfile not found."
    exit 1
fi

echo "🚀 Preparing Source + Manpage Portfile for $NAME v$VERSION..."

# 2. Fetch checksums
echo "📥 Fetching Source & Asset for checksumming..."
SRC_CHKS=($(get_checksums "$SRC_URL" "$SRC_TARBALL"))
MAN_CHKS=($(get_checksums "$MAN_PAGE_URL" "$MAN_PAGE"))

# 4. Generate Cargo Dependency Block
echo "📦 Generating Cargo crate list..."
# Note: cargo-generate-portfile usually runs against a local directory
CRATES_BLOCK=$(cargo generate-portfile | sed -n '/cargo.crates/,/}/p')

# Cleanup downloaded temp files
rm "$SRC_TARBALL" "$MAN_PAGE"

# 5. Write the Portfile
cat <<EOF > "$LOCAL_PORTS_PATH/$CATEGORY/$NAME/Portfile
# -*- coding: utf-8; mode: tcl; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- vim:fenc=utf-8:ft=tcl:et:sw=4:ts=4:sts=4

PortSystem          1.0
PortGroup           cargo 1.0

name                $NAME
version             $VERSION
categories          $CATEGORY
platforms           darwin
license             MIT
maintainers         $MAINTAINER

description         A CLI package manager for pre-built software.
long_description    \${description}

master_sites        https://github.com/$GH_USER/$REPO/archive/refs/tags/:source \\
                    https://github.com/$GH_USER/$REPO/releases/download/v\${version}/:asset

distfiles           v\${version}.tar.gz:source \\
                    $MAN_PAGE:asset

checksums           v\${version}.tar.gz \\
                    rmd160  ${SRC_CHKS[1]} \\
                    sha256  ${SRC_CHKS[2]} \\
                    size    ${SRC_CHKS[3]} \\
                    $MAN_PAGE \\
                    rmd160  ${MAN_CHKS[1]} \\
                    sha256  ${MAN_CHKS[2]} \\
                    size    ${MAN_CHKS[3]}

$CRATES_BLOCK

# The 'cargo' PortGroup automatically handles building and installing the binary.
# We just need to manually install the man page asset.
post-destroot {
    xinstall -m 0644 \${distpath}/$MAN_PAGE \${destroot}\${prefix}/share/man/man1/
}
EOF

echo "✅ Portfile is ready at $LOCAL_PORTS_PATH/$CATEGORY/$NAME/Portfile"

# 6. Update Index
echo "🔄 Updating MacPorts PortIndex..."
(cd "$LOCAL_PORTS_PATH" && portindex)
echo "✅ MacPorts PortIndex updated."
echo "👉 Run 'sudo port -v install $NAME' to test the new version."
