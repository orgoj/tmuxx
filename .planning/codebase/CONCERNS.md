# Codebase Concerns

**Analysis Date:** 2026-01-30

## Tech Debt

**Large File Complexity (UI App Event Loop):**
- Issue: `src/ui/app.rs` is 1978 lines with deeply nested event handling logic spanning 800+ lines of match statements. Event handling for menu navigation, input, and actions is difficult to follow and modify.
- Files: `src/ui/app.rs`
- Impact: Difficult to add new keybindings, high risk of regression when modifying event flow, slow development velocity for UI changes.
- Fix approach: Extract event handlers into separate modules (menu_handler.rs, input_handler.rs, action_handler.rs). Use a visitor pattern or separate match branches into functions to reduce nesting depth.

**Configuration State Management:**
- Issue: `src/app/state.rs` (1444 lines) and `src/app/config.rs` (1351 lines) together handle both runtime state and configuration. State mutations happen in multiple places (app.rs, modal handlers, menu handlers), making it hard to track state changes.
- Files: `src/app/state.rs`, `src/app/config.rs`, `src/ui/app.rs`
- Impact: Difficult to predict behavior after config reload, high risk of stale state during dynamic reconfigurations, potential for desync between config and runtime state.
- Fix approach: Implement explicit state machine for configuration reload lifecycle. Add validation layer that runs on both initial load and reload. Document state invariants.

**Parser Registry and Configuration Validation:**
- Issue: `src/parsers/mod.rs` and `src/parsers/universal.rs` (766 lines) compile regex patterns lazily in UniversalParser constructor. Regex compilation errors only surfaced at runtime when agent is first detected. No validation of agent patterns at config load time.
- Files: `src/parsers/universal.rs`, `src/app/config.rs`
- Impact: Invalid regex in agent config not caught until the agent appears in tmux, may cause monitor loop to hang or crash. Users don't know about malformed config until they use it.
- Fix approach: Add config validation phase in `Config::load()` that compiles all regexes and checks syntax. Report all validation errors at startup before TUI initializes.

**Async Command Execution Without Timeout:**
- Issue: External TODO command (`src/monitor/task.rs` line 107-130) and notification command spawning use `tokio::spawn()` without timeout limits or resource limits. A hanging external command can block the monitor thread.
- Files: `src/monitor/task.rs` (lines 107-130), `src/monitor/task.rs` (lines 439-445)
- Impact: Misconfigured TODO command can freeze the entire UI, zombie processes may accumulate if commands hang, no way to recover without killing the app.
- Fix approach: Add timeout wrapper for all external command execution. Kill processes that exceed `todo_refresh_interval_ms` or `notification_delay_ms` + 5s. Add configurable command timeout option.

## Known Bugs

**Menu Variable Collection State Not Cleared on Cancellation:**
- Symptoms: When user cancels menu variable input popup mid-collection, the `MenuVariableInput` popup type state persists with partially collected variables. Reopening the menu or pressing the trigger key again may restore the incomplete state instead of starting fresh.
- Files: `src/app/state.rs` (MenuVariableInput variant), `src/ui/app.rs` (popup handling around line 1100+)
- Trigger: Start menu item that requires variables → type first variable → press Escape/cancel → reopen same menu
- Workaround: Reload config with `Ctrl+r` to reset all state, or switch focus and back to clear the popup.

**Selection Index Can Exceed Filtered List Bounds:**
- Symptoms: When toggling agent filters (`ToggleFilterActive`, `ToggleFilterSelected`), the selected agent index may point beyond the filtered list. Subsequent navigation produces unexpected behavior (skips items, crashes with index out of bounds).
- Files: `src/ui/app.rs` (filter toggle around line 1400+), `src/app/state.rs` (agent tree selection)
- Trigger: Select an agent → toggle active filter → navigate with arrow keys
- Workaround: Press Escape to reset selection, or toggle filter again to restore visibility.

## Security Considerations

