# TmuxCC

**AI Agent Dashboard for tmux** - Monitor and manage multiple AI coding agents from a single terminal interface.

TmuxCC is a TUI (Terminal User Interface) application that provides centralized monitoring and control of AI coding assistants running in tmux panes. It supports Claude Code, OpenCode, Codex CLI, and Gemini CLI.

---

## Screenshot

<!-- TODO: Add actual screenshot -->
```
+------------------------------------------------------------------+
|  TmuxCC - AI Agent Dashboard                   Agents: 3 Active: 1|
+------------------------------------------------------------------+
| main (Session)                    | Preview: main:0.0             |
| +-- 0: code                       |                               |
| |  +-- ~/project1                 | Claude Code wants to edit:    |
| |  |  * Claude Code  ! [Edit]     | src/main.rs                   |
| |  |     +-- > Explore (Running)  |                               |
| |  +-- ~/project2                 | - fn main() {                 |
| |     o OpenCode   @ Processing   | + fn main() -> Result<()> {   |
| +-- 1: shell                      |                               |
|    +-- ~/tools                    | Do you want to allow this     |
|       o Codex CLI  * Idle         | edit? [y/n]                   |
+------------------------------------------------------------------+
| [Y] Approve [N] Reject [A] All | [1-9] Choice | [Space] Select    |
+------------------------------------------------------------------+
```

*Replace with actual screenshot*

---

## Features

- **Multi-Agent Monitoring**: Track multiple AI agents across all tmux sessions and windows
- **Real-time Status**: See agent states at a glance (Idle, Processing, Awaiting Approval, Error)
- **Approval Management**: Approve or reject pending requests with single keystrokes
- **Batch Operations**: Select multiple agents and approve/reject all at once
- **Hierarchical View**: Tree display organized by Session/Window/Pane
- **Subagent Tracking**: Monitor spawned subagents (Task tool) with their status
- **Context Awareness**: View remaining context percentage when available
- **Pane Preview**: See live content from selected agent's tmux pane
- **Popup Input Dialog**: Quick text input with configurable trigger key (default `/`)
- **Focus Integration**: Jump directly to any agent's pane in tmux (cross-session support)
- **Custom Agent Patterns**: Define regex patterns to detect any process as an agent
- **Wildcard Detection**: Use `pattern = "*"` to monitor ALL tmux panes
- **Customizable**: Configure polling interval, capture lines, and detection patterns

### Supported AI Agents

| Agent | Detection Method | Approval Keys |
|-------|------------------|---------------|
| **Claude Code** | `claude` command, version numbers, window title with icon | `y` / `n` |
| **OpenCode** | `opencode` command | `y` / `n` |
| **Codex CLI** | `codex` command | `y` / `n` |
| **Gemini CLI** | `gemini` command | `y` / `n` |
| **Custom Agents** | User-defined regex patterns in config | Configurable |

**Note:** Use custom agent patterns to monitor any process (shells, editors, build tools, etc.) - see Configuration section below.

---

## Installation

### From crates.io

```bash
cargo install tmuxcc
```

### From Source

```bash
git clone https://github.com/nyanko3141592/tmuxcc.git
cd tmuxcc
cargo build --release
cargo install --path .
```

### Requirements

- **tmux** (must be running with at least one session)
- **Rust** 1.70+ (for building from source)

---

## Usage

### Quick Start

1. Start tmux and run AI agents in different panes
2. Launch TmuxCC from any terminal:

```bash
tmuxcc
```

**Monitor ALL tmux panes:**

```bash
# Create config with wildcard pattern
tmuxcc --init-config
echo '
[[agent_patterns]]
pattern = "*"
agent_type = "All Panes"
' >> ~/.config/tmuxcc/config.toml

# Run tmuxcc - now shows every tmux pane
tmuxcc
```

### Command Line Options

```
tmuxcc [OPTIONS]

Options:
  -p, --poll-interval <MS>      Polling interval in milliseconds [default: 500]
  -l, --capture-lines <LINES>   Lines to capture from each pane [default: 100]
  -f, --config <FILE>           Path to config file
  -d, --debug                   Enable debug logging to tmuxcc.log
      --show-config-path        Show config file path and exit
      --init-config             Create default config file and exit
  -h, --help                    Print help
  -V, --version                 Print version
```

