# Changelog

All notable changes to this fork (orgoj/tmuxcc) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- **Interactive TUI Support** (Terminal Mode)
  - `terminal = true` flag for `execute_command` to run interactive apps like `lazyclaude`, `nano`, `vim`
  - Correctly suspends `tmuxcc`, gives child process terminal control, and restores TUI on exit
- **Enhanced Status Bar** - Messages now expand to full terminal width to prevent truncation
- **Improved Stdio Handling**
  - Background processes (`blocking = false`) now have their output silenced (`/dev/null`) by default to prevent screen corruption
  - In `debug_mode = true`, background outputs are redirected to `.tmuxcc.log`
  - Blocking tasks in debug mode also log full output to `.tmuxcc.log`

### Changed
- **Shift Key Mapping** - Shift+Key now maps to uppercase char (e.g. `M`) instead of `S-m` prefix, matching common config expectations

### Fixed
- **Screen Redraw** - `Ctrl+l` now performs a full screen clear and redraw
- **Key Binding Feedback** - `send_keys` actions now display the exact keys sent in the status bar for debugging

## [0.1.8] - 2026-01-24

### Fixed
- **Modal Editor Usability Fixes** (Shift+I)
  - **Cursor Visibility** - High-contrast cursor (black background block) for better visibility on spaces
  - **Scrollbar** - Added visual scrollbar to right edge showing position in document
  - **Multi-line Submit** - Added keybindings for odeslání: `Alt+Enter`
  - **Paste Support** - Added global multi-line paste handling (`Event::Paste`)
  - **Help Closure** - Fixed bug where Help modal required two Esc presses to close

### Added
- **Modal Multi-line Input Dialog** - Rich text editor for sending longer messages to agents
  - Press `Shift+I` to open modal textarea dialog
  - Multi-line editing with proper cursor movement
  - Undo/Redo support (Ctrl+U / Ctrl+R)
  - Emacs-style shortcuts supported
  - Enter inserts newlines (submit with Alt+Enter)
  - `hide_bottom_input` config option (default: true) - hides bottom input box when modal is preferred
  - Uses `tui-textarea` library for advanced text editing

### Changed
- **Full English Translation** - Translated all Japanese and Czech text to English
  - CLI help text (--help) now fully in English
  - Error messages translated to English
  - Code comments translated to English
  - TODO.md tasks translated to English
  - .dippy config file comments translated

### Added
- **Command Execution in Key Bindings** - Execute shell commands from keybindings with variable expansion
  - `execute_command = { command = "shell command" }` - Execute shell command (non-blocking by default)
  - `blocking = true` - Wait for command to finish and show output
  - Variables: `${SESSION_NAME}` (tmux session), `${SESSION_DIR}` (working directory), `${ENV:VAR}` (environment variable)
  - Example: `z = { execute_command = { command = "zede ${SESSION_DIR}" } }` opens editor in agent's directory
  - Example: `"M-t" = { execute_command = { command = "wezterm cli attach ${SESSION_NAME}", blocking = true } }` attaches tmux session
  - CLI override: `--set kb.z=command:zede\${SESSION_DIR}`
  - **Config validation** - Invalid TOML format now causes parsing errors with helpful messages
  - **--debug-config flag** - Shows loaded config and bindings before starting (useful for debugging)
- **Session Rename** - Rename tmux sessions directly from tmuxcc (default: `r` key)
  - Opens popup dialog with current session name pre-filled
  - Cursor positioned at end of text for easy editing
  - Validates session name (no empty, no `.` or `:` characters)
  - Configurable via `[key_bindings]`: `r = "rename_session"`
- **Configurable Refresh Key** - Refresh/redraw moved from hardcoded `r` to configurable binding
  - Default: `Ctrl+L` (standard terminal redraw key)
  - Supports modifier key format: `"C-l"` (Ctrl), `"M-l"` (Alt)
  - Configurable via `[key_bindings]`: `"C-l" = "refresh"`
- **Modifier Key Support in Key Bindings** - Key bindings now support Ctrl and Alt modifiers
  - Format: `"C-x"` for Ctrl+X, `"M-x"` for Alt+X
  - TOML requires quotes for keys with special characters

