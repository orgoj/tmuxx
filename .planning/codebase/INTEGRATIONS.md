# External Integrations

**Analysis Date:** 2026-01-30

## APIs & External Services

**Tmux CLI:**
- tmux (system command-line utility)
  - What it's used for: List panes, capture pane content, send keys to panes, manage sessions
  - SDK/Client: Direct process execution via `std::process::Command`
  - Location: `src/tmux/client.rs` - `TmuxClient` struct

**Desktop Notifications:**
- Desktop notification system (Linux: notify-send, macOS: osascript, etc.)
  - What it's used for: Alert user when agents need approval or encounter errors
  - SDK/Client: Configurable via `notification_command` in config
  - Template placeholders: `{title}`, `{message}`, `{agent}`, `{session}`, `{target}`, `{path}`, `{approval_type}`, `{count}`
  - Auth: None (system-level command execution)
  - Location: `src/monitor/task.rs` - notification logic

## Data Storage

**Databases:**
- None - Application is stateless regarding external storage

**File Storage:**
- Local filesystem only
  - Configuration: `~/.config/tmuxx/config.toml`
  - Log output: `tmuxx.log` (in working directory when `--debug` flag used)
  - TODO files: Discovered from project root via glob patterns (configurable via `todo_files` list)

**Caching:**
- None (persistent external cache)
- In-memory caching only:
  - Agent tree state (`AppState.agents`)
  - System stats collected from `sysinfo` crate
  - External TODO command output (cached with `todo_refresh_interval_ms` configuration)

## Authentication & Identity

**Auth Provider:**
- None - Application does not perform authentication
- Operates with credentials of user running tmuxx process
- Tmux commands execute with user's tmux session permissions

## Monitoring & Observability

**Error Tracking:**
- None (no error reporting service)
- Local tracing only via `tracing` crate

**Logs:**
- Debug logging to file: `tmuxx.log` in current working directory (when `--debug` flag used)
- Structured logging via `tracing` and `tracing-subscriber` crates
- Log level: `DEBUG` when enabled
- Log output format: Plain text, no ANSI codes in file
- Environment filtering: `RUST_LOG` environment variable controls tracing output

**Terminal Output:**
- Status bar messages (user feedback in TUI)
- Action logging: Logged to status bar when `log_actions` config is true (default)

## CI/CD & Deployment

**Hosting:**
- GitHub (repository hosting)

**CI Pipeline:**
- GitHub Actions (`.github/workflows/ci.yml`)
- Jobs:
  1. **Check**: `cargo check` on Ubuntu latest
  2. **Test**: `cargo test` on Ubuntu latest
  3. **Format**: `cargo fmt --all -- --check` on Ubuntu latest
  4. **Clippy**: `cargo clippy -- -D warnings` on Ubuntu latest (zero warnings enforced)
  5. **Build**: `cargo build --release` on Ubuntu latest and macOS latest

**Release Pipeline:**
- GitHub Actions (`.github/workflows/release.yml`)
- Builds release artifacts for publishing
- Supports crates.io distribution (`tmuxx` package name)

## Environment Configuration

**Required env vars:**
- None (application is fully configurable via config file and CLI args)

**Optional env vars:**
- `RUST_LOG` - Controls tracing/logging output level (e.g., `RUST_LOG=debug`)
- User's shell environment variables: Available for template substitution in `execute_command` actions
  - Supported placeholders: `${SESSION_DIR}`, `${SESSION_NAME}`, `${PANE_TARGET}`

**Secrets location:**
- None - Application does not handle secrets
- User-provided command templates may reference sensitive data (responsibility of user)
- Example: `execute_command { command = "git gui" }` runs git GUI without authentication

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- Desktop notification command execution (one-way system call, not a webhook)
- External command execution via user-configured key bindings and menu items
- TODO command execution (external command configured via `todo_command` in config)

## External Command Execution

**Process Management:**
- Configurable external commands via key bindings (`execute_command` action in `key_bindings` section)
- Example commands from `config.example.toml`:
  - `zed -n ${SESSION_DIR}/TODO.md` - Open editor
  - `lazyclaude -m --directory '${SESSION_DIR}'` - Run CLI tool
  - `git gui` - Launch Git GUI
  - `wezterm start -- bash -lc 'tmux attach -t ${PANE_TARGET}'` - Open terminal
  - `notify-send "{title}" "{message}"` - Send notifications

**Variables Available for Substitution:**
- `${SESSION_DIR}` - Project directory for the session
- `${SESSION_NAME}` - Tmux session name
- `${PANE_TARGET}` - Tmux pane target (e.g., `session:window.pane`)

**Terminal Wrapping:**
- Optional `terminal_wrapper` config: Execute commands in external terminal
- Example: `"wezterm start -- bash -lc '{cmd}'"`
- Placeholder: `{cmd}` - Replaced with the actual command

## Process Detection & Monitoring

**Agent Detection Strings** (from pane metadata):
- Pane command (e.g., `claude`, `opencode`)
- Window title (e.g., `Claude Code 🌟`)
- Full command line (e.g., `/usr/bin/node /usr/bin/claude`)
- Child process commands (for agents run in shells)

**Parser Registry** (`src/parsers/`):
- UniversalParser - Matches agents against pane detection strings
- Each AI agent has configurable detection patterns (regex or literal strings)
- Parser returns `AgentStatus`: Idle, Processing, AwaitingApproval, Error, Unknown

## System Integration

**System Resources:**
- CPU usage monitoring via `sysinfo::System` crate
- Memory usage monitoring via `sysinfo::System` crate
- Display in header: CPU%, Memory (e.g., "CPU: 12% | MEM: 8.2G/16.0G")

**Terminal Integration:**
- Keyboard input via `crossterm` (non-blocking event handling)
- Terminal color support detection (24-bit truecolor, 256-color, basic fallback)
- Terminal size detection for responsive layout

**Process Integration:**
- Tmux session enumeration via `tmux list-panes -a`
- Pane content capture via `tmux capture-pane -p`
- Send keys to panes via `tmux send-keys`
- Kill panes/processes via configurable kill methods (SIGTERM, SIGKILL)
- Child process detection via `/proc` (Linux) or `libc` system calls

---

*Integration audit: 2026-01-30*
