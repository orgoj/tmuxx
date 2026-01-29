# tmuxx - AI Agent Dashboard for tmux

## What This Is

tmuxx is a Rust TUI dashboard that monitors tmux panes and detects AI coding agent status in real-time. It provides native support for AI agents (Claude Code, OpenCode, Gemini CLI, Codex CLI) while serving as a general-purpose process monitoring tool for any tmux workflow.

## Core Value

Accurate, real-time monitoring of AI agent status with intuitive interaction. If agents are working, awaiting approval, or blocked - tmuxx shows it immediately and lets you respond without leaving the dashboard.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Multi-pane monitoring with real-time status updates — existing
- ✓ AI agent process detection (Claude, OpenCode, Gemini, Codex) — existing
- ✓ Status parsing (Idle, Processing, AwaitingApproval, Error) — existing
- ✓ Subagent tracking for spawned tasks — existing
- ✓ Multi-agent selection and batch operations — existing
- ✓ Input buffer for quick responses — existing
- ✓ TOML-based configuration system — existing
- ✓ Regex-based status detection with splitter model — existing
- ✓ Async background monitoring with configurable polling — existing
- ✓ Cross-platform support (Linux, macOS) — existing

### Active

<!-- Current scope. Building toward these. -->

(To be defined in future milestones - refactoring, new features, enhancements)

### Out of Scope

<!-- Explicit boundaries. -->

- Windows support — Unix-only tool (tmux requirement)
- Built-in tmux session management — tmuxx monitors, doesn't create/manage sessions
- Agent execution — tmuxx monitors agents, doesn't run them
- Remote tmux monitoring — local tmux only

## Context

**Current State:**
- Version 0.4.6, stable and functional
- Three-layer architecture: tmux client → parsers → TUI
- Ratatui 0.29 for UI, Tokio for async monitoring
- Codebase mapped in `.planning/codebase/`

**Development Philosophy:**
- Configuration-first: No hardcoded behaviors, everything user-configurable
- Stability: Zero regressions, 100% test pass rate before commits
- Extensibility: Easy to add new agent types, customize detection patterns
- Performance: Efficient polling, conditional redraws, minimal CPU usage

**Known Strengths:**
- Robust agent detection across multiple detection strategies
- Flexible regex-based parsing with priority system
- Selection persistence across monitor updates
- Comprehensive configuration system

**Known Areas for Growth:**
- Large files (ui/app.rs: 1978 lines, app/state.rs: 1444 lines, app/config.rs: 1351 lines)
- Potential for new agent integrations
- UI/UX refinements based on usage patterns

## Constraints

- **Platform**: Linux/macOS only (tmux dependency)
- **Tech Stack**: Rust stable 1.70+, Ratatui 0.29, Tokio async
- **Testing**: 100% regression test pass rate mandatory before commits
- **Performance**: Maintain low CPU usage (background monitoring, efficient parsing)
- **Compatibility**: Preserve existing configuration format across versions

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Ratatui for TUI | Type safety, performance, modern terminal framework | ✓ Good - solid foundation |
| Configuration-first design | User customization without code changes | ✓ Good - highly flexible |
| Regex-based parsing | Balance power and simplicity for status detection | ✓ Good - works well |
| Splitter model for parsing | Avoid false positives from terminal history | ✓ Good - accurate detection |
| Hard fork from tmuxcc | Total rewrite for clean architecture | ✓ Good - maintainable codebase |

---
*Last updated: 2026-01-30 after initialization*