- **Session Ignore Filter** - Config option to ignore (hide) specific tmux sessions at data-collection level
  - `ignore_sessions` - List of patterns to ignore (supports fixed, glob, regex)
  - `ignore_self = true` - Auto-ignore the session where tmuxcc runs (default: true)
  - Pattern auto-detection:
    - `/pattern/` → regex (e.g., `/^ssh-\d+$/`)
    - Contains `*` or `?` → glob (e.g., `test-*`, `cc-?-prod`)
    - Plain string → fixed (exact match)
  - CLI override: `--set ignore_sessions=prod-*,ssh-tunnel` and `--set ignore_self=false`
  - Applied at data-collection level (before agent processing) for efficiency
  - Added SessionPattern module with comprehensive test coverage

### Fixed
- **Preview Line Truncation** - Fixed long lines in preview area not being truncated
  - All lines are now always truncated to fit display width (no wrapping)
  - Removed "important markers" exception that prevented truncation of lines with [y/n], ⚠, etc.
  - Removed `.wrap()` from all Paragraph widgets to prevent line wrapping
  - Each source line = 1 visual line, ensuring "last N lines" shows exactly N visual lines
  - User always sees the actual end of content without lines being pushed off screen

- **Filter Navigation Anti-Pattern** - Fixed critical bug where up/down navigation moved through hidden agents
  - Navigation now only moves through visible (filtered) agents
  - `select_all()` only selects visible agents
  - `toggle_selection()` is no-op for hidden agents
  - `get_operation_indices()` only returns visible indices
  - When filter is applied, cursor jumps to first visible agent if current is hidden
  - Multi-selection is cleaned up when filter changes (removes hidden agents)
  - Added `visible_agent_indices()` and `ensure_visible_selection()` helpers
  - Added 7 unit tests for filter-aware navigation behavior

### Added
- **Popup Input Dialog** - Added popup input dialog feature with configurable trigger key (default `/`)
  - Reusable component for quick text input without switching focus
  - Single-line input with cursor navigation (Home/End/Left/Right)
  - Keyboard shortcuts: Ctrl+U (clear), Ctrl+A (select all), Enter (submit), Esc (cancel)
  - Horizontal scrolling for long text input
  - UTF-8 safe with unicode-width support
  - Configurable trigger key via `popup_trigger_key` in config file or `--set popup_trigger_key=X` CLI override
  - Help display shows current trigger key

### Fixed
- **Language Marker Rules** - Made language marker rules flexible instead of rigid `.cs` extension
  - Language markers can now be: `_cs`, `.cs`, `-cs`, `cs.`, `cs_` anywhere in filename
  - Examples: `README.cs.md`, `README_cs.md`, `notes_cs.txt`, `config.cs.toml`
  - Previous rigid rule only allowed `.cs` extension which didn't work for all file types

### Changed
- **CLAUDE.md Refactoring** - Reduced CLAUDE.md from 468 to 272 lines (42% reduction)
  - Extracted workflow procedures into dedicated skills in `.claude/skills/`
  - Created 4 project-specific skills:
    - `tmuxcc-testing` - Testing workflow and tmux safety rules
    - `tmuxcc-commit` - Pre-commit checklist and git workflow
    - `tmuxcc-library-research` - Library research workflow
    - `tmuxcc-changelog` - TODO/CHANGELOG management workflow
  - CLAUDE.md now contains only skill references and core architecture docs
  - Improves maintainability and reduces cognitive load for AI sessions

### Fixed
- **Preview Empty Display** - Fixed issue where pane preview showed nothing when content had trailing empty lines
  - Preview now trims trailing empty lines before displaying content
  - Affected both summary and detailed preview views
  - Ensures actual content is visible even when tmux pane has many blank lines at bottom

### Added
- **Fully Configurable Key Bindings** - All approval keys (y/n/a) and custom action keys now configurable via `[key_bindings]` in config.toml
  - Default bindings: y=approve, n=reject, a=approve_all, 0-9=send_number, E=ESC, C=Ctrl-C, D=Ctrl-D, K=kill
  - Support for custom `send_keys` actions to send arbitrary key sequences to tmux panes (e.g., Escape, C-c, C-d, Enter, etc.)
  - Support for `kill_app` action with two methods:
    - `sigterm` - Send SIGTERM to process (graceful shutdown, default for K key)
    - `ctrlc_ctrld` - Send Ctrl-C then Ctrl-D sequence (forced interrupt)
  - CLI override support: `--set kb.KEY=ACTION` (e.g., `--set kb.E=send_keys:Escape`, `--set kb.K=kill_app:ctrlc_ctrld`)
  - New config module: `src/app/key_binding.rs` with `KeyAction`, `KeyBindings`, `KillMethod` types
  - HashMap-based flexible key binding storage for easy customization
  - Number keys (0-9) explicitly configurable per user request
