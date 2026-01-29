# Testing Patterns

**Analysis Date:** 2026-01-30

## Test Framework

**Runner:**
- Rust built-in test framework (no external test runner)
- All tests use `#[test]` attribute with `#[cfg(test)]` modules
- Run with: `cargo test`

**Assertion Library:**
- Rust standard `assert!()`, `assert_eq!()`, `assert_ne!()` macros
- No external assertion library (keep dependencies minimal)

**Run Commands:**
```bash
cargo test                          # Run all unit tests
cargo test --lib                    # Run library tests only
cargo test --test '*'               # Run integration tests (if any)
cargo run -- test                   # Run regression test suite
cargo run -- test --dir tests/fixtures     # Run specific test directory
cargo run -- test -d                # Run tests with debug output (explains matching)
```

## Test File Organization

**Location:**
- Unit tests co-located with source code using `#[cfg(test)] mod tests` pattern
- Test modules at bottom of source file after all implementation code
- Fixtures stored in `tests/fixtures/{agent_id}/case_{status}_{description}.txt`
- Test runner in `src/cmd/test.rs` for regression testing

**Naming:**
- Test functions: `test_{feature_being_tested}()`
- Examples: `test_default_config()`, `test_config_serialization()`, `test_apply_override()`, `test_keys_for_action_sorted()`
- Fixture files: `case_{status}_{description}.txt`
  - Status values: `idle`, `working`, `approval`, `error`, `processing`
  - Examples: `case_approval_create.txt`, `case_working_1.txt`, `case_error_exit_status.txt`

**Structure:**
```
src/
├── app/
│   ├── config.rs
│   │   └── #[cfg(test)] mod tests { fn test_default_config() {...} }
│   ├── key_binding.rs
│   │   └── #[cfg(test)] mod tests { fn test_keys_for_action() {...} }
│   └── config_override.rs
│       └── #[cfg(test)] mod tests { fn test_parse_bool() {...} }
└── ...
tests/
└── fixtures/
    ├── claude/
    │   ├── case_approval_create.txt
    │   ├── case_working_1.txt
    │   ├── case_working_2.txt
    │   └── ...
    ├── shell/
    │   ├── case_idle_capture.txt
    │   ├── case_error_exit_status.txt
    │   └── ...
    └── pi/
        ├── case_idle_pytest_output.txt
        ├── case_working_pytest_failure.txt
        └── ...
```

## Test Structure

**Suite Organization:**

Tests follow standard Rust pattern with inline assertions:

```rust
// File: src/app/config.rs, lines 1136-1146
#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.poll_interval_ms, 500);
    assert_eq!(config.capture_lines, 200);
    assert!(config.show_detached_sessions);
    assert!(!config.debug_mode);
    assert!(config.truncate_long_lines);
    assert!(config.log_actions);
    assert_eq!(config.max_line_width, None);
}
```

**Patterns:**

1. **Setup** - Create test data or config
2. **Execute** - Call function being tested
3. **Assert** - Verify expected results with multiple assertions per test

```rust
// File: src/app/key_binding.rs, lines 151-162
#[test]
fn test_keys_for_action() {
    // Setup
    let mut bindings = HashMap::new();
    bindings.insert("y".to_string(), KeyAction::Approve);
    bindings.insert("Y".to_string(), KeyAction::Approve);
    let kb = KeyBindings { bindings };

    // Execute
    let approve_keys = kb.keys_for_action(&KeyAction::Approve);

    // Assert
    assert!(approve_keys.contains(&"y".to_string()));
    assert!(approve_keys.contains(&"Y".to_string()));
    assert_eq!(approve_keys.len(), 2);
}
```

**Configuration Testing:**
```rust
// File: src/app/config.rs, lines 1148-1155
#[test]
fn test_config_serialization() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).unwrap();
    let parsed: Config = toml::from_str(&toml_str).unwrap();
    assert_eq!(config.poll_interval_ms, parsed.poll_interval_ms);
    assert_eq!(config.show_detached_sessions, parsed.show_detached_sessions);
}
```