### Examples

```bash
# Run with default settings
tmuxcc

# Set polling interval to 1 second
tmuxcc -p 1000

# Capture more lines for better context
tmuxcc -l 200

# Use custom config file
tmuxcc -f ~/.config/tmuxcc/custom.toml

# Enable debug logging to tmuxcc.log
tmuxcc --debug

# Enable debug mode in TUI (via config)
tmuxcc --set debug_mode=true

# Initialize default config file
tmuxcc --init-config

# Run regression tests (see tests/README.md for details)
tmuxcc test --dir tests/fixtures/claude
```

### Regression Testing

TmuxCC includes a built-in regression testing suite to verify agent state detection logic against captured pane content.

```bash
# Run tests for a specific agent
tmuxcc test --dir tests/fixtures/claude

# Capture a new test fixture from a running tmux pane
./tests/capture.sh claude cc-ai-maestro idle "my_description"
```

See [tests/README.md](tests/README.md) for more details.

### Wrapper Script for Reliable Focus (Recommended)

The `f` key (focus pane) works best when tmuxcc runs **inside tmux**. Use the wrapper script to ensure tmuxcc always runs in a dedicated tmux session:

```bash
# Install wrapper to ~/bin for quick access
ln -sf "$(pwd)/scripts/tmuxcc-wrapper.sh" ~/bin/tcc

# Now use 'tcc' instead of 'tmuxcc'
tcc
```

**What the wrapper does:**
- Creates/reuses a tmux session named `tmuxcc`
- Launches tmuxcc inside that session
- Enables reliable cross-session focus with `f` key
- Works whether you start it inside or outside tmux

**Without wrapper:**
- `f` key only works within the same tmux session
- Shows error when running outside tmux

**With wrapper:**
- `f` key works for ALL sessions (cross-session navigation)
- Always runs in controlled environment

---

## Key Bindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Next agent |
| `k` / `Up` | Previous agent |
| `Tab` | Cycle through agents |

### Selection

| Key | Action |
|-----|--------|
| `Space` | Toggle selection of current agent |
| `Ctrl+a` | Select all agents |
| `Esc` | Clear selection / Close popup |

**Multi-Selection Workflow:**
1. Navigate to an agent with `j`/`k` or arrow keys
2. Press `Space` to toggle selection (checkbox appears: ☑)
3. Repeat for other agents you want to select
4. Press `y` to approve all selected, or `n` to reject all
5. Press `Esc` to clear all selections

### Actions

| Key | Action |
|-----|--------|
| `y` / `Y` | Approve pending request(s) |
| `n` / `N` | Reject pending request(s) |
| `a` / `A` | Approve ALL pending requests |
| `1`-`9` | Send numbered choice to agent |
| `f` / `F` | Focus on selected pane in tmux (supports cross-session, use wrapper script for best results) |
| `Left` / `Right` | Switch focus (Sidebar / Input) |
| `/` | Show popup input dialog (configurable) |
| `Shift+I` | Open multi-line input modal (Submit with Alt+Enter) |

### Session Management

| Key | Action |
|-----|--------|
| `r` | Rename current session (opens dialog) |

### View

| Key | Action |
|-----|--------|
| `s` / `S` | Toggle subagent log |
| `Ctrl+L` | Refresh / clear error |
| `h` / `?` | Show help |
| `q` | Quit |

---

## Summary View

TmuxCC can display a structured summary of Claude Code activity in a two-column layout (toggle with `s` key):

### Left Column - TODOs

Shows task checkboxes created by Claude Code's built-in task management system:

- `☐` Task title - Pending task
- `☑` Task title - Completed task

**How tasks are created:** When Claude Code uses its `TaskCreate`, `TaskList`, or `TaskUpdate` tools during a session, it displays task markers in the terminal output. TmuxCC parses these markers from the pane content and displays them in the summary view.

**Example:** If you ask Claude Code to "Create tasks to track implementation steps", it will use TaskCreate and display:
```
☐ Research library options
☐ Write implementation
☐ Add tests
```

TmuxCC automatically captures and displays these in the TODO column.

### Right Column - Activity

Shows current agent activity and recent tool usage:

