# Architecture

**Analysis Date:** 2026-01-30

## Pattern Overview

**Overall:** Three-layer async event-driven TUI dashboard with background polling monitor

**Key Characteristics:**
- Layered architecture separating tmux interaction, agent parsing, and UI rendering
- Async tokio-based monitor task polling tmux in background
- Plugin-style parser registry matching panes to agent parsers
- State mutations driven by user input and monitor updates
- Ratatui-based responsive TUI with dynamic layout

## Layers

**tmux Layer:**
- Purpose: Execute tmux commands and extract pane metadata
- Location: `src/tmux/`
- Contains: `TmuxClient` for command execution, `PaneInfo` for pane metadata and process detection
- Depends on: System `ps` command, configuration (capture_lines, detached_sessions)
- Used by: `MonitorTask` for polling, UI for sending commands

**Parser Layer:**
- Purpose: Extract agent status from pane content using regex patterns
- Location: `src/parsers/`
- Contains: `AgentParser` trait defining detection and parsing interface, `UniversalParser` implementation, `ParserRegistry` for matching panes to parsers
- Depends on: Configuration agent patterns, `regex` crate
- Used by: `MonitorTask` to convert pane content into `AgentStatus` and `Subagent` structures

**Agent Data Layer:**
- Purpose: Define agent state and metadata
- Location: `src/agents/`
- Contains: `MonitoredAgent` (pane + status), `AgentStatus` enum (Idle/Processing/AwaitingApproval/Error/Unknown), `Subagent` for spawned tasks, `ApprovalType` for approval kinds
- Depends on: Parser output
- Used by: Application state, UI rendering

**Application Layer:**
- Purpose: Maintain application state and coordinate between monitor and UI
- Location: `src/app/`
- Contains: `AppState` (main state), `Config` (TOML-based configuration), `Action` enum (user actions), `KeyBindings` (keyboard mapping)
- Depends on: Agent layer, configuration files
- Used by: UI event loop, monitor task

**Monitor Task:**
- Purpose: Poll tmux periodically and update application state
- Location: `src/monitor/task.rs`
- Contains: `MonitorTask` that runs as separate tokio task, sends `MonitorUpdate` messages through mpsc channel
- Depends on: tmux layer, parser layer, configuration
- Used by: UI event loop receives updates through channel

**UI Layer:**
- Purpose: Render application state and handle user input
- Location: `src/ui/`
- Contains: Event loop (`run_app`, `run_loop`), widget components, layout calculation, styling
- Depends on: Application layer, ratatui, crossterm
- Used by: Main entry point

## Data Flow

**Initialization:**
1. `main.rs` parses CLI arguments and loads/merges configuration
2. `run_app()` creates `AppState`, `TmuxClient`, `ParserRegistry`
3. Background `MonitorTask` is spawned with mpsc channel
4. Main event loop starts, connecting to monitor updates and user input

**Monitor Update Cycle:**
1. `MonitorTask::run()` sleeps for `poll_interval_ms` (default 500ms)
2. Calls `TmuxClient::list_panes()` to get current panes
3. For each pane:
   - Calls `TmuxClient::capture_pane()` to get content
   - `ParserRegistry::find_parser_for_pane()` matches pane to parser based on detection strings
   - Parser's `parse_status()` extracts `AgentStatus`
   - Parser's `parse_subagents()` extracts running tasks
3. Constructs `MonitorUpdate` with new `AgentTree`
4. Sends through `tx` channel to UI event loop

**UI Render Cycle:**
1. Event loop polls for user input (keyboard, mouse)
2. Receives monitor updates from `rx` channel
3. Applies actions from input to `AppState` (selection, scrolling, approvals)
4. Optionally applies `MonitorUpdate` (if new panes detected, status changed)
5. Renders all UI components using current `AppState`
6. Clears screen and draws frame using ratatui

**Approval Flow:**
1. User presses approval key (default `y`)
2. Action maps to `Action::Approve`
3. Handler iterates selected agents (or current agent)
4. For each agent with `AwaitingApproval` status:
   - Gets approval_keys from parser
   - Calls `TmuxClient::send_keys()` to send approval to pane
