# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tmuxx is a Rust TUI (Terminal User Interface) application that monitors tmux panes based on configuration. It features native support for AI coding agents (Claude Code, OpenCode, Codex CLI, Gemini CLI) but serves as a general-purpose dashboard for monitoring any process.

It is a hard fork and total rewrite of `tmuxcc`.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run locally (must have tmux running with agents)
cargo run

# Run with custom options
cargo run -- -p 1000 -l 200 --debug

# Run tests
cargo test

# Run lints (CI requirement - must pass)
cargo clippy

# Format code (CI requirement)
cargo fmt

# Install locally from source
cargo install --path .
```

## Core Architecture

### Three-Layer Design

1. **tmux Layer** (`src/tmux/`):
   - `TmuxClient`: Executes tmux commands (list-panes, capture-pane, send-keys)
   - `PaneInfo`: Parses tmux pane metadata and detects agent processes
   - Process detection uses cmdline, window title, and child processes

2. **Parser Layer** (`src/parsers/`):
   - `UniversalParser`: Single configurable parser that matches all agent types
   - Patterns organized by agent in `defaults.toml`, not separate parser implementations
   - `AgentParser` trait: `matches()`, `parse_status()`, `parse_subagents()`, `parse_context_remaining()`
   - `ParserRegistry` matches panes to appropriate parser patterns

3. **Application Layer** (`src/app/`):
   - `AppState`: Main application state (agent tree, selection, input buffer)
   - `AgentTree`: Hierarchical structure of root agents and their subagents
   - `Config`: Configuration struct (`src/app/config.rs`) with TOML defaults from `src/config/defaults.toml`
   - `config_override.rs`: CLI argument handling and overrides
   - `menu_config.rs`: Menu configuration
   - Actions flow: User input → `Action` enum → state mutations → UI update

### Key Data Flow

```
tmux panes → TmuxClient.list_panes() → PaneInfo
           → ParserRegistry.find_parser_for_pane()
           → parser.parse_status(content)
           → MonitoredAgent with AgentStatus
           → AppState.agents (AgentTree)
           → UI components render state
```

### Agent Status Detection

`AgentStatus` enum variants:
- `Idle`: No pending work
- `Processing { activity }`: Agent is working
- `AwaitingApproval { approval_type, details }`: User action required
- `Error { message }`: Agent encountered an error
- `Unknown`: Cannot determine status

#### Structural Splitter Model
State detection uses a "Splitter Model" to avoid noise from terminal history:
1. **Splitter**: A primary rule (e.g., `separator_line`, `powerline_box`) divides the buffer into `body` (agent response) and `prompt` (interactive UI).
2. **Refinements**: Granular rules target specific groups and `location` (e.g., `LastLine`, `FirstLineOfLastBlock`).
3. **Regex Anchoring**: Status markers (e.g., `?`) MUST be anchored to the end of the string (`\z`) after accounting for prompt lines.
4. **Priority**: Critical human interaction markers (Approvals) always have Rule 0 priority over background activity (Working).

### Subagent Tracking

Claude Code's Task tool spawns subagents (e.g., Explore, Plan agents). Detection:
- Parse `Task(...subagent_type="X"...description="Y"...)` patterns
- Track spinner indicators (`⠿⠇⠋⠙⠸⠴⠦⠧`) for running state
- Detect completion markers (`✓✔`) for finished state
- Stored in `MonitoredAgent.subagents: Vec<Subagent>`

### UI Architecture (`src/ui/`)

- `components/`: Modular UI components (agent_tree, header, footer, pane_preview, help, etc.)
- `layout.rs`: Calculates Ratatui layout constraints for responsive design
- `styles.rs`: Color schemes and styling constants
- `app.rs`: Main event loop (keyboard input → actions → state updates → rendering)

### CLI Command Layer (`src/cmd/`)

- **`test.rs`**: Regression test runner (`cargo run -- test [-d]` for debug mode)
- **`learn.rs`**: Auto-generates agent definitions from active tmux sessions
- These are invoked via CLI subcommands, not the TUI

### Async Monitoring (`src/monitor/`)

- **`task.rs`**: Background tokio task polls tmux at configured intervals (default: 500ms)
- **`system_stats.rs`**: Tracks CPU/memory usage for header display
- Captures pane content, parses status, updates shared AppState
- Uses `Arc<Mutex<AppState>>` for thread-safe state updates

## Important Implementation Details

### Process Detection Strategy

`PaneInfo::detection_strings()` returns multiple candidates:
1. Pane command (e.g., "claude")
2. Window title (e.g., "Claude Code 🌟")
3. Full cmdline (e.g., "/usr/bin/node /usr/bin/claude")
4. Child process commands (for agents run in shells)

Parsers check ALL detection strings to handle various detection scenarios.

### Multi-Agent Selection

- `AppState.selected_agents: HashSet<String>` tracks multi-selection by unique IDs
- Space key toggles selection, Ctrl+a selects all
- Batch operations (y/n for approval/rejection) iterate selected agents
- **Selection Persistence**: Selection survives monitor updates and renames using a fallback chain: Unique ID → PID → tmux Target.

### Input Buffer Design

- Always-visible input buffer at bottom (`AppState.input_buffer`)
- `FocusedPanel` enum switches between Sidebar (navigation) and Input (typing)
- Left/Right arrow keys toggle focus
- Input buffer persists across focus changes for quick responses

### Configuration System

- Main config struct: `src/app/config.rs` (merged from CLI args, user config file, and defaults)
- Default patterns: `src/config/defaults.toml`
- CLI overrides: `src/app/config_override.rs`
- Menu config: `src/app/menu_config.rs`
- User config location: `~/.config/tmuxx/config.toml` on Linux
- Merge order: CLI args > config file > defaults
- Custom agent patterns can be added via `[[agent_patterns]]` sections in defaults.toml
- `--init-config` creates default config, `--show-config-path` shows location

## Testing Guidelines

- Unit tests in each module (`#[cfg(test)] mod tests`)
- Parser tests verify regex patterns match expected formats
- Mock `PaneInfo` structures for parser testing
- No integration tests yet (would require tmux session)

