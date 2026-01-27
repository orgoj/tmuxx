---
name: work-on-todo
description: Standard workflow for processing tasks from TODO.md (Analyze -> Plan -> Execute -> Verify -> Commit)
---

# Work on TODO Item

Follow this process when asked to "work on the next todo" or similar.

## 1. Analysis & Selection
1. Read `TODO.md` (or project tracking file) to identify the first unchecked item.
   - Include nested sub-items if present.
2. Analyze the intent. What is the user really asking for?
3. Determine the complexity.

## 2. Strategy Phase
**Choose one path:**

### Path A: Clear / Small Task
- Briefly describe the solution.
- Identify the appropriate project-specific skill for implementation:
  - Adding a feature? -> Look for `*-add-feature` or similar.
  - Config change? -> Look for `*-config`.
  - Bug fix? -> Ad-hoc fix + verification.
- **Proceed to Execution.**

### Path B: Complex / Ambiguous Task
- **STOP**. Do not write code yet.
- Initiate a **Brainstorming Session** with the user.
- Visualize the problem and propose potential solutions.
- Wait for user approval on the plan.
- **Proceed to Execution.**

### Path C: New Workflow Pattern
- If the task implies a repeatable process not covered by existing skills.
- Propose creating a new **Skill** first.
- Once the skill is created, use it to solve the task.

## 3. Execution Phase
- Implement the changes following the selected strategy and project architecture.
- Follow the project's "Doer -> Interface -> Logic -> UI" flow if applicable.

## 4. Quality Assurance (QA)
Before calling it done, you MUST verify quality using project tools:
1. **Format**: Run formatter (e.g., `cargo fmt`, `npm run format`, `black`).
2. **Lint**: Run linter (e.g., `cargo clippy`, `eslint`). **Fix ALL warnings.**
3. **Test**:
   - Unit tests: Run standard test suite.
   - Regression/Integration tests: Verify against test cases.

## 5. Finalization
1. Mark the item as `[x]` in `TODO.md`.
2. **Documentation**: Ensure CHANGELOG and README are updated (if the project tracks them).
3. **Commit**: Run the project's commit skill (e.g., `tmuxx-commit`, `git-commit`) to handle the submission.
