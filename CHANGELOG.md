# Changelog

All notable changes to **Tmuxx** (formerly tmuxcc) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Performance
- **Optimized Redraw Loop**: Significantly reduced CPU usage by only redrawing the TUI when state changes or input occurs, instead of forcing 60 FPS redraws. This allows the application to be idle-efficient while remaining responsive.

### Added
- **External Terminal Wrapper**: Added `terminal_wrapper` config option to execute commands in a separate terminal window (e.g. WezTerm, Alacritty) instead of the background or current pane.
- **External Command Execution**: Added `external_terminal` flag to key bindings and menu items to leverage the wrapper.
- **Config Hot Reload**: Added `ReloadConfig` action (default binding: `Ctrl+r`) to reload the application configuration from disk without restarting.
- **Full-Width TODO**: Added `todo_full_width` configuration option (default: `true`). When enabled, the TODO section expands to full width, hiding the right-side activity panel to provide more space for tasks.
- **Menu/Prompt Preview**: Added a preview box at the bottom of the Command and Prompts menus to show the full command or prompt text of the selected item.
- **Menu Hints**: Added usage hints to the bottom of "Prompts" and "Command" menus (e.g., explaining `Enter` vs `Alt+Enter` for prompts).

### Fixed
- **Safe Config Reload**: Modified the configuration reload process to handle errors gracefully. If an invalid configuration is detected during reload, the application now displays an error in the status bar instead of crashing.
- **Status Bar Colors**: Improved the status bar (footer) color logic. Status messages and action logs are now displayed in Green, while actual errors remain Red.
- **Help/Modal Scrolling**: Fixed arrow keys in readonly modals (like Help) to scroll text instead of moving the cursor.
- **Help Window Closing**: Prevented `Left`/`Right`/`Home`/`End` keys from accidentally closing the Help window.


## [Unreleased]

## [0.2.2] - 2026-01-27
### Added
- **Pi Powerline Support**: Added native support for the new Pi extension/theme (Powerline status bar, rounded corners).
- **Content-Based Agent Matching**: Added `content` matcher type to agent definitions, allowing agents to be distinguished not just by command, but by specific content patterns in their output (e.g. rounded corners).
- **Improved Pi Detection**: Standard Pi definition remains robust, while "Powerline" variant is auto-detected via content matching.

## [0.2.1] - 2026-01-27
### Added
- **Kill Session Support**: Added ability to kill the entire tmux session of the selected agent (default binding: `X` or `Shift+x`).
- **Confirmation Dialogs**: Added safety confirmation popup for destructive actions like killing a session.
- **Improved Kill Action**: Added `respawn` method to `KillApp` action, which uses `tmux respawn-pane -k` for reliable process termination. Default binding for `K` updated to use this method.

## [0.2.0] - 2026-01-26 - The "Tmuxx" Rewrite

### 🚨 Breaking Changes
- **Project Renamed**: Entire project renamed from `tmuxcc` to `tmuxx`.
  - Binary name: `tmuxx`
  - Config directory: `~/.config/tmuxx/`
  - Config file: `config.toml` (inside config dir) or `.tmuxx.toml`
  - Log file: `tmuxx.log`