### Testing Discipline

- **INVOKE tmuxx-testing skill**: MANDATORY before ANY testing - contains session structure, send-keys rules, tmux safety
- **100% Pass Rate**: 100% regression test pass rate (`cargo run -- test`) is mandatory before any commit.
- **Regression Verification**: ALWAYS run `cargo run -- test` after modifying source code, `defaults.toml`, or test fixtures (MANDATORY).
- **Debug Mode**: Use `cargo run -- test -d` to debug splitter/refinement matching issues.
- See tmuxx-testing skill for complete testing workflow and safety rules

## Common Pitfalls

1.  **Regex Performance**: Parsers run on every poll cycle. Keep regex patterns efficient.
2.  **Unicode Safety**: Use `unicode-width` crate for text truncation (paths, titles)
3.  **tmux Escaping**: Pane content may contain ANSI codes - parsers handle raw text
4.  **Child Process Detection**: Agents run in shells need child process scanning
5.  **Multi-byte Characters**: Use `safe_tail()` helper for safe string slicing; always use `.chars()` for character indexing, never byte slicing
6.  **Upstream Data Problems**: When a feature "doesn't work", check if upstream data exists FIRST (e.g., agent detection before testing focus functionality)
7.  **Config Integration Completeness**: When adding config options, verify they're actually used in implementation code, not just defined in Config struct
8.  **Ratatui Paragraph Wrapping**: To disable wrapping, omit `.wrap()` call entirely; don't use `Wrap { trim: true }` which controls trimming, not wrapping
9.  **HashMap Display Order**: HashMap iteration is non-deterministic - ALWAYS sort keys explicitly for UI consistency
10. **Trailing Content in UI**: When displaying "last N lines", trim trailing empty content first to ensure actual data is visible
11. **Skipping Skills**: Attempting tasks without invoking relevant skills leads to mistakes - check `.claude/skills/` first
12. **Batch Operations Without Verification**: Sending multiple commands at once can cause destructive failures - verify each step
13. **View/Model Index Mismatch**: When filtering affects display, navigation MUST use filtered indices - otherwise cursor lands on hidden items or skips erratically
14. **Single Method Fix Tunnel Vision**: When fixing one method, audit entire API for same pattern - often multiple methods have the same issue
15. **Designing Before Understanding Use Case**: Ask "what will this be used for?" before architecture design - specific use cases beat generic abstractions
16. **TODO.md Unauthorized Modification**: NEVER modify TODO.md without explicit user approval - when asked to "check" or "verify", report findings and ASK what to do next
17. **Planning without checking git status**: Always run `git status` and `git diff` before writing plans - implementation may already exist

