#!/bin/bash
# Start ct-test session for testing tmuxx
SESSION="ct-test"
PROJECT_DIR="/home/michael/work/ai/TOOLS/tmuxx"
ATTACH=false

# Parse arguments
while getopts "a" opt; do
    case $opt in
        a) ATTACH=true ;;
        *) echo "Usage: $0 [-a]"; exit 1 ;;
    esac
done

# Kill existing session if present
tmux has-session -t "$SESSION" 2>/dev/null && tmux kill-session -t "$SESSION"

# Create new session in project directory
tmux new-session -d -s "$SESSION" -c "$PROJECT_DIR"

# Start tmuxx
tmux send-keys -t "$SESSION" './target/release/tmuxx' Enter

if [ "$ATTACH" = true ]; then
    exec tmux attach -t "$SESSION"
else
    echo "Test session '$SESSION' started. Attach with: tmux attach -t $SESSION"
fi
