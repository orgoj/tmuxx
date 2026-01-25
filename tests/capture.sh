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

# Clean up TUI artifacts (borders)
echo "🧹 Cleaning artifacts..."
# Remove leading pipe characters and spaces used for borders, but keep content
sed -i 's/^[ \t]*│//' "$OUTPATH.tmp"
sed -i 's/^[ \t]*//' "$OUTPATH.tmp" # Optional: aggressive trim if needed, but let's be careful.
# Actually, the regex in defaults.toml handles indentation.
# But for the fixture to be "pure" content as if printed by CLI, we might want to strip common indentation.
# For now, let's just strip the explicit border char if present at start of line with safe leading space match.
sed -i 's/^[ \t]*│//' "$OUTPATH.tmp"

mv "$OUTPATH.tmp" "$OUTPATH"

echo "✅ Saved fixture to: ${OUTPATH}"
echo "   Expected Status: ${STATUS}"
