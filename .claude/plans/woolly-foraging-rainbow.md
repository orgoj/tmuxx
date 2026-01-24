# Modal Text Input Dialog Implementation Plan

## Overview

Replace the current bottom input buffer with a modal text input dialog using the `tui-textarea` library. This will provide better multi-line editing, undo/redo, search, and a proper editor interface.

**Library selected:** `tui-textarea` by rhysd
- Supports ratatui 0.29 ✅
- Includes popup example (popup_placeholder.rs)
- Features: multi-line, undo/redo, selection, search, Emacs shortcuts

## Integration Strategy (User Decision)

**Decision:** Implement both systems, keep existing functionality

**Requirements:**
1. **Keep existing `popup_input.rs`** - used for single-line input (filters, renaming, etc.)
2. **Add NEW modal textarea** - for multi-line input to agents via send-keys
3. **Default keybinding:** `Shift+I` (S-i) - bindable to any key via keybindings
4. **Keep bottom input box** - add config option to hide it by default
5. **New action:** `OpenInputModal` - distinct from existing popup actions

**Benefits:**
- Single-line popups remain lightweight and fast
- Multi-line textarea for complex agent input
- User controls which input method to use via config
- No breaking changes to existing workflow

## Implementation Approach

### Phase 1: Add Dependency

**File:** `Cargo.toml`

```toml
[dependencies]
tui-textarea = "0.7"  # Add to dependencies
```

**Verification:** `cargo build` succeeds

### Phase 2: Create Modal Textarea Component

**File:** `src/ui/components/modal_textarea.rs` (new)

Create a reusable modal textarea widget:

```rust
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use tui_textarea::{TextArea, Input, Key};

pub struct ModalTextareaState {
    pub textarea: TextArea<'static>,
    pub title: String,
    pub prompt: String,
    pub is_single_line: bool,  // If true, Enter submits, otherwise inserts newline
}

impl ModalTextareaState {
    pub fn new(title: String, prompt: String, initial: String, single_line: bool) -> Self {
        let mut textarea = if initial.is_empty() {
            TextArea::default()
        } else {
            TextArea::new(vec![initial])
        };

        // Configure styling
        textarea.set_style(Style::default().fg(Color::White));
        textarea.set_placeholder_style(Style::default().fg(Color::DarkGray));
        textarea.set_placeholder_text(&prompt);

        Self {
            textarea,
            title,
            prompt,
            is_single_line: single_line,
        }
    }

    pub fn handle_input(&mut self, input: Input) -> bool {
        // Returns true if should close (Esc pressed)
        match input {
            Input { key: Key::Esc, .. } => return true,
            Input { key: Key::Enter, .. } if self.is_single_line => {
                // Submit - caller will check textarea.lines()
                return true;
            }
            input => {
                self.textarea.input(input);
            }
        }
        false
    }

    pub fn get_text(&self) -> String {
        self.textarea.lines().join("\n")
    }
}

pub struct ModalTextareaWidget;

impl ModalTextareaWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &ModalTextareaState) {
        // Check minimum terminal size
        if area.width < 80 || area.height < 24 {
            return;
        }

        // Create centered popup (80% width, 60% height)
        let popup_area = Self::centered_popup(area, 80, 60);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Render main block with title
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(&state.title)
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .style(Style::default().fg(Color::Cyan));
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Create layout inside block
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Prompt line
                Constraint::Min(1),    // Textarea
                Constraint::Length(2), // Hints
            ])
            .split(inner);

        // Render prompt
        let prompt = Paragraph::new(state.prompt.as_str());
        frame.render_widget(prompt, chunks[0]);

        // Render textarea
        frame.render_widget(&state.textarea, chunks[1]);

        // Render hints
        let hints = vec![
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(if state.is_single_line { " Submit  " } else { " Insert newline  " }),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::raw(" Cancel"),
            ]),
            Line::from(vec![
                Span::styled("[Ctrl+U]", Style::default().fg(Color::Yellow)),
                Span::raw(" Undo  "),
                Span::styled("[Ctrl+R]", Style::default().fg(Color::Yellow)),
                Span::raw(" Redo  "),
                Span::styled("[Ctrl+S]", Style::default().fg(Color::Yellow)),
                Span::raw(" Search"),
            ]),
        ];
        let hints_paragraph = Paragraph::new(hints);
        frame.render_widget(hints_paragraph, chunks[2]);
    }

    fn centered_popup(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        // Use Flex::Center for proper centering (Ratatui 0.29)
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
```

### Phase 3: Add Action Variants

**File:** `src/app/actions.rs`

Add new actions for modal textarea:

```rust
/// Actions that can be performed in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // ... existing actions ...

    /// Show modal textarea dialog
    ShowModalTextarea {
        title: String,
        prompt: String,
        initial: String,
        single_line: bool,
    },
    /// Hide modal textarea without submitting
    HideModalTextarea,
    /// Submit modal textarea (returns text)
    ModalTextareaSubmit,
}
```

