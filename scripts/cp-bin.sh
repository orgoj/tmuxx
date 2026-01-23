#!/bin/bash
# Script to install tmuxcc binary to ~/bin after approval
set -e

SOURCE="target/release/tmuxcc"
DEST="$HOME/bin/tmuxcc"

if [ ! -f "$SOURCE" ]; then
    echo "Error: $SOURCE not found. Run 'cargo build --release' first."
    exit 1
fi

cp "$SOURCE" "$DEST"
echo "Installed: $DEST"
ls -la "$DEST"
