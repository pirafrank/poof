#!/bin/bash

set -e

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "Running npm packaging tests..."
./run_python_tests.sh

echo "Running launcher tests..."
./run_launcher_tests.sh
