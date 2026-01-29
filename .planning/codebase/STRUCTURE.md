# Codebase Structure

**Analysis Date:** 2026-01-30

## Directory Layout

```
tmuxx/
├── src/                    # Source code
│   ├── main.rs            # CLI entry point and argument parsing
│   ├── lib.rs             # Library exports
│   ├── app/               # Application state and configuration
│   ├── ui/                # Terminal UI and event loop
│   ├── tmux/              # Tmux client and pane information
│   ├── parsers/           # Agent output parsing logic
│   ├── agents/            # Agent type definitions and status
│   ├── monitor/           # Background monitoring task
│   └── cmd/               # Subcommands (learn, test)
├── tests/                 # Test fixtures and test runner
│   └── fixtures/          # Pane content test cases
├── scripts/               # Utility scripts
├── docs/                  # Documentation
├── .claude/               # Claude Code configuration and skills
├── .planning/             # GSD planning documents (generated)
├── Cargo.toml             # Rust package manifest
└── config.example.toml    # Example configuration file
```

## Directory Purposes

**`src/app/`:**
- Purpose: Application state management and configuration
- Contains: State structures, config loading, action definitions, key bindings
- Key files: `state.rs` (AppState), `config.rs` (Config), `actions.rs` (Action enum)

**`src/ui/`:**
- Purpose: Terminal user interface and event handling
- Contains: Main event loop, component rendering, layout calculation, styling
- Key files: `app.rs` (run_app, run_loop), `components/` (all widgets), `layout.rs` (constraint calculation)

**`src/tmux/`:**
- Purpose: Tmux interaction and pane metadata extraction
- Contains: Tmux command execution, pane information parsing, process detection
- Key files: `client.rs` (TmuxClient), `pane.rs` (PaneInfo, process cache)

**`src/parsers/`:**
- Purpose: Parsing agent output from pane content
- Contains: Parser trait, universal parser implementation, registry
- Key files: `mod.rs` (trait definition), `universal.rs` (regex-based parser)

**`src/agents/`:**
- Purpose: Agent type definitions and status representation
- Contains: Agent status enums, approval types, subagent tracking
- Key files: `types.rs` (AgentStatus, ApprovalType, MonitoredAgent), `subagent.rs` (Subagent)

**`src/monitor/`:**
- Purpose: Background monitoring task polling tmux
- Contains: Monitor loop, notification handling, external TODO command execution
- Key files: `task.rs` (MonitorTask), `system_stats.rs` (CPU/memory collection)

**`src/cmd/`:**
- Purpose: CLI subcommands
- Contains: Learn mode (interactive agent definition wizard), test runner (regression tests)
- Key files: `learn.rs` (learn command), `test.rs` (test command)

**`tests/fixtures/`:**
- Purpose: Test case data
- Contains: Subdirectories per agent type (claude, open, codex, gemini, generic) with pane content files
- Structure: `{agent_type}/{case_name}` with `content`, `expected_status`, `rules.toml`

**`.claude/skills/`:**
- Purpose: Project-specific skill definitions for Claude Code
- Contains: Structured workflows for testing, commits, configuration, planning, changelog
- Key files: `tmuxx-testing.md`, `tmuxx-commit.md`, `tmuxx-adding-config-option.md`, etc.

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point, CLI argument parsing, config loading
- `src/lib.rs`: Library module exports
- `src/ui/app.rs::run_app()`: Main application initialization and event loop startup

**Configuration:**
- `src/app/config.rs`: Config structure with defaults
- `src/app/config_override.rs`: Runtime config override from CLI flags
- `src/app/key_binding.rs`: Keyboard shortcut definitions
- `src/app/menu_config.rs`: Command menu structure
- `Cargo.toml`: Project metadata and dependencies

**Core Logic:**
- `src/app/state.rs`: AppState (all mutable app state), AgentTree, PopupInputState
- `src/app/actions.rs`: Action enum defining all possible user actions
- `src/monitor/task.rs`: MonitorTask::run() - background polling loop
- `src/parsers/mod.rs`: AgentParser trait, ParserRegistry
- `src/parsers/universal.rs`: Regex-based status and subagent parsing

**UI Components:**
- `src/ui/components/agent_tree.rs`: Agent list sidebar widget
- `src/ui/components/pane_preview.rs`: Pane content display widget
- `src/ui/components/header.rs`: Top status bar
- `src/ui/components/input.rs`: Bottom input buffer
- `src/ui/components/menu_tree.rs`: Hierarchical command menu
- `src/ui/components/popup_input.rs`: Text input dialog
- `src/ui/components/modal_textarea.rs`: Multi-line input dialog

