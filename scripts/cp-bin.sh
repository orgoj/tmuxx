#!/bin/bash
# Install tmuxx binaries to ~/bin
set -e

declare -A FILES=(
    ["target/release/tmuxx"]="$HOME/bin/tmuxx"
    ["scripts/tmuxx-wrapper.sh"]="$HOME/bin/txx"
)

for src in "${!FILES[@]}"; do
    dst="${FILES[$src]}"
    if [ -f "$src" ]; then
        cp "$src" "$dst"
        chmod +x "$dst"
        echo "Installed: $dst"
    else
        echo "Error: $src not found" >&2
        exit 1
    fi
done
