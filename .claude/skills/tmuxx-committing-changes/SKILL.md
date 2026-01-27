---
name: tmuxx-committing-changes
description: "Use this to check, audit, and commit changes to the git repository. Enforces a pre-commit checklist including documentation updates and code quality checks."
---

# Committing Changes to Tmuxx

You use this skill before every commit to ensure the codebase remains healthy and well-documented.

## Requirements
- Must have functional changes ready to be staged.

## Steps

### 1. Analysis (Pre-commit Checklist)
Verify that:
- `CHANGELOG.md` is updated with the changes.
- `README.md` is updated if new config options or keybindings were added.
- The code builds and tests pass.
- **Git Lock**: Check if `.git/index.lock` exists. If so, remove it (`rm -f .git/index.lock`) before proceeding.

### 2. Execution

#### A. Code Quality
Run the following and fix all issues:
```bash
cargo build --release
cargo clippy
cargo fmt
```

#### B. Staging
```bash
git add -A
git status --short
```

#### C. Committing
Write a clear commit message following this format:
```
<type>: Brief description

Problem: What issue this solves
Solution: How it was solved

Changes:
- Bullet points...
```
Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`.

### 3. Verification
- Verify the commit was successful using `git log -1`.
- Ensure no accidental files were staged.
