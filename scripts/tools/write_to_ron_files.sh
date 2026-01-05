#!/bin/bash

# extract to clipboard
for filename in outputs/*.json; do
    targetfile=${filename%.json}.ron
    echo "[" > $targetfile
    cat $filename | jq -r '.assets[].name' | \
    sed -e 's/^/    "/g' -e 's/$/",/g' >> $targetfile
    echo "]" >> $targetfile
done
# move to src/core/tests/assets
#cp -a outputs/*.ron ../src/core/tests/assets/
