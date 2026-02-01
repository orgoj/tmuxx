# Coding Conventions

**Analysis Date:** 2026-01-30

## Naming Patterns

**Files:**
- Rust modules use `snake_case` (e.g., `config.rs`, `key_binding.rs`, `system_stats.rs`)
- Module files in subdirectories use descriptive names: `src/agents/types.rs`, `src/parsers/universal.rs`
- Barrel files use `mod.rs` to export public APIs
- Test fixtures are named with convention: `case_{status}_{description}.txt` (e.g., `case_approval_create.txt`, `case_working_1.txt`)

**Functions:**
- Use `snake_case` (e.g., `parse_status()`, `capture_pane()`, `list_panes()`)
- Prefix boolean getters with `is_`, `has_`, `should_` (e.g., `is_available()`, `should_ignore_session()`)
- Action methods use verbs: `run()`, `parse()`, `refresh()`, `collect()`
- Initialization methods: `new()` for constructors, `with_*()` for builder pattern (e.g., `with_capture_lines()`)

**Variables:**
- Use `snake_case` for all variables and fields
- Boolean fields are prefixed with descriptive intent: `show_detached_sessions`, `truncate_long_lines`, `cyclic_navigation`, `ignore_self`
- Configuration structures use plural names for collections: `agents`, `ignore_sessions`, `highlight_rules`, `global_highlight_rules`
- Struct fields use clear, explicit names: `poll_interval_ms`, `capture_lines`, `buffer_size`, `approval_since`

**Types:**
- Use `PascalCase` for all struct and enum names
- Enums represent state/choices: `AgentStatus`, `ApprovalType`, `PopupType`, `FocusedPanel`, `MessageKind`
- Traits use `-er` suffix when appropriate: `AgentParser`
- Generic type parameters are descriptive: `T`, `E` for error (not single letters unless standard)
- Newtype wrappers use descriptive names: `PaneInfo`, `AgentTree`, `SystemStats`

## Code Style

**Formatting:**
- Rust edition 2021
- Enforced by `cargo fmt` (required before commit)
- 120 character soft line limit (enforced implicitly by team)
- Clap derives use `#[command(...)]` attributes for CLI metadata

**Linting:**
- Enforced by `cargo clippy` (zero warnings policy - all clippy warnings must be fixed)
- No unsafe code blocks without explicit documentation
- No dead code, unused imports, or unreachable patterns

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate imports (alphabetically: `anyhow`, `chrono`, `clap`, `crossterm`, `regex`, `ratatui`, `serde`, `tokio`, `tracing`, `unicode-width`)
3. Local crate imports (`use crate::...`)
4. Module declarations (`mod ...`)

**Path Aliases:**
- No path aliases configured
- All imports use full paths from crate root: `use crate::app::Config`, `use crate::agents::AgentStatus`
- Module structure is flat enough to avoid aliases

**Examples:**
- `src/main.rs`: Import from `tmuxx::app::Config`, `tmuxx::ui::run_app`
- `src/cmd/test.rs`: Import from `crate::agents::AgentStatus`, `crate::parsers::UniversalParser`
- `src/tmux/client.rs`: Import from `super::pane::PaneInfo`, `crate::app::{Config, KillMethod}`

## Error Handling

**Patterns:**
- Use `anyhow::Result<T>` for fallible operations
- All async functions and top-level commands return `Result<()>` or `Result<T>`
- Use `.context()` to add context to errors before propagation: `.context("Failed to execute tmux list-panes")?`
- Use `anyhow::bail!()` for explicit error propagation with message: `anyhow::bail!("tmux send-keys failed for {}: {}", target, stderr)`
- Use `unwrap_or()` with fallback behavior for recoverable errors (e.g., config loading)
- Use `unwrap_or_else()` with closures for error logging: `.unwrap_or_else(|e| { eprintln!("..."); std::process::exit(1) })`

**Examples from codebase:**
```rust
// File: src/tmux/client.rs, lines 53-61
let output = Command::new("tmux")
    .args([...])
    .output()
    .context("Failed to execute tmux list-panes")?;

// File: src/tmux/client.rs, lines 100-101
anyhow::bail!("tmux capture-pane failed for {}: {}", target, stderr);

// File: src/main.rs, lines 134-137
Config::load_from(config_path).unwrap_or_else(|e| {
    eprintln!("Failed to load config file: {}", e);
    std::process::exit(1);
})
```

## Logging

**Framework:** `tracing` crate with `tracing_subscriber`

**Patterns:**
- Use `tracing::debug()`, `tracing::info()`, `tracing::warn()`, `tracing::error()`
- Debug logging enabled with `--debug` flag which writes to `tmuxx.log`
- File-based debug logging separate from application output
- Structured logging with trace messages (see `src/monitor/task.rs` for usage)

**Example:**
```rust
// File: src/monitor/task.rs, line 8
use tracing::{debug, error, info, warn};

// Usage in code:
debug!("Agent state changed: {:?}", agent);
warn!("Failed to parse agent output");
```

## Comments

**When to Comment:**
- Complex algorithms or status detection rules need explanation
- Parser regex patterns benefit from comments describing what they match
- Splitter model uses comments to document detection flow
- Configuration defaults include inline documentation
- TODO/FIXME comments discouraged - use codebase issues instead

**JSDoc/TSDoc:**
- Use Rust doc comments (`///`) for public items
- Document public structs with field descriptions
- Document public methods with usage examples when complex
- Include `# Example` sections for complex trait implementations

