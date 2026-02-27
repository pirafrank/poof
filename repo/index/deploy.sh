#!/usr/bin/env bash

set -e

TARGET=/tmp/poof-pkgs
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# go no-op
: "${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID is required}"
: "${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY is required}"
: "${R2_BUCKET_NAME:?R2_BUCKET_NAME is required}"
: "${R2_ACCOUNT_ID:?R2_ACCOUNT_ID is required}"

mkdir -p "$TARGET"
"$SCRIPT_DIR/readme2index.sh" > "$TARGET/index.html"
aws s3 cp "$TARGET/index.html" "s3://${R2_BUCKET_NAME}/index.html" \
  --endpoint-url "https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com" \
  --region auto

rm -f "$TARGET/index.html"

