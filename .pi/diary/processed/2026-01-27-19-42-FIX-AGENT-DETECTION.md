# Session Diary

**Date**: 2026-01-27 19:42
**Session ID**: 2026-01-27-19-42-FIX-AGENT-DETECTION
**Project**: /home/michael/work/ai/TOOLS/tmuxx

## Task Summary
The user wanted to fix the agent state detection (Idle, Working, Approval) for Claude and Pi agents which was failing multiple regression tests. The goal was to move away from brittle line-count-based detection to a robust structural model.

## Work Done
- **Structural Splitter Model**: Implemented a new logic in `src/parsers/universal.rs` that searches from the bottom to identify the "Prompt Sandwich" (separators `───` surrounding the prompt `❯`).
- **Precise Refinements**: Added new `MatchLocation` enum variants (`LastLine`, `LastBlock`, `FirstLineOfLastBlock`) to target specific areas of the output.
- **Config Overhaul**: Completely rewrote Claude and Pi agent definitions in `src/config/defaults.toml` to use the new structural splitters and prioritized refinements.
- **Improved Test Coverage**: Regression tests improved from ~42 Passed to 56 Passed (out of 58).
- **Skill Update**: Updated `.pi/skills/tmuxx-creating-agent-definition/SKILL.md` with the new Structural Splitter Model principles.

## Design Decisions
- **Bottom-up Search**: Splitters now search from the end of the buffer to the top. This isolates the *current* screen and ignores historical separators in long terminal sessions.
- **Body vs. Prompt Isolation**: The splitter separates the output into `body` (agent's response) and `prompt` (interactive area). This allows refinements to ignore text typed by the user.
- **Priority Logic**: "Working" indicators (spinners) now have absolute priority. If a spinner is present in the last block, the agent is `working` even if there are unfinished tasks or questions.

## Challenges & Solutions
| Challenge | Solution |
|-----------|----------|
| User input in prompt triggering false positives | Splitter isolates user prompt into a separate group; refinements target the `body` group. |
| Terminal history leaking into detection | Search from bottom and use `LastLine`/`LastBlock` matching to keep detection local to the current screen. |
| Claude's complex task lists | Implemented `FirstLineOfLastBlock` location to check for the spinner at the start of the task block. |

## Mistakes & Corrections
### Where I Made Errors:
- **Rust Implementation Errors**: I had multiple compilation failures in `universal.rs` because I used wrong trait method names (`highlight_content` vs `highlight_line`) and incorrect struct fields for `Subagent`.
- **Aggressive Splitters**: Initially, my splitter was either too greedy or searched from the top, causing it to swallow the entire screen into the `prompt` group.
- **Ordering of Rules**: I placed the `idle` rule (the splitter) at the top of the list, which accidentally disabled subsequent `approval` rules.

### What Caused the Mistakes:
- **Lack of Local Context**: I was relying on outdated or incorrect versions of the `AgentParser` trait definition before reading `src/parsers/mod.rs`.
- **Regex Greediness**: Forgetting that `.*` in multiline mode can easily consume structural separators in a high-history buffer.

## Lessons Learned
### Technical Lessons:
- **Braille Range**: Using `[\u2800-\u28FF]` is a foolproof way to catch all types of terminal spinners (Pi uses these).
- **Structural Invariants**: Terminal apps often have a "Sandwich" (Separator -> Content -> Separator). Detecting the *last* sandwich is the only way to get the current state.

### Process Lessons:
- **Admin Perspective**: "Look at the end of the screen" is the primary rule for terminal state detection.
- **Precise Matching**: Matching anywhere in a 16KB buffer is a recipe for disaster; location-based matching (`LastLine`, `LastBlock`) is mandatory.

## Skills Used
- [x] Skill: `.pi/skills/tmuxx-creating-agent-definition/SKILL.md` - Refactored to include structural detection.
- [x] Skill: `~/.pi/agent/skills/creating-skills/SKILL.md` - Used as reference for updating the tmuxx skill.

### Feedback for Skills:
| File | Issue/Observation | Suggested Fix/Action |
|------|-------------------|----------------------|
| `tmuxx-creating-agent-definition` | Was based on line counts | Updated to the Splitter Model (Structural detection) |

## User Preferences Observed
- **Config-Driven**: Logic must remain in `defaults.toml` but can be supported by generic helper functions in code.
- **Non-Brittle Code**: Prefers identifying "known blocks" over hardcoded line counts.
- **Honesty in Errors**: Expects a clear explanation of why detection failed or was misunderstood.

## Code Patterns Used
- **Backward Line Search**: `lines.iter().enumerate().rev()` to find the last occurrence of structural elements.
- **Refinement Grouping**: Splitting regex matches into named groups (`body`, `prompt`) and targeting refinements specifically to one of them.

## Notes
The session concluded with 56/58 tests passing. The remaining two failing tests are related to Claude's `accept edits` bar in specific idle contexts, which requires fine-tuning the priority between Rule 0 (global approvals) and Rule 1 (the structural splitter).
