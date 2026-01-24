# Isolated tmux Testing System for tmuxcc

## Overview

Design and implement a comprehensive isolated testing infrastructure using tmux's native socket isolation (`-L` flag). This system will provide complete separation from production tmux sessions, safety mechanisms, and support for both unit and integration tests.

## Current State Analysis

### Existing Test Infrastructure
- **Unit tests**: ~30 tests via `cargo test` (no tmux dependency)
- **Test scripts**: 3 bash scripts (reload-test.sh, start-test-session.sh, setup-multi-test.sh)
- **Test sessions**: ct-test (main), ct-multi (5 windows for multi-window testing)
- **Safety approach**: Manual discipline (one key at a time, verify before proceeding)

### Current Limitations
1. **No isolation**: Test sessions run on same tmux socket as production
2. **No rollback**: Can't restore state after failed tests
3. **No health checks**: No automatic validation before operations
4. **Manual verification**: No automated assertion framework
5. **No integration tests**: Only unit tests exist (no end-to-end TUI testing)
6. **State persistence**: Test data accumulates across runs

## Solution Design

### Core Isolation Strategy

**Use tmux `-L socket-name` for complete isolation:**

```bash
# Production socket: /tmp/tmux-UID/default
# Test socket:       /tmp/tmux-UID/tmuxcc-test

# All test operations use:
tmux -L tmuxcc-test <command>
```

**Benefits:**
- Complete isolation from production tmux
- Native tmux security (socket permissions)
- Automatic cleanup on reboot (in /tmp)
- Multiple test environments can coexist
- Easy teardown: `tmux -L tmuxcc-test kill-server`

### Architecture Components

#### 1. Socket Management
- Create/destroy isolated tmux server
- Verify isolation from production
- Health checks for socket state

#### 2. Session Lifecycle
- Setup test environments (minimal, standard, complex)
- State snapshots before risky operations
- Restore snapshots on failure
- Clean teardown

#### 3. Safety Mechanisms
- Pre-flight health checks before operations
- Continuous isolation verification
- Validation framework (structure, content, state)
- Error recovery with snapshots

#### 4. Integration Testing
- Rust integration tests (`tests/integration/`)
- Mock agent scripts for reproducible testing
- Cargo test workflow integration
- CI/CD support (GitHub Actions)

## Implementation Plan

### Phase 0: Socket Selection Support for tmuxcc

**PREREQUISITE**: tmuxcc currently does NOT support socket selection. All tmux commands use default socket.

**Add socket selection to TmuxClient** (src/tmux/client.rs):

1. **Add socket field to TmuxClient struct**:
   ```rust
   pub struct TmuxClient {
       capture_lines: u32,
       show_detached_sessions: bool,
       socket_name: Option<String>,  // NEW: Socket name for -L flag
   }
   ```

2. **Socket selection priority** (highest to lowest):
   - CLI flag: `--socket tmuxcc-test`
   - Environment variable: `TMUX_SOCKET=tmuxcc-test`
   - Config file: `socket_name = "tmuxcc-test"` in config.toml
   - Default: `None` (uses default tmux socket)

3. **Update all Command::new("tmux") calls**:
   ```rust
   fn build_tmux_command(&self) -> Command {
       let mut cmd = Command::new("tmux");
       if let Some(socket) = &self.socket_name {
           cmd.arg("-L").arg(socket);
       }
       cmd
   }
   ```

4. **Files to modify**:
   - `src/tmux/client.rs` - Add socket_name field, build_tmux_command() helper
   - `src/app/config.rs` - Add socket_name config option
   - `src/main.rs` - Add --socket CLI flag
   - `src/ui/app.rs` - Pass socket from config/CLI to TmuxClient

5. **Testing**:
   ```bash
   # Verify socket selection works
   tmux -L tmuxcc-test new-session -d -s test-session
   TMUX_SOCKET=tmuxcc-test ./target/debug/tmuxcc --debug
   # Should detect test-session, not production sessions
   ```

**Estimated effort**: 2-3 hours (straightforward addition)