**Example from code:**
```rust
// File: src/app/state.rs, lines 76-82
/// Tree structure containing all monitored agents
#[derive(Debug, Clone, Default)]
pub struct AgentTree {
    /// Root agents (directly in tmux panes)
    pub root_agents: Vec<MonitoredAgent>,
}
```

## Function Design

**Size:**
- Prefer small, single-responsibility functions
- Complex parsing logic extracted into helper methods
- Parser registry keeps methods focused on matching/parsing
- No explicit line limit but avoid 300+ line functions

**Parameters:**
- Use config objects rather than many parameters: `Config`, `AgentConfig`
- String references for pane targets: `&str` for target
- Owned Vecs for builder patterns: `Vec<String>` for ignore patterns
- Use references for large objects: `&Config`, `&[&str]` for detection strings

**Return Values:**
- Always return `Result<T>` for fallible operations
- Use `Option<T>` for optional values
- Return owned values for small types (`String`, `Vec<T>`)
- Return references for borrowed data from stable structures

**Examples:**
```rust
// File: src/tmux/client.rs, lines 50-88
pub fn list_panes(&self) -> Result<Vec<PaneInfo>> { ... }

// File: src/app/config.rs, line 381
pub fn should_ignore_session(&self, name: &str, current_session: Option<&str>) -> bool { ... }

// File: src/parsers/mod.rs, lines 10-18
pub(crate) fn safe_tail(s: &str, max_chars: usize) -> &str { ... }
```

## Module Design

**Exports:**
- Use barrel files (`mod.rs`) to control public API
- Re-export commonly used types in parent module
- Keep internal modules private unless needed elsewhere
- Example: `src/app/mod.rs` exports `Action`, `Config`, `AppState`

**Barrel Files:**
- Every directory with multiple modules has `mod.rs`
- `mod.rs` imports submodules and selectively re-exports
- Example: `src/app/mod.rs` (lines 1-13) exports public API while keeping `actions`, `config_override` private

**Module Structure:**
- `src/agents/` - Agent types and monitoring data structures
- `src/app/` - Application state, config, actions, key bindings
- `src/cmd/` - Subcommands (learn, test)
- `src/monitor/` - Background monitoring task and system stats
- `src/parsers/` - Parser trait and implementations (universal parser)
- `src/tmux/` - Tmux client and pane info parsing
- `src/ui/` - Ratatui UI components and application loop

## Configuration Management

**Pattern:**
- Single `Config` struct in `src/app/config.rs` with 50+ fields
- All booleans have explicit defaults using `#[serde(default)]` or `#[serde(default = "default_true")]`
- Numeric values use helper functions for defaults: `default_buffer_size()`, `default_true()`
- Config supports TOML serialization/deserialization with `#[serde(deny_unknown_fields)]`
- CLI args override file config via `apply_override()` method
- Config file location: `~/.config/tmuxx/config.toml` (platform-specific via `dirs` crate)

**Examples:**
```rust
// File: src/app/config.rs, lines 25-145
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub poll_interval_ms: u64,

    #[serde(default = "default_buffer_size")]
    pub capture_buffer_size: usize,

    #[serde(default = "default_true")]
    pub cyclic_navigation: bool,
}
```

## String Handling

**Multi-byte Characters:**
- Use `.chars()` for character-based iteration (never byte indexing)
- Use `unicode-width` crate for text truncation calculations
- Safe tail extraction helper in `src/parsers/mod.rs`: `safe_tail(s: &str, max_chars: usize) -> &str`
- Terminal display width handled by Ratatui's width calculations

## Type System Patterns

**Enums for State:**
- `AgentStatus` represents agent execution state (Idle, Processing, AwaitingApproval, Error, Unknown)
- `ApprovalType` represents types of approvals (FileEdit, FileCreate, ShellCommand, UserQuestion, etc.)
- `PopupType` represents active dialog type (Filter, GeneralInput, RenameSession, MenuVariableInput, etc.)
- `FocusedPanel` represents UI focus (Sidebar or Input)

**Trait Implementations:**
- `AgentParser` trait provides parsing abstraction for different agent types
- Implement `Default` for config structs to support default field values
- Implement `Display` for enums used in user-facing text (e.g., `AgentType`, `ApprovalType`)
- Use `Debug` derives for logging and debugging

**Optional Fields:**
- Use `Option<T>` for nullable config fields: `max_line_width: Option<u16>`, `terminal_wrapper: Option<String>`
- Parse optional values with default None behavior in serde
- Display logic handles Some/None cases explicitly

## Performance Considerations

**Regex Compilation:**
- Regex patterns compiled once per parser instance
- Parser registry kept in Arc for shared access
- No per-frame regex compilation

**Caching:**
- Process tree cached in `src/tmux/pane.rs` with refresh interval
- Parser registry instance reused throughout application lifecycle
- Config loaded once at startup, not re-parsed on each update

**Async/Await:**
- Monitor task runs async loop using tokio
- Polling interval configurable, default 500ms
- Single `tokio::main` runtime in main.rs
- No blocking I/O in async code paths

## Consistency Rules

1. All Result types are `anyhow::Result<T>` - never use `Result<T, Box<dyn Error>>`
2. All boolean config fields use `#[serde(default)]` with explicit defaults
3. All public methods in traits have doc comments with `//// Summary`
4. All error messages use `.context()` before `?` operator
5. All clippy warnings must be fixed before commit
6. All imports sorted: std, external crates (alphabetically), local crate
7. All enums use `derive(Debug, Clone, PartialEq, Eq)` for comparability
8. All unit tests use `#[cfg(test)]` module pattern with `#[test]` attribute

---

*Convention analysis: 2026-01-30*