- **Current Activity** (`✽`): What the agent is currently doing
- **Recent Tools** (`⏺`): Last few tool executions (Read, Edit, Bash, etc.)

### Toggle Summary View

Press `s` or `S` to toggle between:
- Full summary view (TODOs + Activity)
- Compact view (status only)

**Note:** The summary view currently works best with Claude Code, as it uses specific output markers. Other agents show basic status information.

---

## Configuration

TmuxCC uses a TOML configuration file.

### Initialize Config

```bash
# Option 1: Create default config file
tmuxcc --init-config

# Option 2: Copy example config and customize
cp config.example.toml ~/.config/tmuxcc/config.toml

# Show config file location
tmuxcc --show-config-path
```

### Config File Location

| OS | Path |
|----|------|
| Linux | `~/.config/tmuxcc/config.toml` |
| macOS | `~/Library/Application Support/tmuxcc/config.toml` |
| Windows | `%APPDATA%\tmuxcc\config.toml` |

### Configuration Options

```toml
# Polling interval in milliseconds
poll_interval_ms = 500

# Number of lines to capture from each pane
capture_lines = 200

# Whether to show detached tmux sessions (default: true)
show_detached_sessions = true

# Enable extra logging in the TUI for debugging (default: false)
debug_mode = false

# Whether to truncate long lines in preview (default: true)
# When true, long lines are truncated to terminal width with "…" indicator
# When false, long lines wrap (legacy behavior)
truncate_long_lines = true

# Max line width for truncation (optional, default: terminal width)
# Only used when truncate_long_lines = true
# max_line_width = 120

# Trigger key for popup input dialog (default: "/")
popup_trigger_key = "/"

# Hide bottom input buffer (use modal textarea instead, default: true)
# When true, the bottom input box is hidden - use Shift+I for multi-line input
# When false, the bottom input box is always visible at the bottom of screen
hide_bottom_input = true

# Whether to log all actions to the status bar (default: true)
# When true, a message is shown in the footer for every key binding action triggered
log_actions = true

# Session Filtering
# Auto-ignore the session where tmuxcc runs (default: true)
# This prevents tmuxcc from showing itself in the dashboard
ignore_self = true

# Ignore specific sessions by pattern (optional)
# Supports three pattern types:
# - Fixed: exact match (e.g., "cc-prod")
# - Glob: shell wildcards (* and ?) (e.g., "test-*", "cc-?-prod")
# - Regex: wrapped in slashes (e.g., "/^ssh-\d+$/")
ignore_sessions = [
  "ssh-tunnel",        # fixed: exact match
  "prod-*",            # glob: matches prod-main, prod-backup, etc.
  "/^(vpn|log)-.*$/",  # regex: matches vpn-* or log-*
]

# Custom agent patterns (optional)
# Patterns are matched against: command, title, cmdline, and child processes
# Built-in agents (Claude Code, OpenCode, etc.) are detected first

# Example 1: Match ALL panes (wildcard)
# Useful for seeing every tmux pane in the dashboard
[[agent_patterns]]
pattern = "*"
agent_type = "All Panes"

# Example 2: Match specific commands
[[agent_patterns]]
pattern = "node"
agent_type = "Node.js"

[[agent_patterns]]
pattern = "bash|zsh"
agent_type = "Shell"

# Example 3: Regex patterns
[[agent_patterns]]
pattern = "python.*"
agent_type = "Python"

# Example 4: Match by process name
[[agent_patterns]]
pattern = "vim|nvim"
agent_type = "Editor"

# Key bindings configuration (optional)
# Configure which keys trigger which actions
[key_bindings]
y = "approve"                          # Approve request
n = "reject"                           # Reject request
a = "approve_all"                      # Approve all pending
"0" = { send_number = 0 }             # Send number choice
"1" = { send_number = 1 }
"2" = { send_number = 2 }
"3" = { send_number = 3 }
"4" = { send_number = 4 }
"5" = { send_number = 5 }
"6" = { send_number = 6 }
"7" = { send_number = 7 }
"8" = { send_number = 8 }
"9" = { send_number = 9 }
E = { send_keys = "Escape" }          # Send ESC key
C = { send_keys = "C-c" }             # Send Ctrl-C
D = { send_keys = "C-d" }             # Send Ctrl-D
K = { kill_app = { method = "sigterm" } }  # Kill with SIGTERM
# or: K = { kill_app = { method = "ctrlc_ctrld" } }  # Ctrl-C then Ctrl-D
r = "rename_session"                  # Rename current session
"C-l" = "refresh"                     # Refresh screen (Ctrl+L)

# Command execution (with variable expansion)
z = { execute_command = { command = "zede ${SESSION_DIR}" } }
v = { execute_command = { command = "code ${SESSION_DIR}" } }
t = { execute_command = { command = "wezterm start -- tmux attach -t ${PANE_TARGET}" } }
d = { execute_command = { command = "zede ~/.dippy" } }

# With modifier keys
"M-t" = { execute_command = { command = "wezterm start -- tmux attach -t ${PANE_TARGET}", blocking = true } }

# Using environment variables
x = { execute_command = { command = "echo ${ENV:USER} - ${SESSION_NAME}" } }
```

