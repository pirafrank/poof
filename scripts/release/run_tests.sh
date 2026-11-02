#!/bin/bash

set -e

echo "Running tests..."
python3.12 -m unittest discover -p 'test_release_assets.py' -v
