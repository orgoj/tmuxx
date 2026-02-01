#!/bin/bash
# Reload tmuxx in ct-test session
SESSION="ct-test"

echo "Building tmuxx..."
cargo build

# Kill any running tmuxx process in the session
# First try graceful Ctrl+C (now fixed to work in dialogs)
tmux send-keys -t "$SESSION" C-c
sleep 0.2
# Then try 'q'
tmux send-keys -t "$SESSION" q
sleep 0.2
# If still running, kill the process group
pkill -9 tmuxx || true

# Reset terminal to fix any corruption
tmux send-keys -t "$SESSION" 'reset' Enter
sleep 0.5

# Start tmuxx
echo "Starting tmuxx in session $SESSION..."
tmux send-keys -t "$SESSION" './target/debug/tmuxx' Enter
sleep 1.5

# Open command palette for inspection
tmux send-keys -t "$SESSION" C-p
sleep 0.5

tmux capture-pane -t "$SESSION" -p
