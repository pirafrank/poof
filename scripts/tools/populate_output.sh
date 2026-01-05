#!/bin/bash

# Download the latest release for each repository
# and save the output to a JSON file to avoid github rate limiting
while IFS= read -r line; do
    echo "Processing: $line"
    filename=${line//\//_}
    curl https://api.github.com/repos/${line}/releases/latest > outputs/$filename.json
done < "$1"

# Extract the asset names from the JSON files and save them to a CSV file in repo root
for filename in outputs/*.json; do
    cat $filename | jq -r '.assets.[].name' >> ../../tests/fixtures/selector/test_db.csv
done
