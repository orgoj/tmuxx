# tmuxx

**Universal Config-Driven Dashboard for tmux & AI Agents**

`tmuxx` is a powerful TUI (Terminal User Interface) dashboard for monitoring and controlling multiple tmux panes simultaneously.

While designed with first-class support for AI coding agents (Claude Code, Pi, Gemini, OpenCode), its completely configuration-driven architecture makes it a universal tool for monitoring **any** terminal process using custom regex rules.

> **Note**: This is a complete rewrite of `tmuxcc`, evolved into a standalone project with a focus on configurability, performance, and native AI agent integration.

---

## 🚀 Features

-   **Universal Monitoring**: Monitor any tmux pane using configurable regex patterns.
-   **Native AI Agent Support**: Out-of-the-box support for:
    -   **Claude Code** (Anthropic)
    -   **Pi** (Inflection)
    -   **Gemini CLI** (Google)
    -   **OpenCode**
-   **Smart Detection**: Automatically identifies agents, their state (Idle, Working, Error), and pending approvals.
-   **Config-Driven Architecture**:
    -   **Status Rules**: Define how to parse pane content into states using regex.
    -   **Menus**: Define custom hierarchical Command Menus and Prompt trees.
    -   **Key Bindings**: Fully customizable keyboard shortcuts.
-   **Interactive Control**:
    -   **Global Input**: Send input to one or multiple agents.
    -   **Approvals**: `y`/`n` to approve/reject agent requests (file edits, execution).
    -   **Menus**: Fuzzy-searchable Command Menu (`m`) and Prompts Menu (`p`).
    -   **Editor**: Built-in multi-line input editor (`Shift+I`).
-   **Focus Management**:
    -   **Cross-Session Jump**: Instantly switch tmux focus to the selected agent's pane (even across sessions).
-   **Filtering & Navigation**:
    -   **Quick Filters**: Show only "Active" (`x`) or "Selected" (`s`) agents.
    -   **Tree View**: Organized by Session -> Window -> Pane (`c` to toggle compact mode).
-   **Project Context**: Automatically displays `TODO.md` or `README.md` from the agent's working directory.
-   **Regression Testing**: Built-in suite to verify regex parsing against captured pane snapshots.

---

## 📦 Installation

### From Source

```bash
git clone https://github.com/orgoj/tmuxx.git
cd tmuxx
cargo build --release
cargo install --path .
```

### Requirements

-   **tmux** (must be running)
-   **Rust** 1.70+ (to build)

---

## ⚡ Quick Start

1.  **Start tmux** and run your agents or processes in various panes.
2.  **Run `tmuxx`**:
    ```bash
    tmuxx
    ```
3.  **Navigate**:
    -   Use `Up`/`Down`/`Home`/`End` to move.
    -   Press `f` to focus the selected pane (if running inside tmux).
    -   Press `m` to open the Command Menu.

### First Run Configuration

Generate a default configuration file to customize behaviors:

```bash
tmuxx --init-config
```

Configuration is stored in:
-   Linux: `~/.config/tmuxx/config.toml`
-   macOS: `~/Library/Application Support/tmuxx/config.toml`

---

## 🎮 Key Bindings

All bindings are configurable in `config.toml`. Defaults:

| Key | Action | Description |
|-----|--------|-------------|
| **Navigation** | | |
| `Up` / `Down` | Prev/Next | Navigate agent list |
| `Home` / `End` | First/Last | Jump to start/end of list |
| `f` | Focus | Switch tmux focus to selected pane (works if tmuxx is inside tmux) |
| `Space` | Select | Toggle selection (multiselect) |
| **Actions** | | |
| `y` / `n` | Approve/Reject | Confirm agent action (e.g. file edit) |
| `a` | Approve All | Approve all pending requests |
| `/` | Input | Open popup to send text to agent |
| `Shift+I` | Editor | Open multiline editor for prompt |
| `C-l` | Refresh | Force refresh / clear error states |
| `C-s` | Capture | Capture current pane state for testing |
| `r` | Rename | Rename current session |
| `K` | Kill | Kill/Respawn the process in the selected pane |
| `X` | Kill Session | Kill the entire tmux session of selected agent |
| **Views & Menus** | | |
| `m` | Command Menu | Open fuzzy-searchable command menu |
| `p` | Prompts Menu | Open tree of saved prompts |
| `c` | Compact Mode | Toggle between Full and Compact tree view |
| `?` | Help | Show dynamic help screen |
| **Filters** | | |
| `s` | Filter Selected | Show only selected agents |
| `x` | Filter Active | Show only active (non-idle) agents |
| `S` | Subagents | Toggle subagent log view |

