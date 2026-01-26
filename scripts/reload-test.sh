#!/bin/bash
# Reload tmuxx in ct-test session
SESSION="ct-test"

# Kill any running process with multiple Ctrl+C
tmux send-keys -t "$SESSION" C-c
sleep 0.3
tmux send-keys -t "$SESSION" C-c
sleep 0.3
tmux send-keys -t "$SESSION" C-c
sleep 0.3

# Reset terminal to fix any corruption
tmux send-keys -t "$SESSION" 'reset' Enter
sleep 1

# Start tmuxx
tmux send-keys -t "$SESSION" './target/release/tmuxx' Enter
sleep 2
tmux capture-pane -t "$SESSION" -p