**Note:** We don't need `ModalTextareaInput` action because we'll handle keys directly in the action processing phase using `Into<Input>` trait.

### Phase 4: Update AppState

**File:** `src/app/state.rs`

Add modal textarea state:

```rust
use crate::ui::components::modal_textarea::ModalTextareaState;

pub struct AppState {
    // ... existing fields ...

    /// Modal textarea dialog state (None = not shown)
    pub modal_textarea: Option<ModalTextareaState>,
}
```

Update `AppState::new()`:

```rust
pub fn new(config: Config) -> Self {
    Self {
        // ... existing fields ...
        modal_textarea: None,
    }
}
```

### Phase 5: Update Event Handling

**File:** `src/ui/app.rs`

#### 5a. Import crossterm event conversion

Add at top of file:

```rust
use crossterm::event::KeyEvent;  // For converting to tui_textarea::Input
use tui_textarea::Input;  // For type conversion
```

#### 5b. Update key mapping

Modify `map_key_to_action()` function to handle modal textarea:

```rust
fn map_key_to_action(key: KeyEvent, state: &AppState) -> Action {
    use crossterm::event::KeyCode;

    // If modal textarea is active, only handle special keys
    if state.modal_textarea.is_some() {
        return match key.code {
            KeyCode::Enter => Action::ModalTextareaSubmit,
            KeyCode::Esc => Action::HideModalTextarea,
            // All other keys are handled directly in action processing
            // to use Into<Input> conversion
            _ => Action::None,
        };
    }

    // ... rest of existing key mapping ...
}
```

**Note:** We return `Action::None` for other keys because they'll be handled directly in the action processing loop where we have access to the raw `KeyEvent`.

#### 5c. Update action handling and key processing

**IMPORTANT:** When modal textarea is active, we need to pass ALL key events (not just mapped actions) to the textarea. Modify the main event loop:

```rust
// In the main event loop, around line 482-658
if let Event::Key(key) = event {
    // Special handling for modal textarea
    if state.modal_textarea.is_some() {
        use tui_textarea::Input;

        // Check for special keys first
        let action = map_key_to_action(key, &state);

        match action {
            Action::ModalTextareaSubmit => {
                if let Some(modal) = state.modal_textarea.take() {
                    let text = modal.get_text();
                    // Send text to selected agent
                    // Use correct agent access pattern
                    if let Some(agent) = state.agents.get_agent(state.selected_index) {
                        if let Err(e) = tmux_client.send_keys(&agent.target, &text) {
                            state.set_error(format!("Failed to send input: {}", e));
                        }
                    }
                }
            }
            Action::HideModalTextarea => {
                state.modal_textarea = None;
            }
            _ => {
                // Pass all other keys to textarea using Into<Input> trait
                if let Some(modal) = &mut state.modal_textarea {
                    let input: Input = key.into();
                    modal.handle_input(input);
                }
            }
        }
    } else {
        // Normal key handling when modal is not active
        let action = map_key_to_action(key, &state);
        // ... existing action handling ...
    }
}
```

**Key improvement:** Use `Into<Input>` trait which is implemented for `crossterm::event::KeyEvent` by tui-textarea. This handles all key mappings automatically.

**Also update the ShowModalTextarea action handler:**

```rust
Action::ShowModalTextarea { title, prompt, initial, single_line } => {
    use crate::ui::components::modal_textarea::ModalTextareaState;
    state.modal_textarea = Some(ModalTextareaState::new(title, prompt, initial, single_line));
}
```

### Phase 6: Update Rendering

**File:** `src/ui/app.rs`

Add modal textarea rendering in the main `draw()` function, after the popup rendering:

```rust
// Around line 154-157 (after popup rendering)
if let Some(modal_state) = &state.modal_textarea {
    use crate::ui::components::modal_textarea::ModalTextareaWidget;
    ModalTextareaWidget::render(frame, size, modal_state);
}
```

### Phase 7: Update Component Exports

**File:** `src/ui/components/mod.rs`

Add the new module:

```rust
pub mod modal_textarea;
```

### Phase 8: Add Keybinding and Config Option

#### 8a. Add Keybinding (Default: Shift+I)

**File:** `src/ui/app.rs` in `map_key_to_action()` function

Add the default keybinding for opening modal textarea:

```rust
// In map_key_to_action() - general keybindings (work from any panel)
KeyCode::Char('I') => {
    Action::ShowModalTextarea {
        title: "Multi-line Input".to_string(),
        prompt: "Enter message to agent (Enter to submit, Esc to cancel)".to_string(),
        initial: String::new(),
        single_line: false,
    }
}
```

**Note:** Shift+I is represented as uppercase 'I' in crossterm.

#### 8b. Add Config Option to Hide Bottom Input

**File:** `src/app/config.rs`

Add new config option:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields ...

    /// Hide bottom input buffer (use modal textarea instead)
    #[serde(default)]
    pub hide_bottom_input: bool,
}
```

Update `Config::default()`:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            hide_bottom_input: true,  // Hide by default as requested
        }
    }
}
```

