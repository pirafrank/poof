#!/bin/sh
set -eu

# Downloads a real Type 2 fixture for the ignored
# test_process_install_real_type_2_appimage test. The test stays ignored because
# the third-party AppImage is not committed to this repository.

url='https://github.com/AnimMouse/hello-world-appimage/releases/download/v1/helloworld-appimage-x86_64.AppImage'
sha256='c60924583828a245ba0b653397ea673258b7dc313a6e66f1ba19b1f891192e9a'
destination=${1:-/tmp/poof-hello-type2.AppImage}

curl --fail --location --silent --show-error "$url" --output "$destination"
printf '%s  %s\n' "$sha256" "$destination" | sha256sum --check --status || {
    rm -f "$destination"
    printf 'Downloaded AppImage failed SHA-256 verification.\n' >&2
    exit 1
}

printf 'Downloaded verified Type 2 AppImage to %s\n' "$destination"
printf 'Run with: POOF_TEST_APPIMAGE=%s cargo test test_process_install_real_type_2_appimage -- --ignored\n' "$destination"
