#!/bin/bash
# Setup ct-multi session with multiple windows for testing tmuxcc session preview
SESSION="ct-multi"
PROJECT_DIR="/home/michael/work/ai/TOOLS/tmuxcc"

# Check if session exists
if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Session '$SESSION' already exists. Windows:"
    tmux list-windows -t "$SESSION"
    exit 0
fi

# Create session with multiple windows
tmux new-session -d -s "$SESSION" -c "$PROJECT_DIR"
tmux rename-window -t "$SESSION:0" "s8"
tmux send-keys -t "$SESSION:0" "echo 'Server s8'" Enter
tmux send-keys -t "$SESSION:0" "echo 'Line 2'" Enter
tmux send-keys -t "$SESSION:0" "echo 'Line 3'" Enter

tmux new-window -t "$SESSION" -n "s9" -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION:1" "echo 'Window s9 - some content here'" Enter

tmux new-window -t "$SESSION" -n "o1" -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION:2" "echo 'Window o1 - more content'" Enter

tmux new-window -t "$SESSION" -n "o2" -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION:3" "echo 'Window o2 - another pane'" Enter

tmux new-window -t "$SESSION" -n "ssh-l2" -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION:4" "echo 'SSH to l2-cx'" Enter

echo "Test session '$SESSION' created with 5 windows:"
tmux list-windows -t "$SESSION"
