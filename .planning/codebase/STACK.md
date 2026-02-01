# Technology Stack

**Analysis Date:** 2026-01-30

## Languages

**Primary:**
- Rust 2021 edition - Complete application implementation including TUI, tmux client, parsers, and async runtime

## Runtime

**Environment:**
- Rust 1.70+ (stable toolchain via `dtolnay/rust-toolchain@stable`)

**Package Manager:**
- Cargo (Rust's built-in package manager)
- Lockfile: `Cargo.lock` (present, committed)

## Frameworks

**Core:**
- Ratatui 0.29 - Terminal UI framework for rendering dashboard and components
- Tokio 1 (full features) - Async runtime for background monitoring task and async operations
- Crossterm 0.28 - Cross-platform terminal handling (input, colors, terminal control)

**CLI:**
- Clap 4 (derive feature) - Command-line argument parsing with procedural macro support (`Cli`, `Commands` enums in `src/main.rs`)

**Configuration:**
- TOML 0.8 - Config file parsing and serialization (supports `~/.config/tmuxx/config.toml`)
- Serde 1 (derive feature) - Serialization/deserialization framework for config structures

**Testing:**
- Cargo built-in test framework (unit tests in modules)
- No external test framework (direct Rust #[cfg(test)] modules)

**Build/Dev:**
- Cargo fmt - Code formatting (CI requirement)
- Cargo clippy - Linting (CI requirement, enforces zero warnings)
- Cargo check - Type checking
- Cargo test - Unit testing

## Key Dependencies

**Critical:**
- Ratatui 0.29 - Required for all TUI rendering (`src/ui/`)
- Tokio 1 - Required for async monitor task (`src/monitor/task.rs`)
- Crossterm 0.28 - Required for terminal control (keyboard input, styling)

**Infrastructure:**
- Regex 1 - Pattern matching for agent status detection in parsers (`src/parsers/`)
- Tracing 0.1 - Structured logging framework
- Tracing-subscriber 0.3 (env-filter feature) - Log subscriber with environment filtering (debug mode writes to `tmuxx.log`)
- Sysinfo 0.32 - System resource monitoring (CPU/memory) for header display (`src/monitor/system_stats.rs`)

**Utilities:**
- Chrono 0.4 - Date/time handling
- Dirs 5 - Cross-platform config directory discovery (`~/.config/tmuxx/config.toml` location)
- Unicode-width 0.1 - Text width calculation for safe truncation of multi-byte characters
- Parking_lot 0.12 - Fast synchronization primitives for Arc<Mutex<>> operations
- Anyhow 1 - Error handling with context chaining
- Glob 0.3 - File glob pattern matching for TODO file discovery
- Libc 0.2 - Low-level system calls (process detection)
- Fuzzy-matcher 0.3 - Fuzzy matching for menu search/filtering
- Tui-textarea 0.7 - Multi-line text input widget for modal dialogs

**Dev Dependencies:**
- Tempfile 3 - Temporary file creation for testing

## Configuration

**Environment:**
- Configuration file: `~/.config/tmuxx/config.toml` (Linux/macOS, discovered via `dirs` crate)
- CLI arguments override config file (merging strategy in `src/main.rs` lines 139-144)
- Default config auto-generated with `tmuxx --init-config` command

**Config Loading Order (src/app/config.rs):**
1. CLI args (highest priority): `--poll-interval`, `--capture-lines`, `--set KEY=VALUE`
2. User config file: `~/.config/tmuxx/config.toml`
3. Built-in defaults (hardcoded in code)

**Build:**
- `Cargo.toml` - Project manifest with dependencies and metadata
- `.github/workflows/ci.yml` - GitHub Actions CI pipeline
- `.github/workflows/release.yml` - Automated release builds
- `config.example.toml` - Example configuration for users

## Platform Requirements

**Development:**
- Rust 1.70+ toolchain (stable)
- Git (for version control and CI)
- Tmux running with panes/agents (for testing, not for compilation)

**Production:**
- Linux (primary target, tested in CI) or macOS (tested in CI `macos-latest`)
- Tmux installed and running (no version requirement stated)
- Terminal with 256-color or truecolor support recommended (Ratatui handles fallback)
- Desktop notifications command available (if `notification_command` configured, e.g., `notify-send` on Linux)

**Runtime Dependencies:**
- System running tmux (panes, sessions, commands)
- Optional: `notify-send` or equivalent for desktop notifications (configurable via `notification_command`)
- Optional: External commands specified in config (e.g., `zed`, `git`, `wezterm`, `pcmanfm-qt`)

## Version

- Current: 0.4.6 (in `Cargo.toml`)
- Semantic versioning (major.minor.patch)

---

*Stack analysis: 2026-01-30*