## Regression Testing (Fixture-Based)

**Purpose:**
- Test parser's ability to correctly identify agent status from terminal output
- Capture real pane content and verify status detection is accurate
- Prevent parser regressions when modifying detection rules

**Fixture Format:**
- Plain text files containing actual captured pane output
- Filename encodes expected status: `case_{status}_{description}.txt`
- Content is raw terminal output that will be parsed

**Test Execution Flow:**

From `src/cmd/test.rs`:

1. Load config and find agent definitions
2. For each test directory (fixtures subdirectory): `claude/`, `shell/`, `pi/`, etc.
3. For each `.txt` file in directory:
   - Extract expected status from filename (idle, working, approval, error)
   - Read file content as pane output
   - Initialize parser for agent (Claude, Shell, Pi, etc.)
   - Call `parser.parse_status(content)` to get actual status
   - Compare discriminant of expected vs actual status enum
   - Print result: PASS (green) or FAIL (red)

**Status Mapping:**
```rust
// File: src/cmd/test.rs, lines 128-144
let expected_status_enum = match status_part {
    "idle" => AgentStatus::Idle { label: None },
    "working" | "processing" => AgentStatus::Processing { activity: "".to_string() },
    "approval" | "awaiting_approval" => AgentStatus::AwaitingApproval {
        approval_type: crate::agents::ApprovalType::Other("".to_string()),
        details: "".to_string(),
    },
    "error" => AgentStatus::Error { message: "".to_string() },
    _ => { /* unknown */ }
};

let actual_status = parser.parse_status(&content);
let is_match = std::mem::discriminant(&actual_status) == std::mem::discriminant(&expected_status_enum);
```

**Test Output:**
```
🧪 Running Regression Tests in tests/fixtures

📂 Found 5 test suites (subdirectories)

🔍 Test Suite: claude (Agent: claude)
  📄 case_approval_create.txt        Expected: approval         Got: approval         -> PASS
  📄 case_working_1.txt              Expected: working          Got: working          -> PASS
  📄 case_idle_sauteed.txt           Expected: idle             Got: idle             -> PASS
  📄 case_error_exit_status.txt      Expected: error            Got: error            -> PASS

📊 Total Results: 85 Passed, 0 Failed
```

**Debug Mode:**
```bash
cargo run -- test -d
```

Adds detailed explanation output showing which detection rules matched:
- Prints explanation from `parser.explain_status(&content)`
- Shows regex match details for debugging splitter/refinement rules
- Useful when adding new test cases or modifying detection logic

## Mocking

**Framework:** None - tests use actual data structures

**Patterns:**
- Create minimal config structs: `Config::default()`, `AgentConfig { ... }`
- Create minimal data for parsing: String content, PaneInfo structs
- No dependency injection - tests construct objects directly

```rust
// File: src/app/config.rs, lines 1220-1227
#[test]
fn test_should_ignore_session_patterns() {
    let mut config = Config::default();
    config.ignore_self = false;
    config.ignore_sessions = vec![
        "prod-*".to_string(),
        "/^vpn-\\d+$/".to_string(),
        "ssh-tunnel".to_string(),
    ];
    // ... assertions follow
}
```

**What to Mock:**
- Command execution (when testing CLI handling)
- Tmux client responses (would need test tmux session)
- Time-based operations (use fake time, currently not done)

**What NOT to Mock:**
- Config parsing/serialization (test with real TOML)
- Regex parsing (test with real patterns and content)
- Path handling (test with real filesystem paths)
- String manipulation (test with real strings)

## Fixtures and Factories

**Test Data:**

Config factories are minimal - use `Config::default()` and modify fields:

```rust
// File: src/app/config.rs, lines 1157-1196
#[test]
fn test_apply_override() {
    let mut config = Config::default();

    // Test show_detached_sessions override
    config.apply_override("show_detached_sessions", "false").unwrap();
    assert!(!config.show_detached_sessions);

    // Test poll_interval override
    config.apply_override("poll_interval_ms", "1000").unwrap();
    assert_eq!(config.poll_interval_ms, 1000);

    // Test invalid key
    assert!(config.apply_override("invalid_key", "value").is_err());
}
```