## Development Workflow

### Skill-First Development

- **Check skills BEFORE starting**: Search `.claude/skills/` for relevant skills before implementing
- **INVOKE skills matching task type**: Testing? → tmuxx-testing. Commit? → tmuxx-committing-changes. Config? → tmuxx-adding-config-options
- **Create skills for repetitive workflows**: If explaining same process twice → create skill
- **Documentation extraction**: Keep CLAUDE.md under 300 lines - extract repetitive workflows into skills

### Project-Specific Skills

**CRITICAL: Project-specific skills have PRIORITY over generic skills!**

- **Project skills FIRST**: tmuxx-testing, tmuxx-committing-changes, etc. take precedence
- **Generic skills (tmux-automation, etc.)**: Only if no project skill exists for the task
- **Skill obligation applies to PROJECT skills**: The "must use skills" rule refers to these project skills

All skills are in `.claude/skills/`:

1. **`tmuxx-adding-config-options`** - Pattern for adding new config options with CLI override support
   - Use when adding bool, string, or number config options
   - Files: config.rs, config_override.rs, README.md, CHANGELOG.md

2. **`tmuxx-testing`** - Testing workflow and tmux safety rules
   - **INVOKE before testing!**
   - Test session structure, send-keys rules, tmux safety
   - Scripts: reload-test.sh, start-test-session.sh, setup-multi-test.sh

3. **`tmuxx-committing-changes`** - Pre-commit checklist and git workflow
   - **INVOKE before every commit!**
   - **Commit Format**: `<type>: description`, followed by Problem/Solution/Changes blocks.
   - CHANGELOG.md updates, README.md updates, cargo build/clippy/fmt
   - Git remotes and lock management

4. **`tmuxx-researching-libraries`** - Library research workflow
   - **INVOKE before implementing features!**
   - WebSearch for libraries, rtfmbro MCP for docs
   - Ratatui 0.29 documentation via MCP
   - Check trait implementations, verify method existence

5. **`tmuxx-managing-changelogs`** - TODO.md and CHANGELOG.md management
   - **INVOKE when completing tasks!**
   - Move completed work from TODO.md to CHANGELOG.md
   - Keep TODO.md clean and focused

6. **`tmuxx-planning`** - Implementation planning workflow
   - **INVOKE before writing code!**
   - Ask clarifying questions, explore codebase
   - Expect 5-7 corrections in review
   - Integration philosophy: coexistence over replacement

### Problem Diagnosis

- **Run diagnostic commands FIRST**: Before analyzing code, gather real data (e.g., `tmux list-panes -a` when debugging detection)
- **Check upstream problems**: Feature failures often indicate missing input data, not broken feature logic
- **Integrate related problems**: Multiple related issues → ONE cohesive plan, not separate fixes or replacements

### Plan Review and Correction Cycles

- **INVOKE tmuxx-planning skill** before writing implementation plans
- User provides precise corrections with exact code examples - apply them before implementation
- For complex bugs/features: use Task tool Explore → Plan workflow, expect user review phase before implementation begins
- See tmuxx-planning skill for complete planning workflow and common mistakes

### Key Principles

- **Strict Config-First**: Never hardcode interactive behaviors, keys, or regex rules; use `defaults.toml` and ensure everything is user-configurable.
- **Atomic Tmux**: Use single multi-argument `send-keys` calls to send key sequences (e.g., `y` + `Enter`) to avoid race conditions.
- **Tool Usage**: Strictly use CLI interfaces (like `dot` for tasks) instead of manual file/metadata edits in hidden directories.
- **Paths**: ALWAYS use relative paths (no absolute paths).
- **Tools**: Use `rg` (ripgrep) for searching, never `grep`.
- **Data Safety**: Never delete test fixtures/data; move or rename them if needed.
- **TUI Performance**: Use conditional redrawing (`needs_redraw`) and lazy data fetching to minimize CPU.
- **Git History**: Treat released CHANGELOG versions as immutable.
- **Temporary files**: Use `./tmp/` in project, never system `/tmp/`
- **Code duplication**: ZERO TOLERANCE - consolidate duplicate methods/structures
- **Debug workflow**: Add visible debug to UI (status bar), not just file writes
- **Build release for testing**: `cargo build --release` before claiming done
- **Clean up warnings**: Run `cargo clippy` and fix all warnings
- **Ratatui dynamic UI**: Prefer dynamic generation from config over hardcoded text - enables runtime customization
- **User feedback validation**: Runtime testing reveals UX issues code review misses - visual verification is CRITICAL
- **Implementation from memory**: Research current docs, don't guess (use `tmuxx-researching-libraries` skill!)
- **Testing environment**: Use ct-multi (5 windows) for multi-window features
- **Branch workflow for risky changes**: Use backup → feature → merge pattern for safe rollback path

