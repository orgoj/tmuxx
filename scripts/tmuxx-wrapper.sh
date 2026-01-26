#!/usr/bin/env bash
# tmuxx-wrapper.sh - Always run tmuxx inside a tmux session
#
# This wrapper ensures tmuxx always runs inside the 'tmuxx' session,
# making the 'f' key (focus pane) work reliably for cross-session navigation.
#
# Usage:
#   ./scripts/tmuxx-wrapper.sh
#   OR: symlink to ~/bin/tcc for quick access

set -euo pipefail

SESSION="tmuxx"
TMUXCC_BIN="tmuxx" # Assumes tmuxx is in PATH

# Check if tmuxx binary exists and get full path
if ! TMUXCC_PATH=$(command -v "$TMUXCC_BIN" 2>/dev/null); then
    echo "Error: tmuxx binary not found in PATH" >&2
    echo "Run: cargo install --path . OR add target/release to PATH" >&2
    exit 1
fi

if [ -n "${TMUX:-}" ]; then
    "$TMUXCC_BIN"
else
    # Check if session exists
    if ! tmux has-session -t "$SESSION" 2>/dev/null; then
        echo "Creating new tmux session: $SESSION"
        # Create new session in detached mode, running bash
        tmux new-session -d -s "$SESSION" bash
        # Send tmuxx command to the session
        tmux send-keys -t "$SESSION" "$TMUXCC_PATH" Enter
    fi

    tmux attach-session -t "$SESSION"
fi