**Nested tmux handling**:
- `--socket` works when already inside tmux (creates separate server)
- tmuxcc can monitor isolated test socket while running in production tmux
- Example: Run tmuxcc in production, test against `tmuxcc-test` socket
- No conflict: Different sockets = different servers

### Phase 1: Core Infrastructure

**Create scripts/test/core/ directory with:**

1. **test-env.sh** - Environment orchestration
   ```bash
   # Commands: setup, teardown, reset, force-cleanup
   # Creates isolated socket, verifies isolation
   # Sets up environment variables

   # CRITICAL: Unset TMUX to avoid nested tmux confusion
   unset TMUX

   # Export test socket name
   export TMUX_SOCKET=tmuxcc-test

   # Source safety wrapper to prevent bare tmux commands
   source "$(dirname "$0")/../lib/tmux-wrapper.sh"

   # Set explicit terminal size for consistent output
   export TMUX_TEST_TERMINAL_SIZE="120x40"  # width x height
   ```

2. **test-socket.sh** - Socket management
   ```bash
   # Commands: create, verify, cleanup
   # Manages -L tmuxcc-test socket
   # Validates isolation from production
   ```

3. **test-session.sh** - Session lifecycle
   ```bash
   # Commands: create, snapshot, restore, list
   # Manages test sessions within isolated socket
   # Snapshot scope (for MVP):
   #   - Pane content (last 500 lines via capture-pane)
   #   - Session/window/pane structure (list-panes output)
   #   - Running processes (ps output for verification)
   # Snapshots stored in: tmp/test-snapshots/
   # Snapshot format: YYYY-MM-DD_HH-MM-SS_description.snapshot
   ```

4. **test-health.sh** - Health checks
   ```bash
   # Commands: check, verify-isolation, validate
   # Pre-flight checks before operations
   # Continuous isolation monitoring
   ```

### Phase 2: Test Fixtures & Mock Agents

**Create scripts/test/fixtures/ directory:**

1. **minimal.sh** - Single session, 1 pane
   - Fast setup (~100ms)
   - Basic functionality testing

2. **standard.sh** - 1 session, 3 windows, 5 panes
   - Multiple mock agents
   - Typical use case testing
   - Default for integration tests

3. **complex.sh** - 3 sessions, 15 windows, 30 panes
   - Multi-session scenarios
   - Performance benchmarking
   - Subagent testing

**Create scripts/test/fixtures/agents/ directory:**

1. **mock-claude.sh** - Mock Claude Code agent
   - Simulates approval prompts
   - Subagent spawning patterns
   - Context remaining output

2. **mock-opencode.sh** - Mock OpenCode agent
3. **mock-codex.sh** - Mock Codex CLI agent
4. **mock-gemini.sh** - Mock Gemini CLI agent

### Phase 3: Assertion Library

**Create scripts/test/lib/ directory:**

0. **tmux-wrapper.sh** - Safe tmux command wrapper
   ```bash
   # Prevent accidental bare tmux commands in tests
   # Source this at the start of all test scripts

   tmux() {
       echo "ERROR: Use tmux_test() instead of bare tmux in test scripts" >&2
       echo "This prevents accidental commands to production socket" >&2
       exit 1
   }

   tmux_test() {
       command tmux -L tmuxcc-test "$@"
   }

   # Export for subshells
   export -f tmux tmux_test
   ```

1. **assertions.sh** - Test assertion functions
   ```bash
   # All assertions support retry with timeout
   # Defaults: timeout=5s, interval=0.5s
   assert_session_exists "session-name" [timeout]
   assert_window_count "session" 3 [timeout]
   assert_pane_contains "session:0.0" "pattern" [timeout]
   assert_agent_detected "pane" "AgentType" [timeout]
   assert_agent_status "pane" "AwaitingApproval" [timeout]

   # Example with retry:
   # assert_pane_contains "test:0.0" "Claude Code" 10  # Wait up to 10s
   ```

