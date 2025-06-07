#!/bin/bash

set -e

# Check if poof is available in PATH
if ! command -v poof &> /dev/null; then
    echo "Error: poof command not found in PATH"
    exit 1
fi

# Check poof version
POOF_VERSION=$(poof version | grep Version | awk '{print $3}')
if [ -z "$POOF_VERSION" ]; then
    echo "Error: Could not determine poof version"
    exit 1
fi

# Compare versions (0.4.0 or lower)
# Make the topmost version the highest supported by this migration script, which is 0.4.0.
if [ "$(printf '%s\n' "0.4.0" "$POOF_VERSION" | sort -rV | head -n1)" != "0.4.0" ]; then
    echo "Error: poof version must be 0.4.0 or lower (found: $POOF_VERSION)"
    exit 1
fi

# Determine platform and set POOF_DIR
case "$(uname)" in
    "Linux")
        POOF_DIR="$HOME/.local/share/poof"
        ;;
    "Darwin")
        POOF_DIR="$HOME/Library/Application Support/poof"
        ;;
    *)
        echo "Error: Unsupported platform"
        exit 1
        ;;
esac

DATA_DIR="$POOF_DIR/data"
NEW_DATA_DIR="$DATA_DIR/github.com"
BIN_DIR="$POOF_DIR/bin"

# Create the new directory structure
mkdir -p "$NEW_DATA_DIR"

# Move all directories except github.com to the new location
for dir in "$DATA_DIR"/*; do
    if [ -d "$dir" ] && [ "$(basename "$dir")" != "github.com" ]; then
        echo "Moving $(basename "$dir") to github.com/"
        mv "$dir" "$NEW_DATA_DIR/"
    fi
done

# Update all symlinks in bin directory
for link in "$BIN_DIR"/*; do
    if [ -L "$link" ]; then
        target=$(readlink "$link")
        if [[ "$target" == "$DATA_DIR"/* ]]; then
            # Extract the path after data/
            relative_path=${target#"$DATA_DIR/"}
            # Create the new target path
            new_target="$NEW_DATA_DIR/$relative_path"
            # Remove old symlink and create a new one
            rm "$link"
            ln -s "$new_target" "$link"
            echo "Updated symlink: $(basename "$link") -> $new_target"
        fi
    fi
done

echo "Migration completed successfully!"