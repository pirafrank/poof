#!/bin/sh
set -e

# Required env vars:
#   PKGVER    - package version without the leading 'v' (e.g. 0.6.0)
#   PKGREL    - package release number (e.g. 0)
#   ARCH      - Alpine architecture (e.g. x86_64, aarch64, armv7)
#   KEY_FILE  - absolute path to the RSA private key inside the container
#
# Required mounts:
#   /input/<pkgname>  - pre-built binary (read-only)
#   /apk-keys/        - directory containing the RSA private key (read-only)
#   /apkbuild/        - directory containing APKBUILD.template (read-only)
#   /output/          - destination for the produced .apk file

PKGNAME="${PKGNAME:-poof}"

# Validate required env vars
for var in PKGVER PKGREL ARCH KEY_FILE; do
    eval "val=\$$var"
    if [ -z "$val" ]; then
        echo "ERROR: $var is not set" >&2
        exit 1
    fi
done

# Set up abuild signing key
mkdir -p ~/.abuild
echo "PACKAGER_PRIVKEY=\"${KEY_FILE}\"" > ~/.abuild/abuild.conf
chmod 600 ~/.abuild/abuild.conf

# Install public key into the system trust store so apk can verify
# signatures when abuild creates the local repository index
cp "${KEY_FILE}.pub" /etc/apk/keys/

# Build the aports-style source directory:
#   ~/src/<repo_name>/<pkgname>/APKBUILD
# abuild derives the repo name from the parent directory name.
REPO_DIR=~/src/poof
PKG_DIR="${REPO_DIR}/${PKGNAME}"
mkdir -p "${PKG_DIR}"

# Fill in the APKBUILD template
sed \
    -e "s/@@PKGVER@@/${PKGVER}/g" \
    -e "s/@@PKGREL@@/${PKGREL}/g" \
    -e "s/@@ARCH@@/${ARCH}/g" \
    /apkbuild/APKBUILD.template > "${PKG_DIR}/APKBUILD"

# Run abuild - skips fetch/unpack since source= is empty;
# package() installs the binary directly from /input/<pkgname>.
cd "${PKG_DIR}"
abuild -F

# Locate and copy the produced .apk to /output/
find ~/packages -name "*.apk" -exec cp {} /output/ \;

echo "Done. Contents of /output/:"
ls /output/
