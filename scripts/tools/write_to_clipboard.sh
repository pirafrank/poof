#!/bin/bash

# extract to clipboard
for filename in outputs/*.json; do
    cat $filename | jq -r '.assets[].name' | \
    sed -e 's/^/"/g' -e 's/$/", /g' | \
    tr -d '\n' ; echo ""
done | xclip
