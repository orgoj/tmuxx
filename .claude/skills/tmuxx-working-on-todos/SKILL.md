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

**CRITICAL: You MUST invoke the specific skill for the type of task you are doing.**

#### Task Type: New Feature / Interactive Functionality
- **INVOKE SKILL**: `tmuxx-adding-new-features`
- Follow its steps for backend, config schema, UI, and docs.

#### Task Type: New Configuration Option
- **INVOKE SKILL**: `tmuxx-adding-config-options`
- Follow its steps for `config.rs`, `config_override.rs`, and CLI args.

#### Task Type: Complex / Unknown / Architecture Change
- **INVOKE SKILL**: `tmuxx-planning`
- Create a plan and wait for user approval before coding.

#### Task Type: Small Fix / Maintenance
- Describe the solution.
- Implement the fix.
- **INVOKE SKILL**: `tmuxx-testing` to verify.

#### Task Type: New Pattern
- If the task requires a new repeatable process, create a new skill first.

### 3. Verification
- Follow the Quality Assurance (QA) steps: `cargo fmt`, `cargo clippy`, and runtime testing.
- Mark the item as `[x]` in `TODO.md`.
- Use `tmuxx-managing-changelogs` to document the work.
- Use `tmuxx-committing-changes` to finalize the task.