**Valid tmux key names for send_keys:**
- Special keys: `Escape`, `Enter`, `Tab`, `BSpace` (backspace), `Space`
- Control sequences: `C-c` (Ctrl-C), `C-d` (Ctrl-D), `C-z` (Ctrl-Z), etc.
- Function keys: `F1`, `F2`, ..., `F12`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Other: `Home`, `End`, `PageUp`, `PageDown`, `Insert`, `Delete`

See `man tmux` section on `send-keys` for complete list.

### Available Key Binding Actions

All key bindings use this format:
```toml
[key_bindings]
# Simple actions (string shorthand)
key = "action_name"

# Complex actions (table format)
key = { action_type = { ... } }
```

**Navigation Actions:**
```toml
j = "next_agent"           # Next agent (down)
k = "prev_agent"           # Previous agent (up)
```

**Approval Actions:**
```toml
y = "approve"              # Approve selected request(s)
n = "reject"               # Reject selected request(s)
a = "approve_all"          # Approve ALL pending requests
```

**Number Choice Actions:**
```toml
"0" = { send_number = 0 }  # Send choice number 0-9
"1" = { send_number = 1 }
# ... up to 9
```

**Send Keys Actions:**
```toml
E = { send_keys = "Escape" }       # Send ESC to agent
C = { send_keys = "C-c" }          # Send Ctrl-C
D = { send_keys = "C-d" }          # Send Ctrl-D
```

**Kill App Actions:**
```toml
K = { kill_app = { method = "sigterm" } }        # Kill with SIGTERM
K = { kill_app = { method = "ctrlc_ctrld" } }    # Kill with Ctrl-C then Ctrl-D
```

**Session Management:**
```toml
r = "rename_session"       # Open dialog to rename session
```

**Refresh Action:**
```toml
"C-l" = "refresh"          # Refresh / clear error
```

**Command Execution:**
```toml
# Simple command (non-blocking)
z = { execute_command = { command = "zede ${SESSION_DIR}" } }

# Blocking command (waits for completion)
"M-t" = { execute_command = { command = "attach ${SESSION_NAME}", blocking = true } }

# Terminal application (interactive, takes over screen)
m = { execute_command = { command = "lazyclaude -m", terminal = true } }
```

### Stdio Redirection
- **Terminal Apps** (`terminal = true`): Inherit stdio (fully visible and interactive).
- **Background Apps** (`blocking = false`): Output is silenced (`/dev/null`) to prevent UI corruption.
- **Debug Mode**: If `debug_mode = true` in config, background output and full results of blocking tasks are written to `.tmuxcc.log` for troubleshooting.

### Modifier Key Syntax

Modifier keys are prefixes to the key name:

| Prefix | Modifier | Example | Description |
|--------|----------|---------|-------------|
| `C-` | Ctrl | `C-l` | Ctrl+L |
| `M-` | Alt | `M-t` | Alt+T (Meta) |
| `S-` | Shift | `S-i` | Shift+I (explicit) |

**Note:** Uppercase letters implicitly include Shift:
- `I` = Shift+i (implicit)
- `S-i` = Shift+i (explicit, same result)
- `M-I` = Alt+Shift+i

### Customizing Hardcoded Keys