**Tmux Interaction:**
- `src/tmux/client.rs`: TmuxClient with list_panes, capture_pane, send_keys, send_command
- `src/tmux/pane.rs`: PaneInfo, ProcessTreeCache, process detection logic

**Testing:**
- `src/cmd/test.rs`: Test runner reading fixtures and verifying parser output
- `tests/fixtures/`: Test case organization
- Each module has `#[cfg(test)] mod tests` with unit tests

## Naming Conventions

**Files:**
- Snake case: `config.rs`, `agent_tree.rs`, `monitor_task.rs`
- Test files: Module-internal under `#[cfg(test)]`, not separate files
- Fixture files: No extension for content, explicit names for rules: `tests/fixtures/{agent}/{case}/content`, `tests/fixtures/{agent}/{case}/rules.toml`

**Functions:**
- Snake case: `load_merged()`, `parse_status()`, `send_keys()`
- Action handlers: `handle_approve()`, `handle_reject()`, `handle_focus_pane()`
- Async functions prefixed with action: `run()`, `refresh_process_cache()`

**Variables:**
- Snake case: `agent_status`, `capture_lines`, `filter_pattern`
- Mutable: Same convention, no `mut_` prefix
- Constants: UPPER_CASE: `SUMMARY_HEIGHT`, `PREVIEW_MIN_HEIGHT`, `DEFAULT_KEYS`

**Types:**
- PascalCase: `AppState`, `MonitoredAgent`, `AgentStatus`, `TmuxClient`
- Enums: PascalCase variants: `AgentStatus::Idle`, `FocusedPanel::Sidebar`
- Traits: PascalCase: `AgentParser`, `Widget`

## Where to Add New Code

**New Feature (e.g., new status detection logic):**
- Primary code: `src/parsers/universal.rs` (add regex rule)
- Config: `src/app/config.rs` (if configurable)
- Tests: `src/parsers/mod.rs::tests` or `tests/fixtures/` for parser verification
- Integration: `src/monitor/task.rs` (already calls parser, no change needed)

**New Component/Widget:**
- Implementation: `src/ui/components/{component_name}.rs`
- Export: Add to `src/ui/components/mod.rs`
- Integration: Add to render in `src/ui/app.rs::run_loop()` in layout section
- State: If stateful, add to `AppState` in `src/app/state.rs`

**New Configuration Option:**
- Structure: `src/app/config.rs` (add field with serde)
- CLI override: `src/app/config_override.rs` (add override case)
- Usage: Pass through to consuming code (e.g., monitor, UI, client)
- Defaults: `src/app/config.rs::Default` impl or `#[serde(default)]` attribute

**New Action/Keybinding:**
- Action: Add variant to `Action` enum in `src/app/actions.rs`
- Key binding: `src/app/key_binding.rs` (add to KeyBindings struct)
- Handler: `src/ui/app.rs::run_loop()` in action dispatch section
- Validation: Unit test in action module

**New Subcommand:**
- Implementation: `src/cmd/{subcommand}.rs` with public async run function
- Registration: Update `Commands` enum in `src/main.rs`
- Dispatch: Add case in main's command handler
- Export: Add to `src/cmd/mod.rs`

**Utilities/Helpers:**
- Shared across modules: `src/app/mod.rs` or module-specific `mod.rs`
- String utilities: `src/parsers/mod.rs::safe_tail()` (safe multi-byte handling)
- Process utilities: `src/tmux/pane.rs::ProcessTreeCache` (cached subprocess lookup)

## Special Directories

**`.planning/codebase/`:**
- Purpose: GSD (Claude Code tool) codebase documentation
- Generated: Yes, by `/gsd:map-codebase` command
- Committed: Yes, guides future implementation
- Contents: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md, CONCERNS.md

**`tests/fixtures/`:**
- Purpose: Test case data for regression testing
- Generated: No (manually created via `tmuxx learn` command or direct editing)
- Committed: Yes, critical for test stability
- Structure: `{agent_type}/{case_name}/content`, `{agent_type}/{case_name}/rules.toml`

**`.claude/skills/`:**
- Purpose: Reusable workflows for Claude Code assistant
- Generated: No (manually created and maintained)
- Committed: Yes, project-specific development guides
- Pattern: SKILL.md files (uppercase) defining workflows, tools, common mistakes

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes
- Committed: No (in .gitignore)

**`tmp/`:**
- Purpose: Temporary files for development/testing
- Generated: Yes
- Committed: No (user-specific, in .gitignore)

---

*Structure analysis: 2026-01-30*
