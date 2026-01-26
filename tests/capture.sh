#!/bin/bash
set -e

# Usage: ./tests/capture.sh <agent_id> <target_pane> <expected_status> [description]
# Example: ./tests/capture.sh claude cc-ai-maestro idle "sauteed_state"

AGENT_ID=$1
TARGET=$2
STATUS=$3
DESC=$4

if [ -z "$AGENT_ID" ] || [ -z "$TARGET" ] || [ -z "$STATUS" ]; then
    echo "Usage: $0 <agent_id> <target_pane> <expected_status> [description]"
    echo "  <expected_status>: idle, awaiting_approval, processing, error, awaiting_input"
    echo "Example: $0 claude cc-ai-maestro idle sauteed_state"
    exit 1
fi

# Determine script directory to allow execution from anywhere
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
FIXTURES_DIR="${SCRIPT_DIR}/fixtures/${AGENT_ID}"
mkdir -p "$FIXTURES_DIR"

if [ -z "$DESC" ]; then
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    DESC="capture_${TIMESTAMP}"
fi

FILENAME="case_${STATUS}_${DESC}.txt"
OUTPATH="${FIXTURES_DIR}/${FILENAME}"

echo "📸 Capturing pane '${TARGET}'..."
if ! tmux capture-pane -p -t "$TARGET" > "$OUTPATH.tmp"; then
    echo "❌ Failed to capture pane '${TARGET}'"
    rm -f "$OUTPATH.tmp"
    exit 1
fi

# Clean up TUI artifacts and anonymize
echo "🧹 Cleaning and anonymizing..."
# Remove leading pipe characters from pane splits
sed -i 's/^[ \t]*│//' "$OUTPATH.tmp"

# Anonymization: replace HOME and username to protect privacy
# Using | as separator for sed to handle paths safely
sed -i "s|$HOME|/home/user|g" "$OUTPATH.tmp"
sed -i "s|$(whoami)|user|g" "$OUTPATH.tmp"

# Optional: Add custom strings from .tmuxx_scrub if it exists
if [ -f "${SCRIPT_DIR}/.tmuxx_scrub" ]; then
    while IFS='=' read -r key value; do
        if [[ ! $key =~ ^# && -n $key ]]; then
            sed -i "s|$key|$value|g" "$OUTPATH.tmp"
        fi
    done < "${SCRIPT_DIR}/.tmuxx_scrub"
fi

mv "$OUTPATH.tmp" "$OUTPATH"

echo "✅ Saved fixture to: ${OUTPATH}"
echo "   Expected Status: ${STATUS}"