2. **test-utils.sh** - Common utilities + output formatting
   - Pane content capture helpers
   - Output formatting (colors, test result display)
   - Timing utilities (sleep_until_ready, wait_for_process)
   - Timeout constants:
     ```bash
     FIXTURE_SETUP_TIMEOUT=2      # Fixture creation max time
     ASSERTION_DEFAULT_TIMEOUT=5   # Default for assertions
     ASSERTION_RETRY_INTERVAL=0.5  # Sleep between retries
     HEALTH_CHECK_TIMEOUT=1        # Health check max time
     ```

### Phase 4: Rust Integration Tests

**Create tests/ directory structure:**

**IMPORTANT**: Rust test module organization
- Use `tests/common.rs` NOT `tests/common/mod.rs`
- Reason: cargo treats `tests/common/mod.rs` as a test file
- Integration tests import via `mod common;`

1. **tests/common.rs** - Shared test utilities
   ```rust
   use std::process::Command;
   use std::sync::Once;

   static INIT: Once = Once::new();

   /// RAII guard for test environment - ensures cleanup on panic
   pub struct TestEnvironment {
       _guard: (),
   }

   impl TestEnvironment {
       pub fn new() -> Self {
           INIT.call_once(|| {
               // Run: ./scripts/test/core/test-env.sh setup
               let status = Command::new("./scripts/test/core/test-env.sh")
                   .arg("setup")
                   .status()
                   .expect("Failed to setup test environment");

               assert!(status.success(), "Test environment setup failed");
           });

           Self { _guard: () }
       }
   }

   impl Drop for TestEnvironment {
       fn drop(&mut self) {
           // CRITICAL: Always cleanup even on panic/failure
           // This prevents test socket from staying alive
           let _ = Command::new("./scripts/test/core/test-env.sh")
               .arg("teardown")
               .status();
       }
   }

   pub fn test_socket() -> &'static str { "tmuxcc-test" }

   pub fn tmux_test_cmd() -> Command {
       let mut cmd = Command::new("tmux");
       cmd.arg("-L").arg(test_socket());
       cmd
   }

   pub fn tmuxcc_test_cmd() -> Command {
       let mut cmd = Command::new("./target/debug/tmuxcc");
       cmd.env("TMUX_SOCKET", test_socket());
       cmd
   }

   // Example usage in tests:
   // #[test]
   // fn test_agent_detection() {
   //     let _env = TestEnvironment::new();  // Auto-cleanup on drop
   //     // ... test code ...
   // }
   ```

2. **tests/integration/test_agent_detection.rs**
   - Test ClaudeCode parser detection
   - Test OpenCode parser detection
   - Test custom agent patterns

3. **tests/integration/test_approval_flow.rs**
   - Test approval prompt detection
   - Test approval forwarding (y/n keys)
   - Test multi-select approvals

4. **tests/integration/test_multi_session.rs**
   - Test multi-session monitoring
   - Test session filtering
   - Test window preview

### Phase 5: Migration & Documentation

1. **Update existing scripts** to optionally use isolated socket
   ```bash
   # start-test-session.sh gains --isolated flag
   # reload-test.sh can target isolated environment
   ```

2. **Create scripts/test/README.md** with:
   - Quick start guide
   - Test environment types
   - Developer workflow examples
   - Troubleshooting guide

3. **Update .claude/skills/tmuxcc-testing/SKILL.md**
   - Document new isolated testing system
   - Update safety rules for isolated environment
   - Add examples using new scripts

4. **Create CI/CD workflow** (.github/workflows/integration-tests.yml)
   - Install tmux (not in default Ubuntu runners)
   - Run unit tests
   - Setup isolated environment
   - Run integration tests
   - Upload test artifacts on failure

   ```yaml
   # Prevent parallel test runs from interfering
   concurrency:
     group: integration-tests-${{ github.ref }}
     cancel-in-progress: false  # Wait for previous test to finish

   jobs:
     integration-tests:
       runs-on: ubuntu-latest
       timeout-minutes: 15  # Prevent hung tests from blocking CI

       steps:
         - name: Install tmux
           run: sudo apt-get install -y tmux

         - name: Setup test environment
           run: ./scripts/test/core/test-env.sh setup

         - name: Run integration tests
           run: cargo test --test integration

         - name: Teardown (always runs)
           if: always()
           run: ./scripts/test/core/test-env.sh teardown

         - name: Upload test artifacts on failure
           if: failure()
           uses: actions/upload-artifact@v3
           with:
             name: test-snapshots
             path: tmp/test-snapshots/
   ```

