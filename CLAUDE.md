# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TmuxCC is a Rust TUI (Terminal User Interface) application that monitors multiple AI coding agents (Claude Code, OpenCode, Codex CLI, Gemini CLI) running in tmux panes. It provides a centralized dashboard for tracking agent status, managing approvals, and viewing subagents.

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
   - Each AI agent has a dedicated parser (ClaudeCodeParser, OpenCodeParser, etc.)
   - `AgentParser` trait: `matches()`, `parse_status()`, `parse_subagents()`, `parse_context_remaining()`
   - Parsers use regex patterns to detect approval prompts, subagents, and status
   - `ParserRegistry` matches panes to appropriate parsers

3. **Application Layer** (`src/app/`):
   - `AppState`: Main application state (agent tree, selection, input buffer)
   - `AgentTree`: Hierarchical structure of root agents and their subagents
   - `Config`: TOML-based configuration (poll interval, capture lines, custom patterns)
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

`ApprovalType` variants:
- `FileEdit/FileCreate/FileDelete`: File operations
- `ShellCommand`: Bash command execution
- `McpTool`: MCP server tool calls
- `UserQuestion { choices, multi_select }`: AskUserQuestion tool with options

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

### Async Monitoring (`src/monitor/task.rs`)

- Background tokio task polls tmux at configured intervals (default: 500ms)
- Captures pane content, parses status, updates shared AppState
- Uses `Arc<Mutex<AppState>>` for thread-safe state updates
- SystemStats module tracks CPU/memory for header display

## Important Implementation Details

### Process Detection Strategy

`PaneInfo::detection_strings()` returns multiple candidates:
1. Pane command (e.g., "claude")
2. Window title (e.g., "Claude Code 🌟")
3. Full cmdline (e.g., "/usr/bin/node /usr/bin/claude")
4. Child process commands (for agents run in shells)

Parsers check ALL detection strings to handle various detection scenarios.

### Multi-Agent Selection

- `AppState.selected_agents: HashSet<usize>` tracks multi-selection
- Space key toggles selection, Ctrl+a selects all
- Batch operations (y/n for approval/rejection) iterate selected agents
- Selected agents highlighted in UI with different colors

### Input Buffer Design

- Always-visible input buffer at bottom (`AppState.input_buffer`)
- `FocusedPanel` enum switches between Sidebar (navigation) and Input (typing)
- Left/Right arrow keys toggle focus
- Input buffer persists across focus changes for quick responses

### Configuration System

- Uses TOML format (`~/.config/tmuxcc/config.toml` on Linux)
- `Config::load()` merges CLI args > config file > defaults
- Custom agent patterns can be added via `[[agent_patterns]]` sections
- `--init-config` creates default config, `--show-config-path` shows location

## Testing Guidelines

- Unit tests in each module (`#[cfg(test)] mod tests`)
- Parser tests verify regex patterns match expected formats
- Mock `PaneInfo` structures for parser testing
- No integration tests yet (would require tmux session)

## Common Pitfalls

1. **Regex Performance**: Parsers run on every poll cycle. Keep regex patterns efficient.
2. **Unicode Safety**: Use `unicode-width` crate for text truncation (paths, titles)
3. **tmux Escaping**: Pane content may contain ANSI codes - parsers handle raw text
4. **Child Process Detection**: Agents run in shells need child process scanning
5. **Multi-byte Characters**: Use `safe_tail()` helper for safe string slicing
6. **Upstream Data Problems**: When a feature "doesn't work", check if upstream data exists FIRST (e.g., agent detection before testing focus functionality)

## Development Workflow

### Testing

