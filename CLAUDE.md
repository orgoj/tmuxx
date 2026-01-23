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

## Development Workflow

### Testing

- **Test sessions**: `cc-test` (main TUI test), `cc-tmuxcc` (this app testing), or create custom with scripts
- **Test scripts**:
  - `scripts/cp-bin.sh` - Install tmuxcc to ~/bin after build (DON'T USE - user has working version there!)
  - `scripts/reload-test.sh` - Reload tmuxcc in ct-test session
  - `scripts/start-test-session.sh` - Start ct-test session
  - `scripts/setup-multi-test.sh` - Setup ct-multi session with multiple windows
- **CRITICAL: YOU test yourself using tmux-automation skill!**
  - Use `./target/release/tmuxcc` for testing (never cp-bin.sh)
  - Use `scripts/reload-test.sh` to reload tmuxcc in ct-test session
  - Use tmux-automation skill to interact with TUI and verify behavior
  - **NEVER ask user to test** - testing is YOUR responsibility
  - **NEVER claim completion without runtime verification** - visual verification mandatory for UI features
- **NEVER kill test sessions!** Use scripts to reload, not kill and recreate

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

- ✅ Code files: English (comments, variable names, function names)
- ✅ Documentation: English (README.md, CHANGELOG.md, CLAUDE.md)
- ✅ Git commits: English (commit messages)
- ✅ Comments in code: English
- ❌ NEVER use Czech in code files or documentation

**Exceptions (Czech allowed):**
- TODO.md (internal notes, user's native language)
- Diary entries in `.claude/diary/` (session notes)
- Implementation plans in `.claude/plans/` (working documents)

**Why:**
- Project is public fork of English codebase
- English is standard for open source projects
- Makes code accessible to wider audience
- Maintains consistency with upstream

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