## Critical Files

### New Files to Create

**Core Infrastructure:**
1. `scripts/test/core/test-env.sh` - Environment orchestration (setup, teardown, reset)
2. `scripts/test/core/test-socket.sh` - Socket management and isolation
3. `scripts/test/core/test-session.sh` - Session lifecycle and snapshots
4. `scripts/test/core/test-health.sh` - Health checks and validation

**Test Fixtures:**
5. `scripts/test/fixtures/minimal.sh` - Minimal test environment
6. `scripts/test/fixtures/standard.sh` - Standard test environment (PRIMARY)
7. `scripts/test/fixtures/complex.sh` - Complex test environment
8. `scripts/test/fixtures/agents/mock-claude.sh` - Mock Claude Code agent (playback mode)
9. `scripts/test/fixtures/agents/mock-opencode.sh` - Mock OpenCode agent (playback mode)
10. `scripts/test/fixtures/agent-output/` - Recorded agent outputs (captured from real sessions)

**Assertion Library:**
10. `scripts/test/lib/tmux-wrapper.sh` - Safe tmux wrapper (prevents bare tmux commands)
11. `scripts/test/lib/assertions.sh` - Test assertion framework (with retry/timeout)
12. `scripts/test/lib/test-utils.sh` - Common utilities + output formatting (combines helpers + colors)

**Rust Integration Tests:**
13. `tests/common.rs` - Shared test utilities with RAII cleanup (NOT tests/common/mod.rs!)
14. `tests/integration/test_agent_detection.rs` - Agent detection tests
15. `tests/integration/test_approval_flow.rs` - Approval workflow tests

**Test Data Directories:**
16. `tmp/test-snapshots/` - Session state snapshots (.gitignore)
17. `tmp/test-logs/` - Test execution logs (.gitignore)

**Documentation:**
18. `scripts/test/README.md` - Testing system guide
19. `.github/workflows/integration-tests.yml` - CI/CD workflow (includes tmux installation)

### Existing Files to Update

1. `src/tmux/client.rs` - Add socket_name field and build_tmux_command()
2. `src/app/config.rs` - Add socket_name config option
3. `src/main.rs` - Add --socket CLI flag
4. `src/ui/app.rs` - Pass socket from config/CLI to TmuxClient
5. `scripts/start-test-session.sh` - Add --isolated flag option
6. `scripts/reload-test.sh` - Add support for isolated socket
7. `.claude/skills/tmuxcc-testing/SKILL.md` - Document new system
8. `CLAUDE.md` - Add reference to new testing system
9. `README.md` - Update testing section
10. `.gitignore` - Add tmp/test-snapshots/ and tmp/test-logs/
11. `CONTRIBUTING.md` - Document test constraints and terminal size requirements
12. `.dippy` - Add test-specific allow rules (isolated socket + test scripts)

## Implementation Priority

### Must-Have (MVP)
1. **Phase 0: Socket support** - Add socket selection to tmuxcc
2. **Update .dippy** - Add test-specific allow rules (isolated socket)
3. **test-env.sh** - Core environment setup/teardown
4. **tmux-wrapper.sh** - Safe tmux wrapper (prevents bare commands)
5. **standard.sh** - Standard test fixture
6. **mock-claude.sh** - Basic mock agent (playback mode)
7. **tests/common.rs** - Rust test utilities with RAII cleanup
8. **test-utils.sh** - Common utilities + colors
9. **scripts/test/README.md** - Usage documentation

