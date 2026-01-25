# Reflection: Last 3 Entries

**Generated**: 2026-01-25
**Entries Analyzed**: 3
**Date Range**: 2026-01-24 to 2026-01-25

## Summary

Three diary entries reveal strong patterns around **technical review discipline**, **library trait usage**, and **tmux automation patterns**. The user consistently provides detailed technical corrections that catch implementation mistakes before coding begins, emphasizing the importance of thorough exploration and verification.

Key insight: **Most mistakes occur when skipping verification steps** - not checking if API methods exist, not reading library docs thoroughly, or making assumptions about integration patterns. The user's technical reviews consistently catch 5-7 issues per plan, suggesting that implementation discipline is more valuable than speed.

## Patterns Identified

### Strong Patterns (3+ occurrences)

1. **Technical Review is Critical** (3/3 entries)
   - **Observation**: Every session involved user catching multiple technical mistakes in plans/code
     - Entry 1: "kurva jak ti mohli projit testy" - unit tests passed but config loading failed
     - Entry 2: User caught 7 major issues in implementation plan (key conversion, action design, API methods)
     - Entry 3: "videl jsi ten vystup" - misanalyzed debug output, session creation vs attach confusion
   - **CLAUDE.md rule**: `plans: Always have implementation plans reviewed before coding - expect 5-7 corrections`

2. **User Provides Czech Feedback, English Code** (3/3 entries)
   - **Observation**: User converses in Czech ("videl jsi ten vystup", "to je spatne") but expects all code/docs in English
     - Entry 1: "kurva jak ti mohli projit testy" (feedback in Czech, code in English)
     - Entry 2: Design discussion in Czech, plan file in English
     - Entry 3: Czech corrections ("to ma poustet v tmux bash"), English git commits
   - **CLAUDE.md rule**: `language: User converses in Czech - OK; code/docs MUST be English - auto-correct if not`

3. **Library/Trait Verification is Essential** (2/3 entries - emerging pattern)
   - **Observation**: Mistakes occurred from not checking library capabilities before implementation
     - Entry 1: Unit tests passed but real config loading failed (serde `deny_unknown_fields` missing)
     - Entry 2: Tried to write manual key conversion instead of using `Into<Input>` trait that already existed
     - Entry 3: Wrong mental model about tmux session lifecycle
   - **CLAUDE.md rule**: `library research: Check library docs/trait implementations BEFORE coding - verify methods exist in codebase`

### Emerging Patterns (2 occurrences)

1. **Integration Strategy Clarification Required** (2/3 entries)
   - **Observation**: Assumptions about integration led to wrong approaches
     - Entry 2: Should modal replace or coexist with existing input? (User clarified: coexist)
     - Entry 3: Direct execution vs send-keys pattern? (User clarified: send-keys)
   - **CLAUDE.md rule**: `integration: Ask clarifying questions before planning - "Should X replace or coexist with Y?"`

2. **TOML/Config Pitfalls** (2/3 entries)
   - **Observation**: Configuration issues caused runtime failures
     - Entry 1: `deny_unknown_fields` missing, silent config parsing errors
     - Entry 2: Config option `hide_bottom_input` defaults to true
   - **CLAUDE.md rule**: `config: Use serde deny_unknown_fields - test with actual config.toml, not just unit tests`

### Rule Violations Detected

**Violation of "Testing Discipline" rule** (Entry 1):
- Rule: "INVOKE tmuxcc-testing skill: MANDATORY before ANY testing"
- Violation: "Used tail with tmux capture-pane" - ignored tmuxcc-testing safety rules
- Action: Strengthen rule to emphasize "One key at a time" and "no tail/head with tmux"

**Violation of "Skill-First Development" rule** (Entry 2):
- Rule: "Check skills BEFORE starting"
- Violation: "Skipped verification of API methods" - didn't check if method exists
- Action: Add "Always verify method existence in codebase before writing plans"

## Proposed CLAUDE.md Updates

### Common Pitfalls section (add to existing list):

```markdown
17. **Assumption-Based Planning**: Ask "Should X replace or coexist with Y?" before designing integration
18. **Method Existence Verification**: Always check codebase for actual method patterns before referencing them
```

### Development Workflow → Plan Review and Correction (strengthen existing):

**Current text:**
> Expect user technical review to catch edge cases (off-by-one, API misuse, config integration)

**Add:**
> - **Expect 5-7 corrections**: User consistently finds multiple issues per plan
> - **No placeholders accepted**: All code in plans must be complete, not "TODO" comments
> - **Trait implementations**: Check if library provides `Into`/`From` before writing manual conversion

### Testing Discipline (strengthen existing):

**Current text:**
> - **One key at a time**: Send ONE key, capture output, verify, then next - prevents destructive commands

**Add:**
> - **Never use tail/head with tmux capture-pane** - could show partial destructive commands
> - **Always test with actual config files** - unit tests ≠ working runtime config loading

### Technical Preferences (add new section after User Preferences Observed):

```markdown
### Technical Preferences:
- **Send-keys pattern for tmux automation**: Create bash session → send-keys to launch tool
- **Full path resolution**: Use `command -v` to prevent PATH issues in tmux sessions
- **Session persistence**: Sessions should remain alive even if main tool crashes
- **Coexistence over replacement**: Keep existing functionality, add new as alternative
- **Config options with defaults**: Boolean options like `hide_bottom_input = true` enable user choice
```

## One-Off Observations

### Entry 1 (Command execution feature):
- Created `tmuxcc-gemini-review` skill - new workflow for AI code review
- UTF-8 panic from byte slicing on ✓ character (3 bytes)
- Multi-line command keybindings use heredoc or escaped newlines

### Entry 2 (Modal textarea design):
- Modal pattern: `Clear` widget + centered layout with `Flex::Center`
- Agent access pattern: `state.agents.get_agent(state.selected_index)` not `state.selected_agent()`
- `tui-textarea` library provides `Into<Input>` trait for key conversion

### Entry 3 (Wrapper script fix):
- Debug mode (`bash -x`) essential for diagnosing issues
- Wrapper script is symlink: `~/bin/tcc` → `scripts/tmuxcc-wrapper.sh`
- Commit format: Problem/Solution/Changes with Co-Authored-By footer

## Metadata

- **Entries analyzed**:
  1. `2026-01-24-12-01-9c0719cb-8223-4269-a831-5e2c701a4285.md`
  2. `2026-01-24-woolly-foraging-rainbow.md`
  3. `2026-01-25-14-35-c1424eea-e4e0-4bd6-a295-0c10a64aaad0.md`

- **Skills mentioned**:
  - `tmuxcc-testing` (Entry 1)
  - `tmuxcc-commit` (Entry 1, Entry 3)
  - `tmuxcc-gemini-review` (Entry 1 - created)
  - `superpowers:brainstorming` (Entry 2)

- **Key technical patterns identified**:
  1. Serde `deny_unknown_fields` for config validation
  2. `Into<Input>` trait for key conversion
  3. Send-keys pattern for tmux automation
  4. UTF-8 safe string slicing with `.chars()`
  5. Modal pattern with `Clear` widget + centered layout
