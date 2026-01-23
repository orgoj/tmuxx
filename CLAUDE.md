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

## Version and Publishing

- Version in `Cargo.toml`: Semantic versioning (currently 0.1.7)
- Published to crates.io (repository: nyanko3141592/tmuxcc)
- Git workflow: Work on `orgoj` branch, allow dirty publish for Cargo.lock changes
