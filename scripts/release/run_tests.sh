#!/bin/bash

set -e

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "Running tests..."
# tomllib requires Python 3.11 or later
if ! python3 -c 'import sys; sys.exit(0 if sys.version_info >= (3, 11) else 1)'; then
    echo "Python 3.11 or later is required." >&2
    exit 1
fi
python3 -m unittest -v test_release_assets