- **Dynamic Help System** - Help text (?) now displays actual configured keys, not hardcoded defaults
  - Help dynamically generated from `config.key_bindings`
  - Shows all custom send_keys and kill_app bindings
  - Updates automatically when config changes
- **Dynamic Footer Buttons** - Footer shows configured key labels, updates when config changes
  - Button labels read from config at runtime
  - Applies to y/n/a buttons (approve/reject/approve_all)
- **Preview Truncation** - Smart line truncation to show approval prompts at bottom
  - Long lines now truncated instead of wrapped, ensuring bottom content visible
  - New config option: `truncate_long_lines` (default: true) to enable/disable truncation
  - New config option: `max_line_width` (default: terminal width) for custom truncation width
  - Important markers ([y/n], approve, reject, Allow, Deny) preserved from truncation
  - Unicode-safe truncation with proper character width calculation
  - Truncation indicator "…" appended to truncated lines
  - CLI override support: `--set truncate:false` or `--set linewidth:100`
  - Increased default `capture_lines` from 100 to 200 for better coverage
- **Config Override System** - General `--set KEY=VALUE` CLI mechanism for config overrides
  - New config option: `show_detached_sessions` (default: true) to control session visibility
  - CLI override support: `--set show_detached_sessions=false` or `--set showdetached=0`
  - Multiple format support for booleans: true/false, 1/0, yes/no, on/off
  - Short aliases: `pollinterval`, `capturelines`, `showdetached`
  - Key normalization (underscores/hyphens ignored, case-insensitive)
  - Multiple `--set` flags can be used together
  - Helpful error messages for invalid keys or values
- **Custom Agent Patterns** - Configure regex patterns to detect any process as an agent
  - Support for wildcard `*` pattern to monitor ALL tmux panes
  - Flexible agent_type naming (e.g., "Node Agent", "Bash Agent", "All Panes")
  - Priority system: built-in parsers checked first, then custom patterns
  - `AgentType::Custom(String)` variant for dynamic agent types
  - Cyan color for custom agents in UI
- **Cross-Session Focus** - `f` key now works across different tmux sessions
  - Automatic session detection (current vs target)
  - Uses `tmux switch-client` for cross-session navigation
  - Clear error message when running outside tmux
- **Wrapper Script** - Ensure tmuxcc always runs inside tmux
  - `scripts/tmuxcc-wrapper.sh` creates/reuses `tmuxcc` session
  - Enables reliable cross-session focus without complex terminal launching
  - Works whether started inside or outside tmux
- **Documentation Updates**
  - Comprehensive README section on custom agent patterns
  - Wrapper script installation and usage guide
  - Updated keyboard shortcuts documentation

### Changed
- **Action enum** - Added `SendKeys(String)` and `KillApp { method: KillMethod }` variants
- **Key handling** - `map_key_to_action()` now accepts `&Config` parameter and checks configured bindings before hardcoded fallbacks
- **TmuxClient** - Added `kill_application()` method supporting SIGTERM and Ctrl-C+Ctrl-D kill methods
- **HelpWidget** - `render()` signature changed to accept `&Config`, builds help text dynamically
- **FooterWidget** - `get_button_layout()` returns `Vec<(String, ...)>` instead of `Vec<(&'static str, ...)>`, accepts `&Config` parameter
- **FooterWidget** - `render()` and `hit_test()` signatures changed to accept `&Config` parameter
- Arrow keys (↓/↑) remain as fallback navigation even if j/k remapped (safety feature)
- `TmuxClient` constructor changed to accept full `Config` reference via `from_config()`
- Session filtering now applied in `TmuxClient::list_panes()` based on `show_detached_sessions`
- CLI argument processing now supports multiple `--set` overrides applied after config load
- `ParserRegistry` now accepts `Config` parameter for custom pattern support
- Agent detection call chain updated (app.rs → monitor/task.rs → ParserRegistry)
- Focus pane behavior enhanced for cross-session support