---

## ⚙️ Configuration

`tmuxx` is built to be customized. You can define your own agents, status rules, and menus in `config.toml`.

### Defining Custom Agents

You can add any process to the dashboard by adding to `config.toml`:

```toml
[[agents]]
id = "my-worker"
name = "Worker"
background_color = "#e0e0e0"

  # Match by command name (regex)
  [[agents.matchers]]
  type = "command"
  pattern = "python|node"

  # Define states based on regex in the pane output
  [[agents.state_rules]]
  status = "Error"
  type = "error"
  pattern = "(?i)exception|error|panic"

  [[agents.state_rules]]
  status = "Working"
  type = "working"
  pattern = "Processing..."
```

### Command Menu

Define your own hierarchical menu (`m` key):

```toml
[menu]
  [[menu.items]]
  name = "Git"
    [[menu.items.items]]
    name = "Status"
    # Execute a command in the agent's directory
    execute_command = { command = "git status", blocking = true }
```

### Prompts Menu

Define commonly used prompts (`p` key):

```toml
[prompts]
merge_with_defaults = true  # Append to default prompts instead of replacing them

  [[prompts.items]]
  name = "Refactor"
  text = "Refactor this code to be more modular."
  
  # Nested menu example
  [[prompts.items]]
  name = "Testing"
    [[prompts.items.items]]
    name = "Generate Unit Tests"
    text = "Create unit tests for this module."
```

### Power User Tips

You can define custom keybindings to execute external commands using variables like `${SESSION_DIR}`, `${PANE_TARGET}`, etc.

**Example 1: Open a new terminal window attached to the selected agent**
Instead of relying on `f` (switch-client), you can spawn a new terminal window (e.g., WezTerm, Alacritty, Ghostty) attached directly to the agent's pane.

```toml
[key_bindings]
# Press 't' to open WezTerm attached to the agent's specific pane
"t" = { execute_command = { command = "wezterm start -- tmux attach -t ${PANE_TARGET}" } }

# Press 'Alt+t' to open Alacritty attached to the session
"M-t" = { execute_command = { command = "alacritty -e tmux attach -t ${SESSION_NAME}" } }
```

**Example 2: Open VS Code or Editor in Agent's Directory**
Quickly jump to the code the agent is working on.

```toml
[key_bindings]
# Press 'v' to open VS Code in the agent's working directory
"v" = { execute_command = { command = "code ${SESSION_DIR}" } }

# Press 'z' to open Zed editor
"z" = { execute_command = { command = "zed ${SESSION_DIR}" } }
```

---

## 🧪 Regression Testing

`tmuxx` includes a native regression testing suite to ensure regex rules works correctly. It tests the parsing logic against real snapshots of tmux panes.

### Running Tests

To run the test suite against all fixtures in `tests/fixtures/`:

```bash
tmuxx test
```

Or target a specific directory:

```bash
tmuxx test --dir tests/fixtures/claude
```

### Capturing Test Cases

To add a new test case:

1.  Run `tmuxx`.
2.  Navigate to the agent pane you want to capture.
3.  Press **`C-s`** (Ctrl+S).
4.  Enter the expected status (e.g., `idle`, `working`, `approval`).

`tmuxx` will automatically:
-   Capture the current pane content.
-   Generate a filename like `case_idle_20260126_200000.txt`.
-   Save it to `tests/fixtures/<agent_name>/`.

This ensures your regex rules remain accurate as tools evolve.

---

## License

MIT License
