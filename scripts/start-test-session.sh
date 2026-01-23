#!/bin/bash
# Start ct-test session for testing tmuxcc
SESSION="ct-test"
PROJECT_DIR="/home/michael/work/ai/TOOLS/tmuxcc"

# Kill existing session if present
tmux has-session -t "$SESSION" 2>/dev/null && tmux kill-session -t "$SESSION"

# Create new session in project directory
tmux new-session -d -s "$SESSION" -c "$PROJECT_DIR"

# Start tmuxcc
tmux send-keys -t "$SESSION" './target/release/tmuxcc' Enter

echo "Test session '$SESSION' started. Attach with: tmux attach -t $SESSION"