### Should-Have
1. **test-socket.sh** - Socket isolation management
2. **test-health.sh** - Health checks
3. **test-session.sh** - State snapshots
4. **assertions.sh** - Assertion framework with retry
5. **test_agent_detection.rs** - Integration tests
6. **CI/CD workflow** - Automated testing

### Nice-to-Have
1. **minimal.sh** & **complex.sh** - Additional fixtures
2. **All mock agents** - Complete agent coverage
3. **Advanced assertions** - Deep validation
4. **Performance benchmarks** - Load testing

## Verification Steps

### After Phase 1 (Core Infrastructure)
```bash
# 1. Setup isolated environment
./scripts/test/core/test-env.sh setup

# 2. Verify isolation
./scripts/test/core/test-socket.sh verify
# Should show: Test socket isolated, production socket unchanged

# 3. Verify no production impact
tmux list-sessions  # Production sessions unchanged
tmux -L tmuxcc-test list-sessions  # Empty or test sessions only

# 4. Teardown
./scripts/test/core/test-env.sh teardown
```

### After Phase 2 (Fixtures)
```bash
# 1. Create standard fixture
./scripts/test/fixtures/standard.sh

# 2. Verify structure
tmux -L tmuxcc-test list-sessions
tmux -L tmuxcc-test list-windows -t test-standard

# 3. Run tmuxcc against test environment
TMUX_SOCKET=tmuxcc-test ./target/release/tmuxcc

# 4. Verify agent detection in UI
# Should show mock agents in agent tree
```

### After Phase 4 (Integration Tests)
```bash
# 1. Build project
cargo build --release

# 2. Run unit tests (no tmux)
cargo test --lib

# 3. Run integration tests (isolated tmux)
cargo test --test integration

# 4. Verify all tests pass
# Expected: All integration tests use isolated socket
```

### Final Verification
```bash
# Run complete test suite
./scripts/test/run-all-tests.sh

# Expected output:
# ✓ Unit tests: 30 passed
# ✓ Integration tests: 15 passed
# ✓ Isolation verified
# ✓ No production impact
# ✓ Cleanup successful
```

## Key Design Decisions

### Why `-L socket-name` over `-S socket-path`?
- Simpler: No need to manage full paths
- Standard location: `/tmp/tmux-UID/` is tmux default
- Automatic cleanup: /tmp cleaned on reboot
- Security: tmux enforces socket permissions

### Why bash scripts + Rust tests?
- Bash: Environment setup, tmux operations, fixtures
- Rust: Integration testing with cargo test workflow
- Best of both: Leverage strengths of each tool

### Why state snapshots?
- Debugging: Restore state to investigate failures
- Safety: Rollback on errors
- Reproducibility: Exact state capture for analysis

### Why mock agents?
- Deterministic: Same output every time
- Fast: No real agent startup time
- Controllable: Trigger specific scenarios
- No dependencies: Works offline

### Why single test run only (no parallel)?
- Simpler: Fixed socket name `tmuxcc-test`
- Realistic: Developer watches tests, needs to see UI
- Safe: No complex socket naming with PIDs
- Limitation: `cargo test` fails if socket busy (document this)

## Success Criteria

- [ ] Can run `cargo test` with full isolation (no production impact)
- [ ] Integration tests cover agent detection, approval flow, multi-session
- [ ] Test environment setup completes in <5 seconds
- [ ] Failed tests don't corrupt environment (snapshots work)
- [ ] CI/CD pipeline runs all tests automatically
- [ ] Documentation enables new developer to test in <5 minutes
- [ ] Zero risk of sending keys to production sessions
- [ ] Complete isolation verified by health checks

## Resources & References