### Fixed
- Agent detection now works properly (was ignoring agent_patterns config)
- Focus pane (`f` key) now functional for cross-session navigation
- Custom agent types display correctly in UI with proper colors

### Technical Details
- New file: `src/app/key_binding.rs` - Key binding data model (`KeyAction`, `KeyBindings`, `KillMethod`, `NavAction`)
- Modified: `src/app/config.rs` - Added `key_bindings: KeyBindings` field to Config struct
- Modified: `src/app/config_override.rs` - Added `KeyBinding(String, KeyAction)` variant, `parse_key_action()` helper
- Modified: `src/app/actions.rs` - Added `SendKeys(String)` and `KillApp { method: KillMethod }` action variants
- Modified: `src/app/mod.rs` - Re-export key_binding types
- Modified: `src/ui/app.rs` - Updated `map_key_to_action()` to use config, added handlers for SendKeys/KillApp actions
- Modified: `src/tmux/client.rs` - Added `kill_application()` method with SIGTERM and Ctrl-C+Ctrl-D support
- Modified: `src/ui/components/help.rs` - Refactored to accept Config, build help dynamically
- Modified: `src/ui/components/footer.rs` - Changed return type to String, accept Config, dynamic button labels
- Added dependency: `libc = "0.2"` for SIGTERM support
- New file: `src/app/config_override.rs` - Config override parsing with alias support
- Modified: `src/app/config.rs` - Added `show_detached_sessions` field and `apply_override()` method
- Modified: `src/tmux/client.rs` - Added `show_detached_sessions` field, `from_config()` constructor, session filtering
- Modified: `src/main.rs` - Added `--set` CLI argument and override application logic
- Modified: `src/ui/app.rs` - Changed to use `TmuxClient::from_config()`
- New file: `src/parsers/custom.rs` - CustomAgentParser implementation
- Modified: `src/parsers/mod.rs` - Added `with_config()` method
- Modified: `src/agents/types.rs` - Added `AgentType::Custom(String)` variant
- Modified: `src/ui/components/agent_tree.rs` - Custom color handling
- New file: `scripts/tmuxcc-wrapper.sh` - Wrapper script for reliable tmux integration

---

## [0.1.7] - 2026-01-23 (orgoj fork baseline)

### Added (from upstream before fork)
- Toggle for TODO/Tools display (`t` key)
- System stats in header (CPU, memory usage)
- 2-column TODO layout
- Working directory and Git branch display in Summary panel
- Mouse support for pane selection and scrolling
- Compact single-line footer without border

### Changed
- Version bumped to 0.1.7
- Allow dirty publish for Cargo.lock changes

### Fixed
- Formatting issues resolved
- Test expectations corrected
- Release workflow updated

---

## Fork Information

**Original Project**: [nyanko3141592/tmuxcc](https://github.com/nyanko3141592/tmuxcc)
**Fork**: [orgoj/tmuxcc](https://github.com/orgoj/tmuxcc)
**Fork Date**: 2026-01-23
**Base Version**: 0.1.7

### Fork Goals
- Add custom agent detection patterns for flexible monitoring
- Improve cross-session focus capabilities
- Enhance usability with wrapper scripts
- Maintain compatibility with upstream

### Not Published to crates.io
This fork is NOT published to crates.io. Install from source:

```bash
git clone https://github.com/orgoj/tmuxcc.git
cd tmuxcc
cargo install --path .
```

---

## Notes

### Configuration Breaking Changes
None yet. All changes are additive and backward compatible.

### Upgrade Guide
When upgrading from upstream or earlier fork versions:

1. Update config to use new agent_patterns feature (optional):
   ```toml
   [[agent_patterns]]
   pattern = "*"
   agent_type = "All Panes"
   ```

2. Install wrapper script for best experience (optional):
   ```bash
   ln -sf $(pwd)/scripts/tmuxcc-wrapper.sh ~/bin/tcc
   ```

3. Rebuild and reinstall:
   ```bash
   cargo build --release
   cargo install --path .
   ```

### Future Plans
See [TODO.md](TODO.md) and [IDEAS.md](IDEAS.md) for planned features and improvements.
