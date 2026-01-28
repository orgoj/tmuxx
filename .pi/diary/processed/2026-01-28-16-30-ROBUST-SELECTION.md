# Session Diary

**Date**: 2026-01-28 16:30
**Session ID**: 2026-01-28-16-30-ROBUST-SELECTION
**Project**: /home/michael/work/ai/TOOLS/tmuxx

## Task Summary
The user needed to fix an issue where agent selection (cursor and multi-select) was being lost during tmux session renames or agent restarts. Additionally, I was tasked with implementing a full definition for the Google Gemini agent and refining detection for Claude and Pi Powerline agents based on failing regression tests.

## Work Done
- **Robust Selection Tracking**:
  - Modified `src/app/state.rs` to include `selected_agent_pid` and `selected_agent_target` for single selection.
  - Added `selected_pids` and `selected_targets` to track multi-selection.
  - Updated `sync_selection` to use a multi-stage fallback: ID -> PID -> Target.
- **AI Agent Detection**:
  - Added full Gemini agent definition to `src/config/defaults.toml` with content-based matching.
  - Improved Claude detection for "approve edits" and "unfinished tasks" (◻).
  - Fixed Pi Powerline "Working..." detection and handled initialization screens.
- **Maintenance**:
  - Updated `CHANGELOG.md` and `TODO.md`.
  - Reorganized and renamed regression test fixtures in `tests/fixtures/`.
  - Verified 63 passing regression tests.

## Design Decisions
- **Selection Fallbacks**: Chose PID as the first fallback for session renames (process stays same) and Target (session:window.pane) as the second fallback for restarts (pane stays same). This covers the most common user workflows where a generated unique ID might change.
- **Splitter-based Detection**: Decided to use `splitter = "none"` for Gemini to ensure the main `pattern` is always checked against the whole body, while using `splitter = "powerline_box"` for Pi to isolate the TUI-like prompt.

## Challenges & Solutions
| Challenge | Solution |
|-----------|----------|
| Selection lost on rename | Tracked PID in addition to ID; PID persists even if session name changes. |
| Gemini detected as Processing | Added `splitter = "none"` and refined regex to match the prompt and model footer correctly. |
| Pi Powerline Idle vs Working | Reordered rules to check Working/Approval before Idle, as the Powerline box is persistent. |

## Mistakes & Corrections

### Where I Made Errors:
- **Incomplete Dumps**: I provided only the footer (prompt box) of a failing test case when the user asked for context. The user rightly called me out on this, as the body content was crucial for determining the state.
- **Incorrect Test Expectation**: I spent several cycles trying to make a "Working" detection work for a screen that was clearly "Idle". The user corrected me that the test fixture itself was likely miscategorized.
- **Rule Skipping**: Initially omitted `splitter = "none"` for Gemini, which caused the rule to be ignored because the `UniversalParser` expects a splitter match if defined.

### What Caused the Mistakes:
- **Tunnel Vision**: Focused too much on the regex and not enough on the overall structural logic of the `UniversalParser` and the specific visual state of the agent in the fixture.

## Lessons Learned

### Technical Lessons:
- **UniversalParser Logic**: If a `state_rule` has a splitter, it *must* find that splitter in the text to proceed. For non-TUI agents, ensuring the rule logic doesn't depend on a missing splitter is vital.
- **State Hysteresis**: Selection persistence is a "must-have" for TUI tools; IDs alone are too fragile in dynamic environments like tmux.

### Process Lessons:
- **Tail the Whole Picture**: When a test fails, read the last 20-40 lines of the fixture, not just the last block. The state often depends on headers/system messages further up.
- **Question the Fixtures**: If a "Working" test is failing and it looks "Idle", check if the expected state in the filename is actually correct before twisting the regex into knots.

### To Remember for CLAUDE.md:
- Selection must always be tied to PID or Target if ID fails.
- Commit messages: Use the Problem/Solution/Changes format.

## Skills Used

### Used in this session:
- [x] Skill: `~/.pi/agent/skills/tmuxx-working-on-todos/SKILL.md` - Used to pick up tasks from the roadmap.
- [x] Skill: `~/.pi/agent/skills/tmuxx-committing-changes/SKILL.md` - Used for pre-commit checks and formatting.
- [x] Skill: `~/.pi/agent/skills/tmuxx-managing-changelogs/SKILL.md` - Used to update documentation.

## User Preferences Observed

### Git & PR Preferences:
- Commit messages must be structured: `<type>: brief`, then Problem/Solution/Changes blocks.
- Don't include unrelated changes (like skill updates) in a feature/fix commit.

### Code Quality Preferences:
- Use `cargo run -- test` for regression testing with fixtures.
- `cargo fmt` and `cargo clippy` are mandatory before commit.

## Code Patterns Used
- **Fallback Selection Sync**: Storing multiple keys (ID, PID, Target) to maintain UI state across external events.
- **Regex-based AI State Refinement**: Splitting TUI output into Body/Prompt groups to apply granular detection rules.

## Notes
The user is very sensitive to selection loss, as it disrupts the workflow. The current implementation is now much more robust.