**Fixture Location:**
- `tests/fixtures/{agent_id}/case_{status}_{description}.txt`
- Each file contains real captured terminal output
- Status prefix in filename determines expected behavior
- Organized by agent type: `claude/`, `shell/`, `pi/`, `pi-powerline/`, `gemini/`

**Adding New Fixtures:**
1. Capture pane output when desired status occurs: `tmux capture-pane -p -t session:pane > capture.txt`
2. Determine the status type: idle, working, approval, error
3. Move to appropriate directory with naming: `case_{status}_{description}.txt`
4. Run tests to verify: `cargo run -- test -d`

## Coverage

**Requirements:** No explicit coverage percentage enforced

**View Coverage:** Not configured - would use `tarpaulin` or `llvm-cov` if needed

**Practical Approach:**
- Critical path: config loading, parsing, status detection
- Test coverage high for: enum variants, config defaults, serialization
- Lower coverage for: UI rendering (hard to test in CLI), async task coordination
- Regression suite (fixtures) provides coverage of parser logic

## Test Types

**Unit Tests:**
- Scope: Individual functions and methods
- Approach: Direct function calls with test data
- Location: `#[cfg(test)] mod tests` in each module
- Examples: Config deserialization, key binding parsing, pattern matching
- File: `src/app/config.rs` has 16 unit tests for config operations

**Integration Tests:**
- Scope: Parser + Config + detection logic together
- Approach: Fixture-based regression testing
- Location: `tests/fixtures/` directories with `.txt` files
- Execution: `cargo run -- test` command
- Coverage: 85+ test cases across 5 agent types
- Current: Parser correctly identifies status from real terminal output

**E2E Tests:**
- Framework: Not used
- Reason: Would require running tmux session with agents
- Alternative: Manual testing with running Claude Code or other agents
- Recommendation: Set up CI tmux session if E2E testing becomes critical

## Common Patterns

**Async Testing:**

No async unit tests currently used. Async logic tested via:
- Integration tests with fixtures
- Manual testing with real tmux sessions
- Monitor task runs in background, state changes tested via UI updates

Async would be tested with `tokio::test` attribute:
```rust
#[tokio::test]
async fn test_async_function() {
    // test async code
}
```

**Error Testing:**

```rust
// File: src/app/config.rs, lines 1189-1195
#[test]
fn test_apply_override() {
    let mut config = Config::default();

    // Test invalid key
    assert!(config.apply_override("invalid_key", "value").is_err());

    // Test invalid value
    assert!(config
        .apply_override("show_detached_sessions", "invalid")
        .is_err());
}
```

**Match Testing (Status Detection):**

```rust
// File: src/cmd/test.rs, lines 147-148
let is_match = std::mem::discriminant(&actual_status)
    == std::mem::discriminant(&expected_status_enum);
```

Uses enum discriminant comparison to check status variant matches (ignores field values).

## Parser Testing Discipline

**Critical Rules:**
1. **Run regression suite after parser changes**: `cargo run -- test`
2. **100% pass rate required**: All 85+ fixture tests must pass before commit
3. **Debug mode for investigation**: `cargo run -- test -d` shows matching details
4. **Add fixtures for new agent types**: Each agent needs representative cases:
   - Idle state (no work pending)
   - Working state (processing)
   - Approval state (awaiting user action)
   - Error state (command failed)
   - Edge cases (special prompts, multi-line output, etc.)
5. **Fixture content is sacred**: Never manually modify fixture content - recapture from real session

## Test Execution Checklist

Before committing code changes:
```bash
# Run all unit tests
cargo test

# Run parser regression tests (must be 100% pass)
cargo run -- test

# Run linter (zero warnings)
cargo clippy

# Format code
cargo fmt

# Build in release mode
cargo build --release
```

All four steps must succeed without errors or warnings.

---

*Testing analysis: 2026-01-30*
