#!/bin/bash

# extract to clipboard
for filename in outputs/*.json; do
    jq -r '.assets[].name' "$filename" | \
    sed -e 's/^/"/g' -e 's/$/", /g' | \
    tr -d '\n' ; echo ""
done | xclip
