#!/bin/bash
#
# Smoke tests for the poof devcontainer feature.
#
set -e

# shellcheck source=/dev/null
source dev-container-features-test-lib

check "poof is on PATH" command -v poof
check "poof --version exits 0" poof --version

reportResults
