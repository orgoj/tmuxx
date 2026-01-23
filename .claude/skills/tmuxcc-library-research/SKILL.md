---
name: tmuxcc-library-research
description: Library research workflow for tmuxcc - NEVER write functionality from scratch!
---

# Library Research Workflow

**CRITICAL: NEVER write functionality from scratch when libraries exist!**

## Before Implementing ANY Feature

**Follow this workflow:**

1. **WebSearch** for current libraries (use year 2026 in query)
   ```
   WebSearch: "rust ratatui [feature] library 2026"
   ```

2. **rtfmbro MCP** to get README/docs of selected library
   ```bash
   mcp__rtfmbro__get_readme package="owner/repo" version="*" ecosystem="gh"
   ```

3. **Study examples** from library repo
   - Look for examples directory
   - Read integration tests
   - Check documentation

4. **Only then implement** using the library

## Example: Modal Text Editor

```bash
# 1. Search for libraries
WebSearch: "rust ratatui text editor widget library 2026"

# 2. Get documentation
mcp__rtfmbro__get_readme package="rhysd/tui-textarea" version="*" ecosystem="gh"

# 3. Check examples in repo
# 4. Implement using library
```

## Ratatui Documentation

**ALWAYS consult ratatui documentation via rtfmbro MCP BEFORE implementing UI features!**

Project uses **ratatui 0.29** - complete documentation available via MCP:

```bash
# Get README with quickstart
mcp__rtfmbro__get_readme package="ratatui/ratatui" version="==0.29" ecosystem="gh"

# Get documentation tree
mcp__rtfmbro__get_documentation_tree package="ratatui/ratatui" version="==0.29" ecosystem="gh"

# Read specific docs
mcp__rtfmbro__read_files package="ratatui/ratatui" version="==0.29" ecosystem="gh" requests='[{"relative_path":"docs/widgets.md"}]'
```

## Selected Libraries for tmuxcc

- **Text editing:** tui-textarea (rhysd) - supports ratatui 0.29, has popup example

## Ratatui Popup Pattern (CRITICAL!)

**NEVER implement popups with manual Rect calculations! ALWAYS use Layout + Flex::Center!**

Official Ratatui popup example:
```rust
use ratatui::layout::{Constraint, Flex, Layout, Rect};

// Create centered popup - CORRECT WAY
fn centered_popup(area: Rect, percent_x: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

// Usage: 70% width, 12 lines fixed height
let popup_area = centered_popup(area, 70, 12);
```

**Why this matters:**
- Manual Rect calculations with percentage-based height are WRONG for popups
- Popups have FIXED content height (borders, text, input fields = specific line count)
- Flex::Center provides proper centering (Ratatui 0.29+ feature)
- Official examples use this pattern - RTFM first!

**Get official popup example:**
```bash
mcp__github__get_file_contents owner="ratatui" repo="ratatui" path="examples/popup.rs" ref="v0.29.0"
```

## Why This Matters

- Don't reinvent the wheel
- Libraries are tested and maintained
- Community-standard solutions
- Saves time and reduces bugs
- Professional code quality
