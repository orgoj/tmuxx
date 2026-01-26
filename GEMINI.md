# Gemini Adapter & Memory

This project uses **CLAUDE.md** as the canonical source of truth. Always refer to it for architecture and standards.

## Critical Project Memories (Lessons Learned)

### 1. Agent Detection (Strong vs Weak)
- **EXE-based detection is Mandatory**: Stale window titles (`pane_title`) persist in tmux. Detect agents ONLY via the process tree (Strong match).
- **MatchStrength**: Prefer `Strong` (Process/Tree) over `Weak` (Title). This prevents Shells from being misdetected as stale agents.
- **Process Depth**: Scanning depth of 4 is required to catch agents behind wrappers (`sudo`, `node`, `bash`).

### 2. State Synthesis (Refinements)
- **Structural Blocks**: Use `refinements` in `StateRule` to parse the structure of the UI (e.g., prompt between horizontal lines).
- **Contextual Indicators**: Determine state (Baked/Idle vs Working) from the specific context (line above prompt) to avoid confusion from history.

### 3. UI Consistency
- **Idle = Prompt**: For shell/generic sessions, use `default_status = "processing"`. Transition to `Idle` ONLY when a prompt (`$#%>❯`) is detected at the very end.
- **Selection Visibility**: Use a combination of light background (Rgb 230, 230, 230) and cursor symbols (`▶`) for high visibility across themes.

### 4. Configuration-Driven Architecture (CRITICAL)
- **NO HARDCODED AGENTS**: The code must NEVER contain hardcoded logic for specific agents (e.g., "Claude", "Gemini").
- **Universal Logic**: All behavior must be derived solely from `defaults.toml` / `config.toml`.
- **Generic Handling**: Use generic systems that interpret the configuration at runtime. Avoid `match agent_id` or similar patterns.
- **Agent Names**: Always use the `name` field from the configuration for display, never hardcoded enum values.

### 5. Core Purpose
- **Status Visibility**: The main purpose of this tool is to visualize the status (Idle/Working/Error) of all tmux panes at a glance. Visuals should prioritize status clarity.

## Testing Checklist
1. **tmux capture-pane -pt <target>**: ALWAYS read the raw buffer before diagnosing. 
2. **Priority Check**: Ensure `generic_shell` has priority 10+ if users have priority 1 catch-alls.
3. **Rebuild Release**: Changes to `defaults.toml` require `cargo build --release` because it is embedded via `include_str!`.
