#!/usr/bin/env bash

set -e

TARGET=/tmp/poof-pkgs

mkdir -p "$TARGET"
./readme2index.sh > "$TARGET/index.html"
aws s3 cp "$TARGET/index.html" "s3://${R2_BUCKET_NAME}/index.html" \
  --endpoint-url "https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com" \
  --region auto

rm -f "$TARGET/index.html"

