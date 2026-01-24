# TODO - tmuxcc

- jak definovat Alt-T ? A-T = { send_keys = "S-Tab" }

- config pro sirku leveho sloupce: pocet znaku nebo procenta

- opravit zbytecne | v zobrazeni session tree - podivej se na tmux s pomoci projekt skill

- konfigurovatelny jiny rezim pro TODO cast obrazovky, chci tam videt zacatek souboru TODO.md pokud je v projektu nebo jiny konfigurovatelny nace, vice nazvu i glob prvni co najde tam zobrazi



- nedetekuje query tool - pise to idle

- todo detekce je spatna toto nepoznal
```
││✢ Processing anthropics/skills… (esc to interrupt · ctrl+t to hide tasks · 1m 17s · ↑ 414 tokens)                  │
││  ⎿  ◼ #1 Process anthropics/skills repository                                                                     │
││     ◻ #2 Process fcakyon/claude-codex-settings repository                                                         │
││     ◻ #3 Process nikiforovall.blog article                                                                        │
││     ◻ #4 Process wshobson GitHub profile                                                                          │
││     ◻ #5 Process wshobson/agents repository
```


## Priority Tasks

### 1. CLI --filter argument for session filtering
**Status:** 💡 Missing CLI option
**What works:** Runtime `/` filter, config `ignore_sessions`
**What's missing:** CLI `--filter` argument for startup filtering

**Actions:**
- [ ] Add `--filter <PATTERN>` argument to CLI (main.rs)
- [ ] Document in README.md and --help

### 2. Translate entire project to English
**Status:** 🌍 i18n - PRIORITY
**Problem:** Project contains Japanese and Czech text in code (help text, error messages, comments)
**Reason:** Project is a public fork - must be in English for wider audience
**Rule:** English EVERYWHERE except files explicitly marked for specific language

**What to translate:**
- [ ] CLI help text (main.rs) - Japanese texts
- [ ] Error messages (main.rs) - Japanese texts
- [ ] Code comments - any non-English comments
- [ ] Debug messages - any non-English debug output
- [ ] Variable names - must be English
- [ ] Function names - must be English
- [ ] TODO.md tasks - translate to English (this file)
- [ ] CHANGELOG.md - keep existing entries as-is, new entries in English

**Files to modify:**
- `src/main.rs` - CLI help, error messages (Japanese → English)
- `src/**/*.rs` - comments, strings, error messages
- `.dippy` - translate to English OR rename to `.dippy.cs`
- Check README.md for non-English content
- Check all documentation for Czech/Japanese text

**Exceptions (can contain other languages):**
- Files with language extension: `.cs` (Czech), `.ja` (Japanese)
  - Example: `.dippy.cs`, `notes.cs`
- `.claude/diary/` - session notes (user's existing - don't translate old, new in English)
- User files explicitly marked with language extension

**After translation test:**
- [ ] `cargo build --release` passes
- [ ] `./target/release/tmuxcc --help` shows English text only
- [ ] Error messages are in English
- [ ] Code comments are in English


### 3. Focus key 'f' - Outside Tmux Support
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


### 4. Modal input dialog with text editor
**Status:** 💡 Ready to implement - Library selected
**Actions:**
- [ ] Add tui-textarea to Cargo.toml
- [ ] Study popup_placeholder.rs example from library
- [ ] Implement modal popup dialog with TextArea
- [ ] Connect with event handling (Esc closes, Enter sends)
- [ ] Replace current input buffer with this solution
- [ ] Test: open popup, enter text, send

**Problem:** Current input buffer has bugs, we need modal dialog with quality editor
**Solution:** Use **tui-textarea** library (by rhysd)

**Selected library: tui-textarea**
- Repo: https://github.com/rhysd/tui-textarea
- Docs: https://docs.rs/tui-textarea
- Supports ratatui 0.29 ✅
- Has popup example! (examples/popup_placeholder.rs)
- Features: multi-line, undo/redo, selection, search, Emacs shortcuts

**Installation:**
```toml
tui-textarea = "*"
```

### 5. Statusline for session + move input to modal dialog
**Status:** 🎨 UI Enhancement
**Problem:** Input buffer takes space where statusline for session could be
**Solution:**
- Remove always-visible input buffer from layout
- Add statusline for selected session (status, context %, activity)
- Move input to modal dialog (see task #5)
**Actions:**
- [ ] Design layout: where statusline will be, what it shows
- [ ] Implement statusline for session (similar to header)
- [ ] Remove input buffer from main layout
- [ ] Connect with modal input dialog from task #4

### 6. Notification System for Action-Required Events
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

### 7. Enhanced Process Detection (Parent + Tree + Content)
**Status:** 💡 Feature Request
**Problem:** Current detection only checks process command → misses agents in wrappers/shells
**Use case:** Agent launched via wrapper script → current detection fails
**Solution:**
- Multi-strategy detection with fallback chain
- Detect parent process (agent wrapper)
- Scan process tree (entire hierarchy)
- Content-based AI type detection (parse output for Claude/Gemini/Codex patterns)

**Actions:**
- [ ] Research: how to get parent PID and process tree on Linux/macOS
- [ ] Implement parent process detection in PaneInfo
- [ ] Implement process tree scanning (recursive parent/child)
- [ ] Implement content-based AI type detection (regex patterns per AI)
- [ ] Update ParserRegistry to use enhanced detection
- [ ] Add detection strategy config (enable/disable strategies)
- [ ] Test: agent in wrapper → detected correctly
- [ ] Test: content-based detection → correct AI type identified
- [ ] Document detection strategies in README.md

### 8. AI-Specific Control Configuration
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

### 9. Configurable Action Menus per Session
**Status:** 💡 Feature Request - COMPLEX SYSTEM (See TODO-MENU.md)

**Problem:** No way to define custom actions/workflows for specific sessions

**Vision:** Powerful action system with variables, inputs, screen capture, editor, and bash pipelines

**Full specification:** See [TODO-MENU.md](TODO-MENU.md) for complete details including:
- Variable system (`${SESSION_DIR}`, `${TMP}`, etc.)
- Input mechanisms (`@{INPUT_LINE}`, `@{SCREEN}`, `@{EDITOR}`)
- Pipeline execution with bash support
- 5 implementation phases
- Config examples and technical challenges

---

## Other ideas

- colapse session? - potrebuje select na session a i session menu
- preview preserver importasnt lines, wrap, must scroll to end after wrap
- scroll in preview area?

---

## Notes
- Before implementation ALWAYS search for existing libraries via web search
- Use rtfmbro MCP for library documentation
- Don't write things from scratch when quality libraries exist
