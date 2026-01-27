---
name: tmuxx-creating-agent-definition
description: "Use this to assess, create, and review AI agent state rules in defaults.toml. Covers structural splitting (Prompt Sandwich), location-based refinements (LastLine, LastBlock), and robust anchor-based matching to prevent history noise."
---

# Creating Tmuxx Agent Definitions (Structural Splitter Model)

Use this skill when adding or updating AI agent monitoring rules in `src/config/defaults.toml`. It ensures robust detection by identifying UI structures instead of brittle line counts.

## Key Principles (Admin Notes)

### 1. The Prompt Sandwich
Most modern AI agents (Claude, Pi) use a "sandwich" at the bottom of the screen:
```
─────────────────  <-- Top Separator
❯ prompt / menu    <-- Interactive area
─────────────────  <-- Bottom Separator
Model info / stats <-- Footer
```
The **Splitter** (`separator_line` or `powerline_box`) identifies this sandwich from the bottom up and separates the output into two groups:
- `body`: Everything ABOVE the prompt sandwich (the actual agent response).
- `prompt`: The interactive area and footer (the UI chrome).

### 2. Refinement Match Locations
Instead of matching anywhere in the text, use precise locations to avoid false positives from history:
- `Anywhere`: Default.
- `LastLine`: The very last non-empty line of the group (perfect for Pi's "Working..." or "?").
- `LastBlock`: The last paragraph of text (separated by double newlines).
- `FirstLineOfLastBlock`: The first line of the most recent block (crucial for Claude, where spinners appear at the top of the task list).

### 3. Prompt-Anchored Matching
If the splitter group still contains noise, use regex anchoring to ensure the marker (like a question mark) is at the very end of the output, immediately followed by optional prompt lines.

**Example for Pi:**
```toml
# Matches '?' only if it's the last non-whitespace before the prompt lines
pattern = '''(?s).*\?\s*(?:(?m)^(?:╭─|╰─|↑|─|❯).*\s*)*\z'''
```

## Workflow

### 1. Analysis: Identify UI Structure
- Identify separators (`───`, `╭─`).
- Identify where spinners (`✻`, `⠋`) and questions (`?`) appear.
- Check if "Idle" state has a specific marker (like a naked `❯ ` prompt).

### 2. Implementation: Splitter First
Always use a structural splitter to isolate the response from the UI chrome.
```toml
[[agents.state_rules]]
status = "idle"
type = "idle"
splitter = "separator_line"  # or "powerline_box"
last_lines = 40              # Search window for the sandwich
```

### 3. Execution: Refinement Ordering
Refinements are evaluated in the order they appear. The first match wins.

#### Order of Priority:
1. **Working** (Spinners): Always first.
2. **Critical Approval Markers**: (e.g., `◻` unfinished tasks in Claude).
3. **Specific Approvals**: (Menus, Questions).
4. **Idle Overrides**: (e.g., Naked prompt `❯ `).
5. **General Fallbacks**.

#### Examples:
```toml
# 1. WORKING (High Priority)
[[agents.state_rules.refinements]]
group = "body"
location = "first_line_of_last_block"
pattern = '''[✽✻✢*∴⏺●·]\s*\w+…'''
status = "working"
type = "working"

# 2. IDLE OVERRIDE (For active prompts with no output)
[[agents.state_rules.refinements]]
group = "prompt"
pattern = '''(?m)^❯\s*$'''
status = "idle"
type = "idle"
```

### 4. Verification
- Run `tmuxx test` to verify all fixtures.
- Use `tmuxx test -d` to see how the splitter divided the screen and which refinement matched.
- **False Positives**: if "Idle" is detected as "Approval", anchor the pattern to the end (`\z`) or use a more specific `location`.

## Guidelines
1. **Prioritize Working**: Spinners should always pre-empt questions.
2. **Anchor to the End**: Use `\z` for status markers that only matter when they are the final output.
3. **Ignore History**: The splitter and precise locations are your best defense against "stale" matches from terminal history.
4. **No Brittle Counts**: Never rely on a fixed number of lines; always look for structural separators.