#### 8c. Update Layout to Respect Config

**File:** `src/ui/layout.rs` or where bottom input height is calculated

Modify layout to skip bottom input when `config.hide_bottom_input` is true:

```rust
// When calculating right column layout
let right_chunks = if state.config.hide_bottom_input {
    // No bottom input - use full height
    Layout::vertical([
        Constraint::Min(0),  // Main content takes all space
    ])
    .split(right_column);
} else {
    // Existing layout with bottom input
    Layout::vertical([
        Constraint::Min(0),  // Main content
        Constraint::Length(3), // Bottom input
    ])
    .split(right_column);
};
```

**File:** `src/ui/app.rs` in render function

Skip rendering input widget when hidden:

```rust
// Only render input if not hidden
if !state.config.hide_bottom_input {
    InputWidget::render(frame, input_area, state);
}
```

## Key Implementation Details

### 1. **Proper Key Conversion**

The most complex part is converting `crossterm::event::KeyEvent` to `tui_textarea::Input`. Reference the popup_placeholder.rs example:

```rust
// From popup_placeholder.rs
match crossterm::event::read()?.into() {
    Input { key: Key::Esc, .. } => break,
    input => {
        textarea.input(input);
    }
}
```

The `Into<Input>` trait is implemented for `crossterm::event::KeyEvent`, so we can use:

```rust
use crossterm::event::KeyEvent;
let input: Input = key_event.into();
```

### 2. **Modal State Management**

The modal textarea should replace the bottom input buffer when active. When the modal is shown:
- Normal input is disabled
- Focus is entirely on the modal
- Escape closes without submitting
- Enter submits (if single_line mode)

### 3. **Multi-line vs Single-line Mode**

- **Single-line mode:** Enter submits, use Shift+Enter for newline
- **Multi-line mode:** Enter inserts newline, use Ctrl+Enter or specific key to submit

This is configurable based on user preference.

### 4. **Integration with Existing Popup System**

The existing `popup_input.rs` uses custom text handling. The new `modal_textarea.rs` should:
- Use the same modal overlay pattern (Clear widget, centered popup)
- Follow the same styling conventions (Cyan borders, rounded corners)
- Provide consistent keyboard shortcuts (Enter/Esc)

## Testing Strategy

1. **Build verification:** `cargo build --release`
2. **Clippy check:** `cargo clippy` (must pass)
3. **Format check:** `cargo fmt --check`

4. **Manual testing:**
   - Open modal with keybinding
   - Type text (single and multi-line)
   - Test cursor movement (arrow keys, home/end)
   - Test editing (backspace, delete, Ctrl+U undo, Ctrl+R redo)
   - Test submission (Enter)
   - Test cancellation (Esc)
   - Test with long text (scrolling)
   - Test Unicode characters (IME support)

5. **Integration testing:**
   - Verify text is sent to correct agent
   - Verify modal closes after submission
   - Verify error handling for send failures

## Files to Modify

1. `Cargo.toml` - Add tui-textarea dependency
2. `src/ui/components/modal_textarea.rs` - NEW FILE
3. `src/ui/components/mod.rs` - Export new component
4. `src/app/actions.rs` - Add modal textarea actions
5. `src/app/state.rs` - Add modal_textarea field
6. `src/ui/app.rs` - Update event handling and rendering
7. `src/app/config.rs` - Add hide_bottom_input option
8. `src/ui/layout.rs` - Update layout to respect hide_bottom_input
9. `README.md` - Document new keybinding and config option
10. `CHANGELOG.md` - Add feature entry
11. `src/ui/components/help.rs` - Update help screen with new modal keybinding

## Documentation Updates

### README.md

Add to "Key Bindings" section:

```
Shift+I    Open multi-line input modal (send to agent)
```

Add to "Configuration" section:

```toml
# Hide bottom input buffer (use modal textarea instead)
hide_bottom_input = true  # Default: true
```

### CHANGELOG.md

Add entry:

```markdown
## [Unreleased]

### Added
- Modal multi-line input dialog for agent communication (Shift+I)
- Rich text editing with undo/redo, search, and Emacs-style shortcuts
- `hide_bottom_input` config option to use modal textarea as primary input
- tui-textarea library integration for advanced text editing

### Changed
- Bottom input buffer now optional (hidden by default)
```

## Future Enhancements

After basic implementation:
1. Search integration (enable `search` feature)
2. Syntax highlighting for code snippets
3. Auto-completion based on agent context
4. Multiple history buffers (one per agent)
5. Configurable modal size and position

## References

- tui-textarea repo: https://github.com/rhysd/tui-textarea
- Popup example: `examples/popup_placeholder.rs`
- Ratatui popup pattern: https://github.com/ratatui/ratatui/blob/v0.29.0/examples/popup.rs
- Current popup implementation: `src/ui/components/popup_input.rs`
