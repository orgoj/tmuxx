---
name: tmuxx-managing-changelogs
description: "Use this to audit, update, and organize the project's CHANGELOG.md and TODO.md files. Ensures completed tasks are moved from the roadmap to history."
---

# Managing Changelogs and TODOs

You use this skill to keep the project's documentation and roadmap clean. It defines the strict relationship between `TODO.md` (active work) and `CHANGELOG.md` (history).

## Steps

### 1. Analysis
- Identify tasks in `TODO.md` that have been implemented.
- Check if they are already described in `CHANGELOG.md` under `[Unreleased]`.

### 2. Execution

#### A. Update CHANGELOG.md
- Move the description of completed work to `CHANGELOG.md`.
- Follow the [Keep a Changelog](https://keepachangelog.com/) format (Added, Fixed, Changed, Removed).
- Be descriptive (e.g., mention specific config options or UI changes).

#### B. Cleanup TODO.md
- Delete completed tasks from `TODO.md`. Do not leave them there marked as checked.
- Ensure `TODO.md` only contains current or upcoming work.

### 3. Verification
- Verify that `CHANGELOG.md` correctly reflects all recent changes.
- Ensure `TODO.md` is focused and doesn't contain "ghost" tasks that are already done.
