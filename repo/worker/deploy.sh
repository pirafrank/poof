#!/usr/bin/env bash

#
# poof repo worker deploy script
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
wrangler deploy --config wrangler.toml

