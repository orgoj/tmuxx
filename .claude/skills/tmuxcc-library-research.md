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

## Why This Matters

- Don't reinvent the wheel
- Libraries are tested and maintained
- Community-standard solutions
- Saves time and reduces bugs
- Professional code quality
