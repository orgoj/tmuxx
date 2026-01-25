#!/bin/bash
# Install tmuxcc binaries to ~/bin
set -e

declare -A FILES=(
    ["target/release/tmuxcc"]="$HOME/bin/tmuxcc"
    ["scripts/tmuxcc-wrapper.sh"]="$HOME/bin/tcc"
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
