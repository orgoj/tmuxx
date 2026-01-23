# Changelog

All notable changes to this fork (orgoj/tmuxcc) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
