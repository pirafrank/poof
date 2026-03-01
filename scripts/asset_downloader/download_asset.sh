#!/bin/bash

# NB. it requires GITHUB_TOKEN env varwith a PAT classic token
#     having public_repo and repo:status permissions.

if [ "$#" -ne 1 ]; then
		echo "Usage: $0 <path to user/repo txt list file>"
		exit 1
fi

if [ -z "$GITHUB_TOKEN" ]; then
  echo '"GITHUB_TOKEN" env var is empty!'
  exit 1
fi

# Download the latest release for each repository
# and save the output to a JSON file to avoid github rate limiting
while IFS= read -r line; do
    if [ -z "$line" ]; then
      continue
    fi
    echo "Processing: $line"
    filename=${line//\//@}.json
    curl -sSL \
      -H "Accept: application/vnd.github.v3+json" \
      -H "Authorization: bearer $GITHUB_TOKEN" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      https://api.github.com/repos/${line}/releases/latest > outputs/$filename
done < "$1"

# Extract the asset names from the JSON files and save them to a CSV file in repo root
#for filename in outputs/*.json; do
#    cat $filename | jq -r '.assets.[].name' >> ../../tests/fixtures/selector/test_db.csv
#done
