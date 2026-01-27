---
name: tmuxx-testing
description: "Use this to assess, check, and verify tmuxx behavior in live tmux sessions. Enforces strict safety rules to prevent destructive command execution."
---

# Testing Tmuxx

You use this skill to verify your changes in a real environment. Tmuxx interacts with tmux, so testing requires careful execution to avoid sending keys to the wrong place.

## Requirements
- A running tmux environment with test sessions (`ct-test`, `ct-multi`).

## Steps

### 1. Analysis
- Identify which session needs to be monitored and which will be used for input.
- **Rule**: `ct-test` is the ONLY session for `send-keys`.

### 2. Execution

#### A. Deployment
- Use `./target/release/tmuxx` or `./target/debug/tmuxx`.
- Use `scripts/reload-test.sh` to refresh the app in the test session.

#### B. Interaction
- Send keys **ONE AT A TIME**.
- **Rule**: Capture the pane and check the result AFTER EVERY KEY.
```bash
tmux send-keys -t ct-test "echo 'test'" Enter
sleep 0.5
tmux capture-pane -t ct-test -p
```

### 3. Verification (Safety Audit)

#### A. Content Check
- **NEVER** use `tail` or `head` with `capture-pane`. Read the full output.
- If capture is empty or doesn't show a bash prompt, **STOP IMMEDIATELY**.

#### B. UI Verification
- Perform visual verification of TUI elements.
- Verify that configuration overrides work as expected.
- Run `cargo clippy` to ensure no linting regressions.
