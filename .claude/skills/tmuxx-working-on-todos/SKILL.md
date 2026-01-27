---
name: tmuxx-working-on-todos
description: "Use this to assess, plan, and execute tasks from TODO.md. Coordinates the full development lifecycle from task selection to final commit."
---

# Working on Todos

You use this skill when asked to pick up the next task from the project's roadmap. It ensures a consistent approach to task completion.

## Steps

### 1. Analysis
- **CRITICAL**: Always read `TODO.md` from the beginning.
- If you are using this skill repeatedly in the same session, **READ THE FILE AGAIN** before starting a new task. The user may have updated the tasks or reordered them between your actions.
- Identify the first unchecked item.
- Analyze the complexity and determine the implementation strategy.

### 2. Execution

#### Path A: Small Task
- Describe the solution and identify the relevant skill (e.g., `tmuxx-adding-config-options`).
- Implement the fix/feature.

#### Path B: Complex Task
- Initiate a brainstorming session.
- Create an implementation plan using `tmuxx-planning`.
- Wait for user approval.

#### Path C: New Pattern
- If the task requires a new repeatable process, create a new skill first.

### 3. Verification
- Follow the Quality Assurance (QA) steps: `cargo fmt`, `cargo clippy`, and runtime testing.
- Mark the item as `[x]` in `TODO.md`.
- Use `tmuxx-managing-changelogs` to document the work.
- Use `tmuxx-committing-changes` to finalize the task.