**Shell Injection via Notification Command:**
- Risk: Notification command template placeholder replacement happens at line 424-435 in `src/monitor/task.rs`. While placeholders are shell-escaped via `shell_escape()`, the final command is executed via `bash -c`. If a placeholder value contains special bash syntax, escaping may be insufficient.
- Files: `src/monitor/task.rs` (lines 404-446)
- Current mitigation: Single-quote escaping of all dynamic values using `format!("'{}'", s.replace('\'', "'\\''"))`. Hardcoded placeholders like `{title}` and `{approval_type}` are not escaped.
- Recommendations:
  1. Use `bash` array-based execution or `sh` `-c` with more restricted environment.
  2. Consider moving notification to dedicated spawned process that takes args as CLI args instead of template substitution.
  3. Add security audit of all user-configurable commands (notification_command, todo_command, terminal_wrapper, menu item commands).

**Command Execution via Terminal Wrapper and Menu Commands:**
- Risk: User-provided `terminal_wrapper` config and menu item commands are expanded with variables (agent path, session, target) and executed via `bash -c` at lines 1200-1350 in `src/ui/app.rs`. Variable expansion is done via string replacement without validation.
- Files: `src/ui/app.rs` (lines 1155-1370), `src/app/config.rs` (terminal_wrapper field)
- Current mitigation: Variables come from tmux session/pane data and should be safe, but no sanitization is applied.
- Recommendations:
  1. Use structured command execution instead of template strings (pass variables as env vars or process args).
  2. Validate all user-provided command templates at config load time to catch shell metacharacters.
  3. Document that terminal_wrapper and menu commands should be simple, not complex shells scripts.

**Unsafe Libc Call for Process Signals:**
- Risk: `src/tmux/client.rs` line 202 uses `unsafe { libc::kill(pid, libc::SIGTERM) }` to send SIGTERM. While SIGTERM is safe, this is the only `unsafe` block in the codebase and could be misused if extended.
- Files: `src/tmux/client.rs` (lines 202-209)
- Current mitigation: Only SIGTERM is sent, not arbitrary signals. PID is validated by parsing tmux output.
- Recommendations:
  1. Consider using `nix` crate's safe wrapper for signal handling instead of direct libc.
  2. Add comments explaining why unsafe is necessary and what precautions are taken.

**Regex Denial of Service (ReDoS):**
- Risk: Agent config allows user-defined regex patterns in state rules, splitter patterns, and refinement rules. Maliciously crafted regexes could cause catastrophic backtracking when applied to captured pane output.
- Files: `src/parsers/universal.rs` (regex compilation), `src/app/config.rs` (agent pattern definitions)
- Current mitigation: None. Regexes are compiled at parser creation with no timeout or complexity checking.
- Recommendations:
  1. Add timeout to regex operations (set deadline for pattern matching, kill if exceeded).
  2. Document that agent patterns are user-configurable and should be simple.
  3. Consider adding max regex complexity limit or switch to simpler pattern matching (fnmatch style globs instead of full regex).

## Performance Bottlenecks

**Regex Compilation on Every Poll Cycle:**
- Problem: `src/monitor/task.rs` line 272 compiles process indicator regexes for each agent on every poll cycle. With N agents and M indicators, this is O(N*M) regex compilations per poll (default 500ms).
- Files: `src/monitor/task.rs` (lines 272-282), `src/parsers/universal.rs`
- Cause: Process indicators are defined per-agent in config but regexes are not pre-compiled during parser creation.
- Improvement path: Pre-compile all indicator regexes during `UniversalParser::new()`. Cache compiled patterns in the parser struct.

**Pane Content Capture Redundancy:**
- Problem: Monitor captures pane content twice per cycle: once for content-based matcher validation (line 204-210), again for status parsing (line 228-235). If pane has large output (>16KB), this doubles memory and tmux command overhead.
- Files: `src/monitor/task.rs` (lines 200-235)
- Cause: Design captures on-demand when content-based matching is needed, but doesn't cache the result.
- Improvement path: Implement single-pass capture with immediate caching. Pass captured content through matcher validation before parsing.

**UI Redraw on Every Event:**
- Problem: Main loop forces full terminal redraw on every keypress or monitor update (no conditional redraw). With 60+ FPS potential, this wastes CPU even when state doesn't change.
- Files: `src/ui/app.rs` (render loop around line 500+)
- Cause: Event loop redraws unconditionally, relies on terminal refresh rate to throttle rather than explicit dirty-flag checking.
- Improvement path: Already partially optimized (v0.2.3 moved to event-driven rendering). Verify that only state changes trigger redraws, not every tick.

