#!/bin/bash

set -e

cd "$(dirname "${BASH_SOURCE[0]}")"

if ! command -v node >/dev/null 2>&1; then
    echo "Node.js 18 or later is required for launcher tests." >&2
    exit 1
fi

# Explicit file paths: Node 18/20 lack --test glob support and Node 24 rejects
# a bare directory argument, so let the shell expand the test files.
node --test ../../npm/tests/*.test.cjs
