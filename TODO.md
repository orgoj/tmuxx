# TODO - tmuxcc


- [x] generic shell poznat ze bezi tuio aplikace a neni to procesing (mc, htop, vim)
- [x] oprava detekce stavu claude - implementován nativní regression testing a robustní regexy <!-- id: 100 -->

- Configurable different mode for TODO section of screen, want to see beginning of TODO.md file if in project or other configurable name, multiple names and glob - first found is displayed

- slected pane rychly filtr - defailt binding s - prepina mezi zobrazenim selected a normalnim

- jak poznat commandy co bezi v ssh ?

## Priority Tasks

### Notification System for Action-Required Events
**Status:** 💡 Feature Request
**Problem:** No alerts when agent needs user action → user must constantly watch tmuxcc
**Use case:** Agent awaits approval → terminal bell + desktop notification + custom command
**Solution:**
- Notification system with multiple channels (terminal, command, hooks)
- Only notify for actionable events (approval, error, question)
- Do NOT notify for informational events (subagent done, idle, processing)
- Configurable via TOML (enable/disable, channels, custom commands)

**Actions:**
- [ ] Design notification architecture (event detection, channel dispatch)
- [ ] Implement terminal notifications (visual bell/flash)
- [ ] Implement command execution (`notify-send`, `osascript`, custom)
- [ ] Implement hook system (callback scripts per event type)
- [ ] Add config options to Config struct and TOML
- [ ] Event filtering: only approval_needed, agent_error, user_question
- [ ] Test: agent approval → notification fires
- [ ] Test: subagent done → NO notification
- [ ] Document in README.md and config reference

**Config example:**
```toml
[notifications]
enabled = true
channels = ["terminal", "command"]
command = "notify-send 'tmuxcc' '{message}'"

[[notifications.hook]]
event = "approval_needed"
script = "/path/to/notify.sh"
```

### Enhanced Process Detection (Parent + Tree + Content)
**Status:** 💡 Feature Request
**Problem:** Current detection only checks process command → misses agents in wrappers/shells
**Use case:** Agent launched via wrapper script → current detection fails
**Solution:**
- Multi-strategy detection with fallback chain
- Detect parent process (agent wrapper)
- Scan process tree (entire hierarchy)
- Content-based AI type detection (parse output for Claude/Gemini/Codex patterns)

**Actions:**
- [x] Research: how to get parent PID and process tree on Linux/macOS
- [x] Implement parent process detection in PaneInfo
- [x] Implement process tree scanning (recursive parent/child)
- [x] Implement content-based AI type detection (regex patterns per AI)
- [x] Update ParserRegistry to use enhanced detection
- [x] Add detection strategy config (enable/disable strategies)
- [x] Recover all deleted hardcoded agent definitions into defaults.toml
- [x] Enhance UniversalParser to support subagents and approval types
- [x] Restore catch-all behavior to ensure no sessions are lost
- [x] Migrate user config to new format-based detection → correct AI type identified
- [ ] Document detection strategies in README.md

### AI-Specific Control Configuration
**Status:** 💡 Feature Request
**Problem:** All AI agents use same key bindings (Y/N) → not flexible for different AI types
**Use case:** Claude uses Y/N, Gemini uses A/R, custom AI uses different workflow
**Solution:**
- Define AI profiles in config with custom key bindings
- Per-AI approval workflows (single-key vs confirmation)
- Custom actions/commands per AI type

**Actions:**
- [ ] Design AI profile config schema (TOML format)
- [ ] Add `ai_profiles` to Config struct
- [ ] Implement AI profile matching (agent type → profile)
- [ ] Update key handling to use AI-specific bindings
- [ ] Support custom approval workflows per AI
- [ ] Add AI-specific action definitions
- [ ] Test: Claude agent → Y/N keys work
- [ ] Test: Gemini agent → A/R keys work (if configured)
- [ ] Document AI profiles in config reference

**Config example:**
```toml
[[ai_profile]]
name = "claude-code"
approval_keys = { yes = "y", no = "n" }
requires_confirmation = false

[[ai_profile]]
name = "gemini"
approval_keys = { approve = "a", reject = "r" }
requires_confirmation = true
```

### Configurable Action Menus per Session
**Status:** 💡 Feature Request - COMPLEX SYSTEM (See TODO-MENU.md)

**Problem:** No way to define custom actions/workflows for specific sessions

**Vision:** Powerful action system with variables, inputs, screen capture, editor, and bash pipelines

**Full specification:** See [TODO-MENU.md](TODO-MENU.md) for complete details including:
- Variable system (`${SESSION_DIR}`, `${TMP}`, etc.)
- Input mechanisms (`@{INPUT_LINE}`, `@{SCREEN}`, `@{EDITOR}`)
- Pipeline execution with bash support
- 5 implementation phases
- Config examples and technical challenges

### CLI --filter argument for session filtering - NOTE: toto je snad hotovo jen to nema --filter ale standardni --set ...
**Status:** 💡 Missing CLI option
**What works:** Runtime `/` filter, config `ignore_sessions`
**What's missing:** CLI `--filter` argument for startup filtering

**Actions:**
- [ ] Add `--filter <PATTERN>` argument to CLI (main.rs)
- [ ] Document in README.md and --help

### Focus key 'f' - Outside Tmux Support
**Status:** ⚠️ WORKAROUND IMPLEMENTED - Needs proper solution

**Current status:**
- ✅ Inside tmux, same session - works
- ✅ Inside tmux, cross-session - works (switch-client)
- ⚠️ Outside tmux - **temporary workaround** with wrapper script

**Temporary solution:** Wrapper script `scripts/tmuxcc-wrapper.sh`
- Automatically ensures tmuxcc ALWAYS runs inside tmux session `tmuxcc`
- If session doesn't exist, creates it
- If running inside tmux: switch-client to tmuxcc session
- If running outside tmux: attach to tmuxcc session
- Eliminates "outside tmux" problem but is not elegant

**Proper solution needed:**
- Detect terminal emulator (kitty, alacritty, etc.)
- Launch platform-specific command to open new terminal
- Attach to target tmux session in that new terminal
- Eliminate need for wrapper script

**Why wrapper is provisional:**
- User must remember to use `tcc` instead of `tmuxcc`
- Not intuitive for new users
- Better: `tmuxcc` detects outside-tmux and launches terminal automatically

**Files:**
- `scripts/tmuxcc-wrapper.sh` - temporary wrapper script
- `README.md` - documents workaround usage


---

## Other

- Config for left column width: character count or percentage

- Fix unnecessary | in session tree display - check tmux with project skill


- Does not detect query tool - shows idle



- collapse session? - needs select on session and session menu
- preview preserver importasnt lines, wrap, must scroll to end after wrap
- scroll in preview area?

---

## Notes
- Before implementation ALWAYS search for existing libraries via web search
- Use rtfmbro MCP for library documentation
- Don't write things from scratch when quality libraries exist