**tmux Socket Isolation:**
- [tmux Advanced Use](https://github.com/tmux/tmux/wiki/Advanced-Use/62b1f350d060f88a7de79b95a6af122642cf765f) - Official tmux wiki on socket management
- [tmux man page](https://man7.org/linux/man-pages/man1/tmux.1.html) - `-L` and `-S` options documentation

**Testing Frameworks:**
- [tmux-test](https://github.com/tmux-plugins/tmux-test) - Isolated testing framework for tmux plugins
- [tmux-test README](https://github.com/tmux-plugins/tmux-test/blob/master/README.md) - Setup and usage examples

**Security Best Practices:**
- tmux creates sockets with permissions preventing access by other users
- Never chmod 777 the socket - use group ownership for minimal sharing
- Test socket isolated in `/tmp/tmux-UID/tmuxcc-test` with user-only access

## Implementation Notes

### Socket Selection Implementation Details

**Priority chain** (highest to lowest):
1. CLI: `tmuxcc --socket tmuxcc-test`
2. Env: `TMUX_SOCKET=tmuxcc-test tmuxcc`
3. Config: `socket_name = "tmuxcc-test"` in config.toml
4. Default: `None` (standard tmux socket)

**Code pattern**:
```rust
// In main.rs or config loading
let socket_name = cli_args.socket
    .or_else(|| env::var("TMUX_SOCKET").ok())
    .or(config.socket_name);
```

### Mock Agent Output Format - Recording/Playback Approach

**Recording real agent sessions**:
```bash
# Capture real Claude Code session to file
tmux capture-pane -t cc-session:0.0 -p -S -500 > \
  scripts/test/fixtures/agent-output/claude-code-approval.txt

# Capture subagent output
tmux capture-pane -t cc-session:0.0 -p -S -500 > \
  scripts/test/fixtures/agent-output/claude-code-subagent.txt
```

**Playback in mock agents**:
```bash
#!/bin/bash
# mock-claude.sh - Plays back recorded output
cat "$(dirname "$0")/agent-output/claude-code-approval.txt"
sleep infinity  # Keep process alive for detection
```

**Benefits**:
- **Exact match**: Output is captured from real agents
- **Easy updates**: Re-record when agent output changes
- **No manual sync**: No risk of mock drift from reality
- **Timing**: Add `sleep` between outputs to simulate delays if needed

**Maintenance workflow**:
1. When real agent output changes, re-record affected sessions
2. Commit new recordings to git
3. Mock agents automatically use new output
4. No code changes needed in mock scripts

### Test Data Directory Structure

```
tmp/
├── test-snapshots/          # Session state snapshots
│   ├── 2026-01-24_14-30-00_before-send-keys.snapshot
│   └── 2026-01-24_14-35-12_after-approval.snapshot
├── test-logs/               # Test execution logs
│   ├── test-env.log
│   ├── test-integration.log
│   └── mock-claude.log
└── test-fixtures/           # Captured agent outputs
    └── agent-output/
        ├── claude-code-approval.txt
        ├── claude-code-subagent.txt
        └── opencode-processing.txt
```

### Test Execution Constraints

**IMPORTANT**: Only one test run allowed at a time
- Socket name is fixed: `tmuxcc-test`
- Running multiple `cargo test` simultaneously will fail
- Error message: "Socket tmuxcc-test is already in use"
- Workaround: Wait for test to complete, or manually kill socket

**CI/CD handling**:
- GitHub Actions: Uses `concurrency` group to serialize test runs
- Multiple PRs: Tests queue, no parallel execution
- Timeout: 15 minutes to prevent hung tests from blocking CI

**Document in CONTRIBUTING.md**:
```markdown
## Testing Constraints

- Only one test run at a time (isolated tmux socket limitation)
- If tests fail with "socket busy", wait or run: `tmux -L tmuxcc-test kill-server`
- Integration tests require tmux installed
- Terminal size: Tests assume 120x40 for consistent output
```

### Timing and Race Conditions

**Common race conditions**:
1. Fixture creation → tmuxcc detection lag
2. send-keys → agent response delay
3. Subagent spawn → parser detection lag

**Solutions**:
- All assertions retry with timeout (default 5s)
- Fixtures sleep briefly after setup (FIXTURE_SETUP_TIMEOUT=2s)
- Mock agents output immediately (no startup delay)
- Use `assert_pane_contains` with timeout, not immediate grep

**Example**:
```bash
# BAD: Immediate check, may fail due to race
./scripts/test/fixtures/standard.sh
tmux -L tmuxcc-test capture-pane -t test-standard:0.0 -p | grep "Claude Code"

# GOOD: Retry with timeout
./scripts/test/fixtures/standard.sh
assert_pane_contains "test-standard:0.0" "Claude Code" 10  # Wait up to 10s
```

### Terminal Size Handling

**Problem**: tmux pane dimensions affect output (line wrapping, truncation)

**Solution**:
- Set explicit terminal size in test environment: `120x40` (width x height)
- Create test sessions with fixed dimensions:
  ```bash
  tmux_test new-session -d -x 120 -y 40 -s test-standard
  ```
- All fixtures use same size for consistency
- Documented in test-utils.sh as `TMUX_TEST_TERMINAL_SIZE`

### Safety Mechanisms Summary

**1. Bare tmux Command Protection**
- `tmux-wrapper.sh` prevents accidental bare `tmux` in test scripts
- Forces use of `tmux_test()` wrapper with `-L tmuxcc-test`

**2. TMUX Environment Variable**
- Unset in test environment to avoid nested tmux confusion
- Tests run as if outside tmux (clean environment)

**3. Cleanup on Failure**
- Rust `Drop` impl ensures teardown even on panic
- CI workflow uses `if: always()` for teardown step

**4. CI Concurrency Control**
- GitHub Actions concurrency group prevents parallel runs
- Tests queue instead of failing with "socket busy"

### Backward Compatibility

- Existing test scripts (reload-test.sh, etc.) remain functional - gradual migration
- No breaking changes to current development workflow
- Isolated testing is opt-in via environment variable or flag
- Production tmux sessions completely unaffected by tests
- Mock agents use recorded output for deterministic testing

## .dippy Permission System Integration

**Current .dippy protections**:
```bash
# Read-only tmux commands allowed
allow tmux list-sessions *
allow tmux capture-pane *

# Write allowed ONLY to ct-test session
allow tmux send-keys -t ct-test *

# Deny dangerous operations
deny scripts/cp-bin.sh *
```

**Add test-specific rules** (.dippy updates):
```bash
# Test infrastructure scripts (safe - use isolated socket)
allow ./scripts/test/**

# Isolated test socket (complete isolation from production)
allow tmux -L tmuxcc-test *

# Test environment setup/teardown
allow ./scripts/test/core/test-env.sh *
allow ./scripts/test/core/test-socket.sh *
allow ./scripts/test/core/test-health.sh *

# Test fixtures (create test sessions on isolated socket)
allow ./scripts/test/fixtures/**

# Allow test output redirection
allow-redirect ./tmp/test-snapshots/*
allow-redirect ./tmp/test-logs/*
```

**Why this is safe**:
1. **`-L tmuxcc-test`** creates separate tmux server (zero production impact)
2. **Scripts in `scripts/test/`** only use test socket via tmux-wrapper.sh
3. **Existing protection** for ct-test remains (production test session)
4. **Claude can't accidentally** run bare `tmux` on production socket

**Benefits**:
- Claude can run integration tests safely
- No risk of send-keys to production sessions
- Test infrastructure changes allowed without manual approval
- .dippy enforces socket isolation at permission level

## Clarification Questions - Answered

**Q1: Should `tmuxcc --socket` work when already inside a tmux session?**
A: Yes. Different sockets create separate tmux servers. tmuxcc can run in production tmux while monitoring test socket.

**Q2: What's the expected test suite runtime?**
A: MVP estimate:
- Unit tests: ~5s (cargo test --lib)
- Integration tests: ~30s (fixture setup + tmuxcc detection + teardown)
- Total: ~35-40s
- CI timeout: 15 minutes (safety margin)

**Q3: Will mock agents simulate timing or output instantly?**
A: **Instant output** for MVP. Mock scripts `cat` recorded output immediately. This is acceptable because:
- Assertions retry with timeout (handles any real delays)
- Faster test execution
- If timing bugs found, can add `sleep` between outputs in mock scripts
- Race conditions tested via assertion timeouts, not mock delays
