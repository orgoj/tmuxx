# TODO - tmuxcc


## Priority Tasks

### 1. Session filtering - show only specific sessions
**Status:** 💡 Feature Request - PRIORITY
**Problem:** tmuxcc shows ALL sessions → difficult testing (see production sessions during testing)
**Use case:** `tmuxcc --filter test` → shows only ct-test, cc-test, etc.
**Solution:**
- Add `--filter <PATTERN>` CLI argument
- Add `session_filter` to Config (regex or glob pattern)
- Filter sessions in TmuxClient.list_panes() or MonitorTask
- If filter not set → show all (current behavior)

**Actions:**
- [ ] Add `--filter` argument to CLI (main.rs)
- [ ] Add `session_filter: Option<String>` to Config
- [ ] Implement filtering in MonitorTask or TmuxClient
- [ ] Test: `./tmuxcc --filter test` → see only test sessions
- [ ] Test: `./tmuxcc` → see all sessions (default)
- [ ] Document in README.md and --help
- [ ] Change test scripts/* to auto setup test filter

**Example usage:**
```bash
# Show only test sessions
./tmuxcc --filter test

# Show only cc-* sessions
./tmuxcc --filter "^cc-"

# Show all (default)
./tmuxcc
```

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
**Status:** ✅ SOLVED WITH SIMPLER APPROACH (2026-01-23)

**What works:**
- ✅ Inside tmux, same session - works
- ✅ Inside tmux, cross-session - works (switch-client)
- ✅ Outside tmux - solved with **wrapper script** (simpler than terminal launcher)

**Solution:** Wrapper script `scripts/tmuxcc-wrapper.sh`
- Automatically ensures tmuxcc ALWAYS runs inside tmux session `tmuxcc`
- If session doesn't exist, creates it
- If running inside tmux: switch-client to tmuxcc session
- If running outside tmux: attach to tmuxcc session
- Eliminates "outside tmux" problem completely

**Usage:**
```bash
# Symlink to ~/bin
ln -sf $(pwd)/scripts/tmuxcc-wrapper.sh ~/bin/tcc

# Run wrapper instead of direct tmuxcc
tcc
```

**Note:** Original plan (Step 6) with platform-specific terminal launcher is UNNECESSARY.
Wrapper script is simpler, more reliable, and cross-platform.

**Files:**
- `scripts/tmuxcc-wrapper.sh` - wrapper script
- `README.md` - usage documentation


### 4. Preview pane shows end incorrectly - missing Claude prompts
**Status:** ✅ IMPLEMENTED - Waiting for runtime test (2026-01-23)

**Problem:** Session preview doesn't show end of pane content → approval prompts/menus not visible
**Root cause:** Text wrapping causes long lines to consume multiple display rows → bottom content is off-screen

**Solution implemented:**
- ✅ Smart line truncation instead of wrapping
- ✅ Truncate to terminal width with Unicode-safe logic
- ✅ Preserve important markers ([y/n], approve, reject) - never truncated
- ✅ Configurable: `truncate_long_lines` (default: true), `max_line_width` (default: terminal width)
- ✅ Config override: `--set truncate:false` for backward compatibility
- ✅ Increased capture_lines: 100 → 200 for better coverage
- ✅ Build passes, clippy clean, formatted

**Waiting for test:**
- [ ] Runtime verification: navigate to agent with long lines
- [ ] Verify: approval prompts visible at bottom of preview
- [ ] Verify: truncation indicator "…" on long lines
- [ ] Test: `--set truncate:false` restores wrapping behavior


### 5. Modal input dialog with text editor
**Status:** ✅ Library selected - Ready to implement
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

### 6. Statusline for session + move input to modal dialog
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
- [ ] Connect with modal input dialog from task #5

---

## Notes
- Before implementation ALWAYS search for existing libraries via web search
- Use rtfmbro MCP for library documentation
- Don't write things from scratch when quality libraries exist