Some keys are hardcoded in the application and cannot be changed via config:
- **`Shift+I`** (open multi-line input modal) - hardcoded for convenience
- **`Space`** (toggle selection) - hardcoded
- **`Tab`** (cycle agents) - hardcoded
- **`Esc`** (cancel/close) - hardcoded
- **`h` / `?`** (help) - hardcoded
- **`q`** (quit) - hardcoded
- **Arrow keys** - hardcoded fallback

To change the modal input binding, you would need to modify the source code in `src/ui/app.rs`:
```rust
// Line ~869: Change 'I' to your preferred key
KeyCode::Char('I') => Action::ShowModalTextarea { ... }
```

**Command execution:**
- Execute shell commands from keybindings with variable expansion
- `execute_command = { command = "shell command" }` - Execute a shell command (non-blocking by default)
- `blocking = true` - Wait for command to finish and show output (default: false)
- Variables:
  - `${SESSION_NAME}` - Selected agent's tmux session name
  - `${SESSION_DIR}` - Selected agent's working directory path
  - `${WINDOW_INDEX}` - Selected agent's tmux window index
  - `${WINDOW_NAME}` - Selected agent's tmux window name
  - `${PANE_INDEX}` - Selected agent's tmux pane index
  - `${PANE_TARGET}` - Selected agent's tmux target (session:window.pane)
  - `${ENV:VAR}` - Environment variable value

Example:
```toml
z = { execute_command = { command = "zede ${SESSION_DIR}" } }  # Open editor in agent's directory
t = { execute_command = { command = "wezterm start -- tmux attach -t ${PANE_TARGET}" } } # Attach to pane
"M-t" = { execute_command = { command = "wezterm cli attach ${SESSION_NAME}", blocking = true } }
```

**Pattern Matching:**
- Use `*` for wildcard (matches everything)
- Use regex syntax for complex patterns
- Patterns check command, window title, full cmdline, and child processes
- Invalid regex patterns are silently ignored

**Priority:**
- Built-in parsers (Claude Code, OpenCode, etc.) match first
- Custom patterns are checked in order of definition

### CLI Config Overrides

Override any config option via command line using `--set KEY=VALUE`:

```bash
# Hide detached sessions (full name)
tmuxcc --set show_detached_sessions=false

# Hide detached sessions (short alias)
tmuxcc --set showdetached=0

# Multiple overrides
tmuxcc --set poll_interval=1000 --set showdetached=false

# Supported value formats
--set showdetached=true     # true/false
--set showdetached=1        # 1/0
--set showdetached=yes      # yes/no
--set showdetached=on       # on/off
```

**Available config keys:**
- `poll_interval_ms` (or `pollinterval`) - Polling interval in milliseconds
- `capture_lines` (or `capturelines`) - Lines to capture from panes
- `show_detached_sessions` (or `showdetached`) - Show/hide detached sessions
- `debug_mode` (or `debug`) - Enable/disable debug logging in the TUI
- `truncate_long_lines` (or `truncate`) - Enable/disable line truncation in preview
- `max_line_width` (or `linewidth`) - Max line width for truncation (number or 'none')
- `popup_trigger_key` (or `popupkey`) - Key to trigger popup input dialog (default: "/")
- `ignore_sessions` (or `ignoresessions`) - Comma-separated list of sessions to ignore (supports glob/regex)
- `ignore_self` (or `ignoreself`) - Auto-ignore own session (default: true)
- `log_actions` (or `log`) - Enable/disable action logging to status bar
- `keybindings.KEY` (or `kb.KEY`) - Map key to action (see below)

**Session filtering examples:**
```bash
# Ignore specific sessions (comma-separated patterns)
tmuxcc --set ignore_sessions=prod-*,ssh-tunnel,/^vpn-\d+$/

# Show your own session (disable ignore_self)
tmuxcc --set ignore_self=false
```

**Key binding overrides:**
```bash
# Change E key to send Escape
tmuxcc --set kb.E=send_keys:Escape

# Change K key to kill with SIGTERM
tmuxcc --set kb.K=kill_app:sigterm

# Change K key to kill with Ctrl-C+Ctrl-D
tmuxcc --set kb.K=kill_app:ctrlc_ctrld

# Remap y key to reject
tmuxcc --set kb.y=reject

# Send custom key sequence
tmuxcc --set kb.X=send_keys:C-z
```