- **Repo Change**: New repository location: [orgoj/tmuxx](https://github.com/orgoj/tmuxx).

### Documentation
- **README Rewrite**: Complete overhaul of README.md to reflect current features (tmuxx rename, config-driven architecture, menu system).
- **Wrapper Documentation**: Added dedicated section for `txx` wrapper script.

### Added
- **Native CLI Agents Support**: Explicit support for running as a universal dashboard with specialized support for AI agents.
- **Todo from File**: Added default support for reading `TODO.md` from the agent's working directory.

### Changed
- **Documentation**: Complete overhaul of README and internal docs to reflect the new identity.
- **Config Defaults**: Updated defaults to be more robust for general usage.

### Fixed
- **Wrapper Script**: Updated `scripts/tmuxx-wrapper.sh` to pass command line arguments to the binary.
- **Clippy Warning**: Fixed collapsible-else-if warning in `src/ui/app.rs`.

---

## [0.1.25] - 2026-01-26 (Last tmuxcc version)

### Performance
- **Optimized Rendering**: Implemented pre-parsing of templates and color caching to eliminate UI lag during navigation, especially with many agents.
- **Efficient Tree Rendering**: Refactored agent list rendering to use a single `ListItem` per agent, resolving scrolling artifacts and cropping issues.

### Added
- **Session Header Styling**: Configurable colors for session headers to visually separate groups.
- **Enhanced Navigation**:
  - **Home/End**: Jump to first/last agent in the list.
  - **Cyclic Navigation**: Configurable option `cyclic_navigation` (default: true).

### Fixed
- **Mouse Interaction**: Fixed mouse click detection for multi-line agents.

## [0.1.24] - 2026-01-26

### Added
- **"pi" Agent**: Support for "pi" coding agent with state detection and approval monitoring.
- **Config-Driven Status Architecture**: Transitioned to a 100% configuration-driven status system.

### Fixed
- **Claude Detection**: Added support for "Would you like to proceed?" prompt.
- **Improved Test System**: Standardized status naming convention.

## [0.1.23] - 2026-01-26

### Added
- **Prompts Menu**: New "Prompts" tree menu (toggle with `p`).

### Fixed
- **UI Details Rendering**: Fixed bug where the detailed view pane would show stale content.

## [0.1.22] - 2026-01-25

### Added
- **Quick Filters**: Added dynamic filtering capabilities (Show Selected `s`, Show Active `x`).
- **Config-Driven Help**: Help screen (`?`) is now dynamically generated.
- **Configurable Pane Tree**: Complete overhaul of the agent list display with configurable templates.
- **Persistent Tree Menu**: Refactored the Command Menu into a true persistent tree view.

### Fixed
- **Mouse Selection**: Fixed inaccurate mouse click selection.

## [0.1.20] - 2026-01-25

### Added
- **Command Menu Support**: Introduced a hierarchical, fuzzy-searchable command menu accessible via the `m` key.
- **Ratatui Upgrade**: Standardized on `ratatui` 0.29.0.

## [0.1.17] - 2026-01-25

### Added
- **Configurable Sidebar Width**: Configurable via `sidebar_width`.
- **Automatic Working Directory context**.

## [0.1.16] - 2026-01-25

### Changed
- **Immediate Screen Capture**: `Capture Test Case` (`C-s`) captures immediately.
- **Capture Directory Structure**: Test case captures saved in directories named after Agent Name.
- **Key Binding Architecture**: All default bindings defined in `src/config/defaults.toml`.

### Fixed
- **Shell Detection Robustness**: Increased `last_lines` scan limit.

## [0.1.15] - 2026-01-25

### Fixed
- **Claude Code Approval Prompt Detection**: Fixed detection of numbered choice prompts.
- **tmuxcc-wrapper.sh Session Creation**: Fixed wrapper script.

### Added
- **Test Anonymization**: Added basic path and username anonymization.
- **Enhanced Claude Detection**: Added or updated patterns for several processing and idle states.

## [0.1.14] - 2026-01-25

### Added
- **Native Regression Testing**: Added `tmuxx test` command.
- **Dynamic Approval Types**: Added `approval_type` support.

### Fixed
- **Structural Regex Robustness**: Improved structural prompt block regex for Claude Code.

## [0.1.12] - 2026-01-24

### Added
- **Action Logging**: All key binding actions are logged to status bar.

## [0.1.11] - 2026-01-24

### Changed
- **UI Simplification**: Removed footer buttons and characters.

## [0.1.10] - 2026-01-24

### Added
- **New Command Variables**: `${PANE_TARGET}`, `${WINDOW_INDEX}`, etc.

## [0.1.9] - 2026-01-24

### Added
- **Interactive TUI Support**: `terminal = true` flag.

## [0.1.8] - 2026-01-24

### Fixed
- **Modal Editor Usability Fixes**.

### Added
- **Modal Multi-line Input Dialog** (Shift+I).
- **Command Execution in Key Bindings**.
- **Session Rename** (default: `r`).
- **Configurable Refresh Key** (default: `Ctrl+L`).
- **Modifier Key Support**.
- **Session Ignore Filter**.
- **Popup Input Dialog** (default: `/`).

## [0.1.7] - 2026-01-23

### Added
- **Configurable Key Bindings**.
- **Dynamic Help System**.
- **Dynamic Footer Buttons**.
- **Preview Truncation**.
- **Config Override System**.
- **Custom Agent Patterns**.
- **Cross-Session Focus**.
- **Wrapper Script**.

---

## Origin Story

**Tmuxx** started as a fork of `tmuxcc` (by nyanko3141592) but evolved into a complete rewrite and distinct project. We thank the original authors for their inspiration.
