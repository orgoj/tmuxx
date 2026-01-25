#!/usr/bin/env bash
# tmuxcc-wrapper.sh - Always run tmuxcc inside a tmux session
#
# This wrapper ensures tmuxcc always runs inside the 'tmuxcc' session,
# making the 'f' key (focus pane) work reliably for cross-session navigation.
#
# Usage:
#   ./scripts/tmuxcc-wrapper.sh
#   OR: symlink to ~/bin/tcc for quick access

set -euo pipefail

SESSION="tmuxcc"
TMUXCC_BIN="tmuxcc"  # Assumes tmuxcc is in PATH

# Check if tmuxcc binary exists and get full path
if ! TMUXCC_PATH=$(command -v "$TMUXCC_BIN" 2>/dev/null); then
    echo "Error: tmuxcc binary not found in PATH" >&2
    echo "Run: cargo install --path . OR add target/release to PATH" >&2
    exit 1
fi

# Check if session exists
if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Creating new tmux session: $SESSION"
    # Create new session in detached mode, running bash
    tmux new-session -d -s "$SESSION" bash
    # Send tmuxcc command to the session
    tmux send-keys -t "$SESSION" "$TMUXCC_PATH" Enter
fi

# Attach or switch to the session
if [ -n "${TMUX:-}" ]; then
    # Already inside tmux: switch to tmuxcc session
    echo "Switching to session: $SESSION"
    tmux switch-client -t "$SESSION"
else
    # Outside tmux: attach to session
    echo "Attaching to session: $SESSION"
    tmux attach-session -t "$SESSION"
fi
