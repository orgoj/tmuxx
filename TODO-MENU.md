# Tree Menu Commands Specification

**Goal:** Implement a fast, keyboard-centric hierarchical menu system for `tmuxcc` commands.

## Requirements

1.  **Hierarchical Structure (Tree)**
    - Defined in configuration (User `config.toml` and Project `tmuxcc.toml`).
    - Support for nested submenus.

2.  **Efficient Navigation**
    - **Arrow Keys:**
        - `Up`/`Down`: Move selection.
        - `Right`: Expand node or enters submenu.
        - `Left`: Collapse node or go back to parent.
    - **Expansion:**
        - `*` (Asterisk): Expand all nodes.
    - **Fuzzy Filtering:**
        - Typing immediately filters the visible tree nodes.
        - Matching logic: Fuzzy or subsequence match (case-insensitive).

3.  **Command Execution**
    - Reuse existing `ExecuteCommand` / `SendKeys` infrastructure.
    - No duplicated logic; menus are just another way to trigger defined actions.

4.  **Configuration & Merging**
    - **User Config**: Global menu definitions in `~/.config/tmuxcc/config.toml`.
    - **Project Config**: Session-specific menu definitions in `./tmuxcc.toml` (in the session root).
    - **Merging Strategy**:
        - Project configuration is loaded and **merged** into the menu structure.
        - Mechanism: "Temporary Merge" - effectively overlaying project items onto the menu tree for the duration of the session.

## Configuration Schema (Draft)

```toml
# config.toml concept

[[menu_items]]
name = "Git"
description = "Git operations"
items = [
    { name = "Status", command = "git status" },
    { name = "Log", command = "git log --oneline --graph" },
    { name = "Push", command = "git push" }
]

[[menu_items]]
name = "Project"
# Project specific items might be injected here or in their own section
```

## Implementation Plan

### Phase 1: Configuration Structs
- Define `MenuItem` and `MenuConfig` structs.
- Implement recursive deserialization.
- **Implement specific `Config::load_project_config()` logic** to merge local `tmuxcc.toml` menus.

### Phase 2: TUI Component (`menu_tree.rs`)
- Create `TreeMenuWidget`.
- Handle keyboard events (Arrows, Alpha-numeric for filter).
- Render hierarchical tree state.

### Phase 3: Integration
- Add new key binding (e.g., `Space` or `m`) to open the Menu Modal.
- Connect selection execution to `App::execute_command`.

## Open Questions
- **Library**: Does `ratatui` (or `tui-rs`) have a robust Tree widget with filtering?
    - *Research needed.* If not, `tui-tree-widget` crate might be useful, or custom impl.
- **Project Merge Details**: Should project items appear at the top level or under a "Project" node?
    - *Decision*: Allow flexible placement, but default to appending at root or merging by name.
