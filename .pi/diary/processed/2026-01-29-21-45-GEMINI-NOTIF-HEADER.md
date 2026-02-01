# Session Diary

**Date**: 2026-01-29 21:45
**Session ID**: 2026-01-29-21-45-GEMINI-NOTIF-HEADER
**Project**: /home/michael/work/ai/TOOLS/tmuxx

## Task Summary
Fix Gemini agent detection (when running as node child), resolve notification spamming during user interaction, and distinguish between AI agents and generic processes in the header count.

## Work Done
- **Gemini Detection**:
    - Added `ancestor` matcher for "gemini" in `defaults.toml`.
    - Increased `last_lines` buffer from 40 to 100 for Gemini state detection.
    - Moved Gemini start screen fixture from `generic_shell` to `gemini` and renamed to `case_idle_...`.
- **Notification Spam**:
    - Fixed `handle_notifications` to reset `global_notification_sent` and `notified_agents` only when queue is empty or interaction occurs.
    - Implemented timer reset: User interaction now sets `approval_since` values to `now`, resetting the delay.
- **AI Agent Distinction**:
    - Added `is_ai` boolean flag to `AgentConfig`.
    - Updated `MonitoredAgent` and `AgentTree` to track AI status.
    - Modified `HeaderWidget` to display "agents" and "other" counts separately.
    - Set `is_ai = false` for `generic_shell` in `defaults.toml`.
- **Project Memory**:
    - Updated `CLAUDE.md` with mandatory regression testing rules when touching fixtures or `defaults.toml`.
- **Versions**:
    - Bumped version from 0.4.1 to 0.4.5 across multiple commits.

## Design Decisions
- **Ancestor Matching for Gemini**: Necessary because Gemini often runs as a child of `node` in tmux panes, and `node` is too generic for top-level command matching.
- **Interaction Timer Reset**: Decided to reset `approval_since` to `now` upon user interaction so that the notification delay starts over, giving the user quiet time after they've acknowledged the app's state.
- **Explicit `is_ai` Flag**: Added to the config to allow users to define which monitored processes are actual AI agents, enabling cleaner summary stats in the UI.

## Challenges & Solutions
| Challenge | Solution |
|-----------|----------|
| Gemini idle state not detected | Increased `last_lines` in `defaults.toml` because the fixture had many trailing empty lines. |
| Notification spam despite 'sent' flag | Discovered that while the flag was reset, the `elapsed` check against the original `approval_since` was still true, causing immediate re-firing. Solved by resetting the timers to `now`. |
| Header aggregation noise | User noted 7 agents when 3 were just shells. Solution was adding `is_ai` and splitting the counts. |

## Mistakes & Corrections

### Where I Made Errors:
- **Missing Tests**: I initially applied Gemini detection changes without running `cargo run -- test`. The user corrected me immediately.
- **Incomplete Notification Fix**: My first attempt at fixing notifications only reset the boolean flags. The user pointed out it was still spamming. I realized I needed to reset the timestamp as well.

### What Caused the Mistakes:
- **Assumed Success**: I assumed simple regex/config changes didn't need full regression suite runs.
- **Logical Oversight**: I didn't account for the fact that `Instant::now().duration_since(old_time)` would remain greater than the delay even after resetting the "sent" bit.

## Lessons Learned

### Technical Lessons:
- **Tmux State Extraction**: Increasing `last_lines` is often necessary for agents that output a lot of "chrome" or empty space.
- **Interaction Feedback Loops**: When resetting notification state based on user input, always reset the temporal reference (the "since" timer) to avoid immediate re-triggering.

### Process Lessons:
- **Regression Testing is Mandatory**: Always run the fixture-based regression tests (`cargo run -- test`) when changing `defaults.toml` or moving fixtures.
- **CLAUDE.md Enforcement**: Explicitly documenting these "automatic" steps in `CLAUDE.md` helps maintain discipline.

### To Remember for CLAUDE.md:
- Added: "When modifying source code or test fixtures, ALWAYS run regression tests: `cargo run -- test`"
- Added: "Always verify agent detection by running `cargo run -- test` after changing `defaults.toml`"

## Skills Used

### Used in this session:
- [x] Skill: `tmuxx-committing-changes` - Used for pre-commit checks and structured messages.
- [x] Skill: `selflearn-diary` - Documenting this session.

### Feedback for Skills:
*(None)*

## User Preferences Observed

### Git & PR Preferences:
- Commits must have a specific format (type: description, Problem/Solution/Changes blocks).
- Versions should be bumped for each logic/fix iteration.
- Changelog must be updated *before* the version bump and commit.

### Code Quality Preferences:
- **Mandatory Testing**: 100% pass rate on `cargo run -- test` before every commit.
- **Explicit Context**: Don't just fix, explain why (especially in commits and changelogs).

### Technical Preferences:
- Config-driven everything. No hardcoded logic for specific agents if possible.

## Code Patterns Used
- **Atomic Tmux Keys**: Using single multi-arg `send-keys` to avoid races.
- **Atomic Flags**: `AtomicBool` for thread-safe user interaction signaling.

## Notes
The project is maturing in its AI agent handling, with clearer separation of concerns in the UI and more robust temporal handling for notifications.