5. UI continues polling for status change (approver may still be working)

## State Management

**Atomic Unit:** `AppState` contains entire application state (agents, selection, filters, input buffer, popup state)

**Updates Come From:**
- Monitor task (agents tree, status changes) via channel
- User input (selection, navigation, input buffer) via direct action dispatch
- Config (colors, sidebar width, key bindings) - loaded once at startup

**Selection Persistence:**
Multiple layers of tracking allow selection to survive pane renames/restarts:
1. **Unique ID** (primary): session:window.pane-pid constructed at detection
2. **PID fallback** (secondary): If ID changes but PID matches, reuse selection
3. **Target fallback** (tertiary): If both change but tmux target (session:window.pane) matches, reuse selection

**Filtering:**
- `AppState.filter_pattern`: Optional string filter on agent names/sessions
- `AppState.filter_active`: Only show agents needing attention
- `AppState.filter_selected`: Only show multi-selected agents
- `AppState.visible_indices`: Cached filtered indices for navigation

## Key Abstractions

**AgentParser Trait:**
- Purpose: Define interface for agent-specific parsing logic
- Location: `src/parsers/mod.rs`
- Implementations: One concrete `UniversalParser` supporting all agents via config
- Pattern: Regex-based status detection with priorities (approval markers over activity markers)

**ParserRegistry:**
- Purpose: Match panes to parsers at runtime
- Location: `src/parsers/mod.rs`
- Pattern: Build registry from config, match using detection strings and match strength ranking

**MonitorUpdate Channel:**
- Purpose: Decouple monitor task from UI event loop
- Location: `src/ui/app.rs` creates channel, `src/monitor/task.rs` sends updates
- Pattern: Unbuffered mpsc channel (size 32) with opt-in update application in render cycle

**Splitter Model (Parsing):**
- Purpose: Avoid noise from terminal history when detecting status
- Pattern: Primary rule (e.g., separator line) divides buffer into body (agent output) and prompt (interactive UI). Refinements target specific groups/locations. Status markers anchored to end.
- Priority: Approval markers (Rule 0) always higher than activity markers to catch user-facing prompts first

## Entry Points

**CLI Entry:**
- Location: `src/main.rs`
- Triggers: Binary invocation `tmuxx [OPTIONS] [COMMAND]`
- Responsibilities: Parse CLI args, load config, initialize logging, dispatch to run_app or subcommands

**Application Start:**
- Location: `src/ui/app.rs::run_app()`
- Triggers: Called from main after config setup
- Responsibilities: Initialize terminal, create state, spawn monitor task, run event loop

**Event Loop:**
- Location: `src/ui/app.rs::run_loop()`
- Triggers: Called from run_app
- Responsibilities: Poll for user input and monitor updates, dispatch actions, render UI

**Background Monitor:**
- Location: `src/monitor/task.rs::MonitorTask::run()`
- Triggers: Spawned as separate tokio task
- Responsibilities: Poll tmux periodically, parse pane content, send updates to UI

## Error Handling

**Strategy:** Result types with anyhow for error propagation, graceful degradation in parsing

**Patterns:**
- `Command execution`: Returns `anyhow::Result`, logged at monitor level, displayed as status message
- `Configuration loading`: Falls back to defaults if file missing, exits with error if invalid
- `Parser failures`: Silent (returns `AgentStatus::Unknown`), no breaking
- `Tmux unavailable`: Detected at startup, warning shown to user

## Cross-Cutting Concerns

**Logging:** `tracing` crate with file output via `--debug` flag, no console logging (preserves TUI)

**Validation:** Config file validation via serde deny_unknown_fields, CLI arg parsing via clap derive

**Authentication:** Not applicable (local tmux interaction only)

**Configuration:** TOML file at `~/.config/tmuxx/config.toml` merged with CLI args and defaults via `Config::load_merged()`

**Concurrency:**
- Monitor task runs in separate tokio task
- AppState accessed immutably in render, mutably only in action dispatch
- No shared mutable state except Arc<AtomicBool> for user_interacted flag

---

*Architecture analysis: 2026-01-30*
