#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
VENV_DIR=${VENV_DIR:-"$SCRIPT_DIR/.venv"}
PYTHON=${PYTHON:-python3}

"$PYTHON" -m venv "$VENV_DIR"
"$VENV_DIR/bin/python" -m pip install --upgrade pip
"$VENV_DIR/bin/python" -m pip install "$SCRIPT_DIR"

printf 'Virtual environment ready: %s\n' "$VENV_DIR"
printf 'Activate it with: . "%s/bin/activate"\n' "$VENV_DIR"