**Valid action formats:**
- `approve` - Approve current/selected agent(s)
- `reject` - Reject current/selected agent(s)
- `approve_all` - Approve all pending requests
- `send_number:N` - Send number choice (0-9)
- `send_keys:KEYS` - Send tmux key sequence (e.g., `Escape`, `C-c`, `Enter`)
- `kill_app:sigterm` - Kill app with SIGTERM (graceful)
- `kill_app:ctrlc_ctrld` - Kill app with Ctrl-C+Ctrl-D (forced)
- `rename_session` - Open rename session dialog
- `refresh` - Refresh screen / clear error
- `navigate:next_agent` - Navigate to next agent
- `navigate:prev_agent` - Navigate to previous agent
- `command:CMD` - Execute shell command (supports variable expansion, use `command:CMD:blocking` for blocking or `command:CMD:terminal` for interactive apps)

**Key format for modifier keys:**
- `"C-x"` - Ctrl+X (use quotes in TOML for keys with special characters)
- `"M-x"` - Alt+X (Meta key)

**Key normalization:** Underscores and hyphens are ignored, case-insensitive (except for key names themselves which are case-sensitive)

---

## Status Indicators

| Icon | Status |
|------|--------|
| `!` `[Edit]` | File edit approval pending |
| `!` `[Shell]` | Shell command approval pending |
| `!` `[Question]` | User question awaiting response |
| `@` | Processing |
| `*` | Idle |
| `?` | Unknown |

---

## How It Works

1. **Discovery**: TmuxCC scans all tmux sessions, windows, and panes
2. **Detection**: Identifies AI agents by process name, window title, and command line
3. **Parsing**: Agent-specific parsers analyze pane content for status and approvals
4. **Monitoring**: Continuously polls panes at configurable intervals
5. **Actions**: Sends keystrokes to panes for approvals/rejections

---

## Project Structure

```
tmuxcc/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── agents/           # Agent type definitions
│   │   ├── types.rs      # AgentType, AgentStatus, MonitoredAgent
│   │   └── subagent.rs   # Subagent, SubagentType, SubagentStatus
│   ├── app/              # Application logic
│   │   ├── state.rs      # AppState, AgentTree, InputMode
│   │   ├── actions.rs    # Action enum
│   │   └── config.rs     # Configuration
│   ├── monitor/          # Monitoring
│   │   └── task.rs       # Async monitoring task
│   ├── parsers/          # Agent output parsers
│   │   ├── mod.rs        # AgentParser trait, ParserRegistry
│   │   ├── claude_code.rs
│   │   ├── opencode.rs
│   │   ├── codex_cli.rs
│   │   ├── gemini_cli.rs
│   │   └── custom.rs     # CustomAgentParser (user-defined patterns)
│   ├── tmux/             # tmux integration
│   │   ├── client.rs     # TmuxClient
│   │   └── pane.rs       # PaneInfo, process detection
│   └── ui/               # UI implementation
│       ├── app.rs        # Main loop
│       ├── layout.rs     # Layout definitions
│       └── components/   # UI components
└── Cargo.toml
```

---

## Tech Stack

- **Language**: Rust (Edition 2021)
- **TUI Framework**: [Ratatui](https://ratatui.rs/) 0.29
- **Terminal**: [Crossterm](https://github.com/crossterm-rs/crossterm) 0.28
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **CLI Parser**: [Clap](https://clap.rs/) 4

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Contributing

Contributions are welcome! Here's how you can help:

### Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run clippy (`cargo clippy`)
6. Format code (`cargo fmt`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

### Areas for Contribution

- **New Agent Support**: Add parsers for other AI coding assistants
- **UI Improvements**: Enhance the terminal interface
- **Performance**: Optimize polling and parsing
- **Documentation**: Improve docs and examples
- **Bug Fixes**: Report and fix issues
- **Tests**: Improve test coverage

### Code Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality

---

## Related Projects

- [Claude Code](https://claude.ai/code) - Anthropic's AI coding assistant
- [OpenCode](https://github.com/opencode-ai/opencode) - Open-source AI coding assistant
- [Codex CLI](https://github.com/openai/codex-cli) - OpenAI's Codex CLI
- [Gemini CLI](https://github.com/google/gemini-cli) - Google's Gemini CLI