### CRITICAL SAFETY (NON-NEGOTIABLE)

1.  **NEVER kill test sessions!** (`ct-test`, `ct-multi`, etc.)
2.  **NEVER kill user-started processes** (even if triggered by testing) unless EXPLICITLY ordered.
3.  **NEVER close the test tmux session!** It must run continuously for the user to see state.
4.  **DESTRUCTIVE ACTIONS**: Always ask before `kill`, `rm`, or `tmux kill-session`.

### Code and Documentation Language

**CRITICAL: Write ALL code files, documentation, and git commits in ENGLISH!**

**Rules for ALL code and documentation:**
- ✅ **Source code (.rs files):** English ONLY
  - Comments: English
  - Variable names: English
  - Function names: English
  - Error messages: English
  - CLI help text: English
- ✅ **Documentation:** English (README.md, CHANGELOG.md, CLAUDE.md)
- ✅ **Git commits:** English (commit messages)
- ❌ **NEVER use:** Japanese (日本語), Czech (čeština), Chinese (中文), or any other language in code
- ❌ **NO exceptions** for CLI help, error messages, or user-facing strings

**Language rules for ALL project files:**
- ✅ Files WITHOUT language marker → MUST be English
- ✅ Files WITH language marker → Can be in that language
  - Language marker can be: suffix, prefix, or in filename
  - Examples: `_cs`, `.cs`, `-cs`, `cs.`, `cs_`, etc.

**Valid language markers (flexible):**
- Markdown: `README.cs.md`, `README_cs.md`, `README-cs.md`
- Text: `notes_cs.txt`, `notes.cs.txt`, `notes-cs.txt`
- Config: `config_cs.toml`, `config.cs.toml`
- Any file: `filename_cs.ext`, `filename.cs.ext`, `filename-cs.ext`

**Examples:**
- ❌ `.dippy` → MUST be English (no marker) OR add marker: `.dippy_cs`, `.dippy.cs`, `.dippy-cs`
- ✅ `.dippy_cs` → Czech OK (has `_cs` marker)
- ✅ `.dippy.cs` → Czech OK (has `.cs` marker)
- ❌ `notes` → MUST be English OR add marker: `notes_cs`, `notes.cs`
- ✅ `notes_cs` → Czech OK
- ✅ `README.cs.md` → Czech OK (markdown with `cs` marker)
- ✅ `TODO_cs.md` → Czech OK (markdown with `_cs` suffix)

**Special exceptions (don't need `.cs`):**
- TODO.md (internal working notes - Czech OK)
- `.claude/diary/` (user's existing entries - don't translate old ones, new ones in English)
- `.claude/plans/` (user's existing entries - don't translate old ones, new ones in English)

**CRITICAL: Auto-correct Czech to English:**
- User writes in Czech in conversation → OK
- User writes Czech in CODE/DOCS → AI fixes IMMEDIATELY when seen in diff
- AI must check ALL diffs for Czech text in English-only files
- If Czech found in .rs, README.md, CHANGELOG.md, CLAUDE.md → Fix to English immediately
- User reads English well but writes Czech → AI translates for them
- Don't ask permission, just fix it in the same response

## Project Context

**tmuxx** - AI Agent Dashboard for tmux (originally tmuxcc)

- **Origins**: Hard fork and total rewrite of `tmuxcc`, inspired by `tmuxclai` vision
- **Repository**: `https://github.com/orgoj/tmuxx`
- **Vision**: See TODO.md for roadmap and future features

## Version and Publishing

- Version in `Cargo.toml`: Semantic versioning
- **Publishing**: Ready for crates.io release under `tmuxx` name
