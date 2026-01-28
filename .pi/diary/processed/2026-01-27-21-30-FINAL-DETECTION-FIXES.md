# Session Diary

**Date**: 2026-01-27 21:30
**Session ID**: FINAL-DETECTION-FIXES
**Project**: /home/michael/work/ai/TOOLS/tmuxx

## Task Summary
The goal was to reach 100% pass rate in AI agent detection tests (59/59). This involved fixing false positives for Pi/Claude, implementing missing features in the Universal Parser, and ensuring Dippy approvals have absolute priority over background work.

## Work Done
- **Bugs fixed**:
    - Fixed compilation error in `src/parsers/universal.rs` regarding `Subagent` initialization.
    - Fixed Pi detection false positives by using strict anchor-based matching (`\z`) for question marks.
    - Fixed Claude "Idle" detection when a persistent prompt `❯` is active.
    - Fixed Dippy approval priority: Moved TUI menu detection (`→ Yes`) to Rule 0 to ensure it's not hidden by background spinners.
- **Features implemented**:
    - Implemented `highlight_line` and `process_indicators` (SSH/Docker icons) in `UniversalParser`.
    - Added `explain_status` to `UniversalParser` for better debugging.
    - Added `-d` (debug) flag to `tmuxx test` command.
- **Metadata**:
    - Updated `CHANGELOG.md` with all recent fixes and features.
    - Cleaned up `TODO.md` (removed finished items).
    - Updated `/skill:tmuxx-creating-agent-definition` with anchor-based matching guidelines.
    - Bumped project version to **0.2.3**.

## Design Decisions
- **Absolute Priority for Human Interaction**: Dippy approval markers (TUI menus) are now detected in Rule 0 (highest priority), even before structural splitting. This ensures the user is notified of a required action even if a task is still "spinning" in the background.
- **Regex Anchoring (`\z`)**: All status markers that could appear in terminal history (like `?`) MUST be anchored to the very end of the string (`\z`) after accounting for prompt/status bar lines.
- **Structural Over Brittle**: Reinforced the "Structural Splitter Model" as the standard for all agents to avoid breaking on line count changes.

## Challenges & Solutions
| Challenge | Solution |
|-----------|----------|
| False positives for Pi "?" from history | Used `(?s).*\?\s*(?:PromptLines)*\z` to ensure `?` is the final content. |
| Claude showing "Approval" while Idle | Added a refinement to match the naked prompt `❯ ` as `Idle` with correct priority. |
| Dippy Approval missed during background tasks | Moved Dippy TUI detection to the very top of `state_rules` (Rule 0). |
| Hard to debug test failures | Implemented `explain_status` and `-d` flag to show which specific rule/refinement matched. |

## Mistakes & Corrections

### Where I Made Errors:
- **Breaking TUI detection**: I removed the `"Approval Required"` literal string in an attempt to be more precise with `?`, which broke detection for standard Pi TUI menus that don't end in a question mark.
- **Testing frequency**: I didn't run the full test suite immediately after a change, leading to missed regressions.
- **Greedy Regex**: My initial Pi regex was too greedy and matched old history.

### What Caused the Mistakes:
- **Over-optimization**: Trying to make the regex too "clean" by removing strings that seemed redundant but were actually critical for specific UI variants (TUI vs Powerline).
- **Assumptions**: Assuming that if one test case passed, similar ones would too without verifying.

## Lessons Learned

### Technical Lessons:
- **Anchoring is key**: When parsing terminal buffers, always anchor to `\z` for state detection to avoid noise from history.
- **Refinement order**: Order of `refinements` inside a `StateRule` is critical (First Match Wins).
- **Subagent initialization**: Always check for new fields when initializing structs in Rust after upstream changes.

### Process Lessons:
- **Run tests early and often**: Especially with `-d` now available.
- **Listen to user frustration**: If the user says "new fail screenshot", check for new files immediately before assuming state.

### To Remember for CLAUDE.md:
- **Approval Priority**: Approval MUST always have priority if a question/menu is present at the end of output.
- **False Positives**: False detection of Error/Approval is unacceptable.

## Skills Used

### Used in this session:
- [x] Skill: `/home/michael/work/ai/TOOLS/tmuxx/.pi/skills/tmuxx-creating-agent-definition/SKILL.md` - Fixed Pi/Claude rules.
- [x] Skill: `/home/michael/work/ai/TOOLS/tmuxx/.pi/skills/tmuxx-committing-changes/SKILL.md` - Final commit.
- [x] Skill: `/home/michael/work/ai/TOOLS/tmuxx/.pi/skills/tmuxx-bumping-versions/SKILL.md` - Bumped to 0.2.3.
- [x] Skill: `~/.pi/agent/skills/selflearn-diary/SKILL.md` - This diary.

### Feedback for Skills:

| File | Issue/Observation | Suggested Fix/Action |
|------|-------------------|----------------------|
| `tmuxx-creating-agent-definition` | Needed info about anchoring | Already updated with `\z` anchoring section. |

## User Preferences Observed

### Git & PR Preferences:
- Detailed commit messages explaining Problem/Solution/Changes.
- Only commit your own changes (don't bundle skill changes if not asked).

### Code Quality Preferences:
- 100% test pass rate required.
- No compiler or clippy warnings.
- Universal Parser must support all features (highlighting, icons).

## Code Patterns Used
- **Anchor-based status detection**: `(?s).*[MARKER]\s*(?:[PROMPT_PATTERN])*\z`
- **Rule Explainability**: `explain_status` method in traits to expose inner matching logic for debugging.

## Notes
The detection engine is now very robust. The addition of the debug flag `-d` to the test command is a significant improvement for future maintenance.
