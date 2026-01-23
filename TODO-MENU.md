# TODO - Menu System

**Status:** 💡 Feature Request - COMPLEX SYSTEM
**Problem:** No way to define custom actions/workflows for specific sessions
**Use case:** Press 'm' → popup menu → select action → execute with variables/inputs/pipelines

**Vision:** Powerful action system with variables, inputs, screen capture, editor, and bash pipelines

---

## Core Features

### A. Session Selection (Currently Missing)
- **Problem:** Can only select pane, not entire session
- **Need:** Select entire session (jump between sessions)
- **Action:** Add session-level selection to UI

### B. Menu System
- **Trigger key:** Configurable (default: 'm')
- **UI:** Popup menu (center, dynamic size, configurable position)
- **Pattern matching:** Regex on session name (e.g., `^cc-.*`)
- **Display:** List of actions with key bindings and labels

### C. Variable System
Variables expanded before execution:
- `${SESSION_DIR}` - current working directory of tmux session
- `${SESSION_NAME}` - name of tmux session
- `${PANE_ID}` - tmux pane ID
- `${TMP}` - auto-generated temp file (cleaned up after)
- `${HOME}`, `${USER}` - standard env vars

### D. Input Mechanisms
- `@{INPUT_LINE:Label:default}` - single-line input with label and default value
  - Example: `@{INPUT_LINE:Dippy rule:allow }`
  - Opens modal input dialog, waits for user, returns value
- `@{SCREEN:N}` or `@{SCREEN:-N}` - capture last N lines from pane
  - Example: `@{SCREEN:-30}` captures last 30 lines
- `@{EDITOR}` or `@{EDITOR:file}` - open file in $EDITOR, wait for close, return content
  - Creates temp file if no file specified
  - Blocks until editor exits

---

## Action Types

### 1. Simple Command
Send-keys to session:
```toml
[[session_menu.action]]
key = "e"
label = "Edit dippy"
command = "edit ${SESSION_DIR}/.dippy"
```

### 2. Pipeline with Input
Prompt user, append to file:
```toml
[[session_menu.action]]
key = "a"
label = "Add dippy rule"
command = "cat @{INPUT_LINE:Dippy rule:allow } >> ${SESSION_DIR}/.dippy"
```

### 3. Complex Pipeline
Capture screen, edit, process with AI, paste result:
```toml
[[session_menu.action]]
key = "t"
label = "Translate screen to English"
command = "cat @{SCREEN:-30} > ${TMP} && editor ${TMP} && cat ${TMP} | claude -p 'Translate to English programmer text'"
paste_result = true  # Paste output to pane instead of executing
shell = "bash"  # Execute in bash (default if command has pipes/redirects)
```

---

## Execution Flow

1. **User presses 'm'** → popup menu appears
2. **User selects action** → action definition loaded
3. **Variable expansion:** `${SESSION_DIR}`, `${TMP}`, etc.
4. **Input resolution:** `@{INPUT_LINE}`, `@{SCREEN}`, `@{EDITOR}`
5. **Execute command:**
   - If `paste_result = false` (default): send-keys to tmux session
   - If `paste_result = true`: capture stdout, paste to tmux pane
6. **Cleanup:** Remove temp files (`${TMP}`)

---

## Config Example

```toml
[menu]
trigger_key = "m"
popup_position = "center"  # center, top, bottom
popup_width = 60  # characters
popup_height = 20  # lines

[[session_menu]]
pattern = "^cc-.*"  # Claude Code sessions
name = "Claude Code Actions"

[[session_menu.action]]
key = "e"
label = "Edit .dippy"
command = "edit ${SESSION_DIR}/.dippy"

[[session_menu.action]]
key = "a"
label = "Add dippy rule"
command = "cat @{INPUT_LINE:Dippy rule:allow } >> ${SESSION_DIR}/.dippy"

[[session_menu.action]]
key = "t"
label = "Translate screen"
command = "cat @{SCREEN:-30} > ${TMP} && editor ${TMP} && cat ${TMP} | claude -p 'Translate to English'"
paste_result = true
shell = "bash"

[[session_menu]]
pattern = "^ct-.*"  # Test sessions
name = "Test Actions"

[[session_menu.action]]
key = "r"
label = "Run tests"
command = "cargo test"
```

---

## Implementation Phases

### Phase 1: Basic Menu System
- [ ] Add session-level selection to UI (jump between sessions)
- [ ] Design menu popup component (ratatui)
- [ ] Config parsing for `[[session_menu]]`
- [ ] Pattern matching (regex on session name)
- [ ] Simple command execution (send-keys)
- [ ] Test: 'm' → popup → select action → send-keys

### Phase 2: Variable System
- [ ] Variable expansion engine (`${SESSION_DIR}`, `${TMP}`, etc.)
- [ ] Detect session CWD from tmux
- [ ] Auto-generate temp files (`${TMP}`)
- [ ] Cleanup temp files after action
- [ ] Test: action with `${SESSION_DIR}` works

### Phase 3: Input Mechanisms
- [ ] `@{INPUT_LINE:label:default}` - modal input dialog
- [ ] `@{SCREEN:N}` - capture last N lines from pane
- [ ] `@{EDITOR}` - open $EDITOR, wait, return content
- [ ] Test: each input mechanism independently

### Phase 4: Pipeline Execution
- [ ] Detect if command needs bash (pipes, redirects, &&, ||)
- [ ] Execute complex pipelines in bash subshell
- [ ] Capture stdout for `paste_result = true`
- [ ] Paste result to tmux pane
- [ ] Test: complex pipeline with editor + claude

### Phase 5: Advanced Features
- [ ] Multiple menu patterns per session
- [ ] Menu inheritance (global → session-specific)
- [ ] Conditional actions (only show if file exists, etc.)
- [ ] Action confirmation prompts
- [ ] Error handling and user feedback

---

## Technical Challenges

- **Async editor:** Need to block tmuxcc while editor is open
- **Pipeline stdout capture:** Must distinguish send-keys vs paste-result
- **Temp file cleanup:** Ensure cleanup even on errors
- **Variable security:** Prevent injection (escape shell special chars)
- **CWD detection:** tmux panes may have different CWD than session
- **Bash vs simple:** Auto-detect when bash is needed

---

## Future Enhancements

- `@{MULTILINE}` - multi-line input (textarea)
- `@{SELECT:option1,option2}` - dropdown selection
- `@{FILE}` - file picker
- `@{AI:prompt}` - call Claude/Gemini inline (integration with menu actions)
- **History:** Remember recent inputs per action
- **Macro recording:** Record actions, save as new action
- **Action chains:** Execute multiple actions in sequence
- **Conditional execution:** Only run action if condition met (file exists, variable set, etc.)

---

## Related Ideas

See IDEAS.md sections:
- "Konfigurovatelné Menu Akcí per Session" (lines 319-358)
- "Configurable Command Pipelines" (lines 90-117)
- "Menu System (Future)" (lines 129-137)