- **Test sessions**: `ct-test` (for tmuxcc testing), `cc-test` (main TUI test), `cc-tmuxcc` (other testing)
- **ONLY USE EXISTING SESSIONS** - NEVER create random tmux sessions!
- **Test scripts**:
  - `scripts/cp-bin.sh` - Install tmuxcc to ~/bin after build (DON'T USE - user has working version there!)
  - `scripts/reload-test.sh` - Reload tmuxcc in ct-test session
  - `scripts/start-test-session.sh` - Start ct-test session
  - `scripts/setup-multi-test.sh` - Setup ct-multi session with multiple windows
- **CRITICAL: YOU test yourself using tmux-automation skill - INVOKE IT, don't just mention it!**
  - Use `./target/release/tmuxcc` for testing (never cp-bin.sh)
  - Use `scripts/reload-test.sh` to reload tmuxcc in ct-test session
  - **INVOKE tmux-automation skill with Skill tool** - don't skip this step!
  - Use tmux-automation skill to interact with TUI and verify behavior
  - **NEVER ask user to test** - testing is YOUR responsibility
  - **NEVER claim completion without runtime verification** - visual verification mandatory for UI features
- **NEVER kill test sessions!** Use scripts to reload, not kill and recreate

**CRITICAL TMUX SAFETY RULES (NON-NEGOTIABLE):**

1. **NEVER use tail/head with capture-pane!**
   - ❌ WRONG: `tmux capture-pane -t session -p | tail -30`
   - ✅ CORRECT: `tmux capture-pane -t session -p`
   - **Why**: Line 31 could be `reboot` or other destructive command!

2. **Empty capture = ERROR state → STOP IMMEDIATELY!**
   - If `capture-pane -p` returns empty → DON'T send any commands
   - Check session exists, check for errors
   - **NEVER proceed without visible output**

3. **No bash prompt = ERROR state → STOP IMMEDIATELY!**
   - If you don't see `$`, `>`, or clear input prompt → DON'T send commands
   - Something is wrong with the session
   - **NEVER blindly send Enter or other keys**

4. **ALWAYS capture FULL screen first to understand state:**
   ```bash
   tmux capture-pane -t ct-test -p  # Full screen, no tail!
   ```

5. **Check what you're doing BEFORE sending keys:**
   - Capture full screen
   - Verify prompt is visible
   - Verify expected state
   - ONLY THEN send commands

6. **NEVER send keys to session where tmuxcc is RUNNING!**
   - ❌ FATAL: `tmux send-keys -t ct-test "y"` when tmuxcc runs there
   - **Why**: tmuxcc forwards keys to monitored sessions → unintended approvals!
   - ✅ CORRECT: Use dedicated test session WITHOUT tmuxcc for interactive testing
   - **Testing tmuxcc**: Only capture output, NEVER send keys to ct-test!

### Problem Diagnosis

- **Run diagnostic commands FIRST**: Before analyzing code, gather real data (e.g., `tmux list-panes -a` when debugging detection)
- **Check upstream problems**: Feature failures often indicate missing input data, not broken feature logic
- **Integrate related problems**: Multiple related issues → ONE cohesive plan, not separate fixes or replacements

### Key Principles from tmuxclai-arch

- **Unicode Safety**: Use `unicode-width` crate for text truncation, NEVER use byte slicing `&str[..n]`
- **Temporary files**: Use `./tmp/` in project, never system `/tmp/`
- **Code duplication**: ZERO TOLERANCE - consolidate duplicate methods/structures
- **tmux commands**: ALWAYS verify behavior manually before coding assumptions
- **Debug workflow**: Add visible debug to UI (status bar), not just file writes
- **Build release for testing**: `cargo build --release` before claiming done
- **Clean up warnings**: Run `cargo clippy` and fix all warnings

### Library Research Workflow

**CRITICAL: NEVER write functionality from scratch when libraries exist!**

**Before implementing ANY feature:**
1. **WebSearch** for current libraries (use year 2026 in query)
2. **rtfmbro MCP** to get README/docs of selected library
3. Study examples from library repo
4. Only then implement using the library

**Example workflow (modální text editor):**
```bash
# 1. Search for libraries
WebSearch: "rust ratatui text editor widget library 2026"

# 2. Get documentation
mcp__rtfmbro__get_readme package="rhysd/tui-textarea" version="*" ecosystem="gh"

# 3. Check examples in repo
# 4. Implement using library
```

**Selected libraries for tmuxcc:**
- **Text editing:** tui-textarea (rhysd) - supports ratatui 0.29, has popup example

### Ratatui Documentation

**ALWAYS consult ratatui documentation via rtfmbro MCP BEFORE implementing UI features!**

Project uses **ratatui 0.29** - complete documentation available via MCP:
```bash
# Get README with quickstart:
mcp__rtfmbro__get_readme package="ratatui/ratatui" version="==0.29" ecosystem="gh"

# Get documentation tree:
mcp__rtfmbro__get_documentation_tree package="ratatui/ratatui" version="==0.29" ecosystem="gh"
```

### Common Pitfalls

1. **tmux command assumptions**: Test manually first, verify actual output
2. **Implementation from memory**: Research current docs, don't guess
3. **Testing in wrong environment**: Use ct-multi (5 windows) for multi-window features
4. **Over-engineering**: Remove complexity instead of fixing it when possible
5. **NEVER edit config files without explicit user permission** - Only create/modify ~/.config/tmuxcc/* when user explicitly asks for it

### Git Workflow

**CRITICAL: Pre-commit checklist (NON-NEGOTIABLE):**

Before EVERY commit with new features/config options:
1. ✅ **Update CHANGELOG.md** - Add feature to Unreleased section
   - Describe what was added/changed/fixed
   - Include config options with defaults
   - Include CLI override examples
2. ✅ **Update README.md** - Add config options to Configuration section
   - Add to config.toml example with comments
   - Add to "Available config keys" list with description
   - Update default values if changed
3. ✅ **Build and test** - cargo build --release, cargo clippy, cargo fmt
4. ✅ **Stage all changes** - git add <files>
5. ✅ **Write commit message** - Clear description with Co-Authored-By

**If you skip documentation updates, commit will be REJECTED!**

**Commit message format:**
```
feat: Brief description (imperative mood)

Problem: What issue this solves
Solution: How it was solved (bullet points)

Changes:
- File changes
- Config updates
- Documentation updates

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

- **ALWAYS use `git add -A`** unless explicitly told otherwise by user
- When staging files, prefer adding specific files by name is WRONG - use `git add -A`

**Remotes:**
- `origin` - git@github.com:orgoj/tmuxcc.git (main fork)
- `original` - git@github.com:nyanko3141592/tmuxcc.git (upstream)
- `neon` - git@github.com:frantisek-heca/tmuxcc-neon.git (tracking)

### TODO.md and CHANGELOG.md Management

**CRITICAL: Keep TODO.md clean - completed tasks don't belong there!**

When a task is completed:
1. **Move to CHANGELOG.md** - Document what was done with proper detail
2. **Delete from TODO.md** - Don't leave completed tasks in TODO
3. **Mark as ✅ COMPLETED** only temporarily if needs verification, then move to CHANGELOG

**Why:**
- TODO.md is for ACTIVE work - what needs doing
- Completed tasks haunting TODO confuse future sessions
- CHANGELOG.md is the proper place for completed work history
- Keep TODO focused on next steps, not past achievements

**Example workflow:**
```
Task done → Update CHANGELOG.md → Delete from TODO.md → Git commit
```

**Don't:**
- ❌ Leave tasks marked "✅ COMPLETED" in TODO.md long-term
- ❌ Accumulate completed tasks at the top of TODO.md
- ❌ Use TODO.md as a changelog

**Do:**
- ✅ Move completed work to CHANGELOG.md immediately
- ✅ Keep TODO.md focused on current/upcoming work
- ✅ Use "Completed Tasks ✅" section only as temporary staging before CHANGELOG move

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
- ✅ Files WITHOUT language extension (`.cs`, `.ja`, etc.) → MUST be English
- ✅ Files WITH language extension → Can be in that language
  - `.cs` = Czech allowed
  - `.ja` = Japanese allowed
  - etc.

**Examples:**
- ❌ `.dippy` → MUST be English (no extension) OR rename to `.dippy.cs`
- ✅ `.dippy.cs` → Czech OK (has `.cs` extension)
- ❌ `notes` → MUST be English OR rename to `notes.cs`
- ✅ `notes.cs` → Czech OK

**Special exceptions (don't need `.cs`):**
- TODO.md (internal working notes - Czech OK)
- `.claude/diary/` (user's existing entries - don't translate old ones, new ones in English)
- `.claude/plans/` (user's existing entries - don't translate old ones, new ones in English)

**Action required for `.dippy`:**
- [ ] Option 1: Translate `.dippy` to English
- [ ] Option 2: Rename to `.dippy.cs` to mark as Czech content

**CRITICAL: Auto-correct Czech to English:**
- User writes in Czech in conversation → OK
- User writes Czech in CODE/DOCS → AI fixes IMMEDIATELY when seen in diff
- AI must check ALL diffs for Czech text in English-only files
- If Czech found in .rs, README.md, CHANGELOG.md, CLAUDE.md → Fix to English immediately
- User reads English well but writes Czech → AI translates for them
- Don't ask permission, just fix it in the same response

**Important:**
- **Upstream inheritance:** Original fork (nyanko3141592/tmuxcc) was Japanese → must translate ALL Japanese text
- CLI help was in Japanese → translate to English
- Error messages were in Japanese → translate to English
- This is NOT optional - project must be English for open source community

**Why:**
- Project is public fork - needs to be accessible globally
- English is standard for open source projects
- Makes code readable by wider audience
- Professional presentation for international community

## Project Context

**tmuxcc** - AI Agent Dashboard for tmux (fork of nyanko3141592/tmuxcc)

- **Origins**: Fork of tmuxcc, inspired by tmuxclai vision
- **Repository**: This is a fork - only push to `orgoj` branch, NO publishing to crates.io
- **Vision**: See IDEAS.md for roadmap and future features

## Version and Publishing

- Version in `Cargo.toml`: Semantic versioning (currently 0.1.7)
- This is a FORK - changes pushed to `orgoj` branch only
- NO publishing to crates.io (that's upstream's job)
- Git workflow: Work on `orgoj` branch, allow dirty publish for Cargo.lock changes