**Agent Tree Sorting on Every Poll:**
- Problem: Line 314 in `src/monitor/task.rs` sorts all agents by target on every poll cycle: `tree.root_agents.sort_by(|a, b| a.target.cmp(&b.target))`. With 50+ agents, this is O(N log N) comparison overhead per 500ms.
- Files: `src/monitor/task.rs` (line 314)
- Cause: Ensures consistent ordering across redraws, but agents already come from tmux in consistent order.
- Improvement path: Remove sorting if tmux already returns consistent order. Add explicit ordering guarantee by sorting once at startup if needed, not per-poll.

## Fragile Areas

**Splitter-Based Content Parsing Model:**
- Files: `src/parsers/universal.rs` (lines 8-51)
- Why fragile: The "Splitter Model" depends on exact formatting of Claude/Pi prompts. If Claude's UI changes slightly (different separator line, moved powerline box), detection breaks silently. Tests exist but don't cover all Claude versions and configurations.
- Safe modification: Add regression tests for each Claude major version. Document the exact formatting assumptions. Add fallback splitter when none match (don't fail silently).
- Test coverage: Fixtures cover common cases but not edge cases (wrapped prompts, very long paths, special characters in agent names).

**Agent Detection Based on Process Ancestry:**
- Files: `src/tmux/pane.rs`, `src/monitor/task.rs` (process detection)
- Why fragile: Detection uses process command matching (e.g., "claude", "pi", "node containing gemini") via ancestor process inspection. Changes to CLI tool naming, shell wrappers, or process tree structure break detection.
- Safe modification: Add explicit agent ID config to allow manual override. Log detection confidence scores. Add --debug mode that shows detected agent type and match strength.
- Test coverage: Process detection is hard to unit test without mocking. Current tests check final agent types but not detection logic.

**Custom Parser Configuration Compilation:**
- Files: `src/parsers/universal.rs` (UniversalParser::new), `src/app/config.rs`
- Why fragile: Each custom agent config creates a new UniversalParser with compiled regexes. Config reload recompiles all parsers. If a regex pattern becomes invalid, parser creation fails at runtime during agent polling, not at config load.
- Safe modification: Validate all regex patterns during `Config::load()` before any parser is created. Implement `Config::validate()` that can be called to check syntax without creating parsers.
- Test coverage: No unit tests for invalid config handling. Test suite should include invalid regex, empty patterns, circular references.

## Scaling Limits

**Monitor Channel Capacity:**
- Current capacity: 32 messages (line 58 in `src/ui/app.rs`)
- Limit: If UI event loop is slow or blocked, monitor updates queue up. With 100+ agents and 500ms poll interval, 32 is tight for bursiness.
- Scaling path: Increase channel capacity to 128-256. Add metric tracking for queue depth. Alert if queue overflows (messages dropped).

**Agent Tree Memory Usage:**
- Current: Each MonitoredAgent stores last_content (full pane output up to capture_buffer_size). With 50 agents and 16KB per agent, baseline is 800KB.
- Limit: With 200+ agents and large outputs, memory can reach 3-4MB. No memory limits or content pruning.
- Scaling path: Implement configurable content retention (keep last 10 lines only, or last N bytes). Add memory usage metric to status bar.

**Regex Pattern Matching Latency:**
- Current: Agent status parsing applies up to 20+ regex rules per pane per poll cycle.
- Limit: With 50 agents and 500ms poll, each pattern match must complete in <10ms to avoid lag. Complex patterns risk timeout.
- Scaling path: Profile regex performance. Precompile patterns. Add timeout wrappers. Consider switching to simpler glob-based matching for less critical patterns.

## Dependencies at Risk

**Regex Crate Vulnerability Surface:**
- Risk: Untrusted regex patterns from user config could trigger ReDoS (Regex Denial of Service). Regex crate has had vulnerabilities in backtracking behavior.
- Impact: Malicious config file could freeze the app indefinitely.
- Migration plan: Switch to `fancy-regex` with timeout support, or add explicit timeout wrapper around all `is_match()` and `captures()` calls.

**Tokio Panic on Unhandled Spawned Task Errors:**
- Risk: `tokio::spawn()` at lines 107, 1210 do not await or handle results. If spawned task panics, it's silently logged as warn/error but UI continues. Multiple panics could accumulate.
- Impact: Undetected failures in external commands, notification sending, or menu execution.
- Migration plan: Use task JoinHandle to track spawned tasks. Collect errors in a thread-safe queue and display in UI status bar.

## Missing Critical Features

**Config Validation at Load Time:**
- Problem: Invalid regex patterns, missing required fields, incompatible option combinations are only discovered at runtime when they're used.
- Blocks: User doesn't know their config is broken until they start the app and something fails during polling.
- Solution: Implement `Config::validate()` method that checks:
  1. All regex patterns compile successfully
  2. All required colors are valid
  3. All external commands (todo_command, notification_command, terminal_wrapper) exist or are reasonable (bash code)
  4. Agent IDs are unique
  5. Key bindings don't conflict

**Process Timeout Management:**
- Problem: External commands (todo, notification, menu execution) have no timeout. A hanging command blocks the monitor or UI event loop indefinitely.
- Blocks: User experience degrades when a misconfigured todo_command hangs.
- Solution: Add `[timing]` config section with `command_timeout_ms` (default 5000). Wrap all `Command::new()` and `tokio::process::Command` calls with timeout wrapper that kills process after threshold.

**Resource Limits and Monitoring:**
- Problem: No memory limits, no CPU limits, no file descriptor tracking. App can exhaust system resources with many agents and large pane outputs.
- Blocks: On resource-constrained systems (VPS, Raspberry Pi), tmuxx could OOM or hit fd limits.
- Solution: Add configurable limits for `max_agents`, `max_content_size`, `max_memory_mb`. Track and display resource usage in header. Add --no-memory-limits flag for unconstrained environments.

## Test Coverage Gaps

**Process Detection Logic:**
- What's not tested: The ancestor process matching logic in `src/monitor/task.rs` and `src/tmux/pane.rs` is not covered by unit tests. Hard to test without mocking /proc filesystem.
- Files: `src/tmux/pane.rs`, `src/monitor/task.rs` (process detection)
- Risk: Changes to detection logic could break agent discovery. No regression suite for detection across different shell wrappers.
- Priority: High - detection is critical path. Recommend adding integration tests with real tmux sessions.

**Configuration Reload Path:**
- What's not tested: Config reload via `Ctrl+r` is not tested. State inconsistency during reload is not covered. Reload during active operations (menu open, modal visible) not tested.
- Files: `src/ui/app.rs` (ReloadConfig action), `src/app/state.rs`, `src/app/config.rs`
- Risk: Reload could leave state in inconsistent state, lose user input, or crash if reload happens during critical operation.
- Priority: Medium - reload is relatively new feature. Add tests for reload during modal/menu/input buffer active.

**Menu Variable Collection:**
- What's not tested: The menu variable input flow (multi-step variable collection popup) is not tested. Edge cases like empty variables, special characters, very long values not covered.
- Files: `src/app/state.rs` (MenuVariableInput), `src/ui/app.rs` (menu handling)
- Risk: Users might lose data or encounter crashes during variable collection. Variable substitution could fail silently.
- Priority: Medium - variable feature is new. Add tests for all menu item types, variable validation, edge cases.

**Command Execution:**
- What's not tested: Menu command execution with variable expansion, external terminal spawning, and blocking command error handling are not tested.
- Files: `src/ui/app.rs` (ExecuteCommand action handling, lines 1155-1370)
- Risk: Commands with special characters in variables might fail. Spawned processes might not clean up. Error reporting might be unhelpful.
- Priority: Medium - critical for usability. Add tests for command expansion, process spawning, error cases.

**Notification System:**
- What's not tested: Notification sending, shell escaping, timeout behavior, and mode switching (First vs Each) not tested.
- Files: `src/monitor/task.rs` (lines 323-446)
- Risk: Shell injection via improperly escaped values. Notifications might spam or silently fail. Mode switching might leave inconsistent state.
- Priority: Low - notification is optional feature. Add tests for all mode transitions and shell escaping edge cases.

---

*Concerns audit: 2026-01-30*
