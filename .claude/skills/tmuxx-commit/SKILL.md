---
name: tmuxx-commit
description: Pre-commit checklist and git workflow for tmuxx project
---

# Pre-Commit Workflow for tmuxx

**Use this skill before EVERY commit!**

## Pre-Commit Checklist (NON-NEGOTIABLE)

Before EVERY commit with new features/config options:

1. ✅ **Update CHANGELOG.md** - Add feature to Unreleased section
   - Describe what was added/changed/fixed
   - Include config options with defaults
   - Include CLI override examples

2. ✅ **Update README.md** - Add config options to Configuration section (if applicable)
   - Add to config.toml example with comments
   - Add to "Available config keys" list with description
   - Update default values if changed

3. ✅ **Build and test**
   ```bash
   cargo build --release
   cargo clippy  # Fix all warnings!
   cargo fmt
   ```

4. ✅ **Stage all changes**
   ```bash
   git add -A  # ALWAYS use -A, never specific files
   git status --short  # Verify what will be committed
   ```

5. ✅ **Write commit message** - Clear description with Co-Authored-By

**If you skip documentation updates, commit will be REJECTED!**

## Commit Message Format

```
<type>: Brief description (imperative mood)

Problem: What issue this solves
Solution: How it was solved (bullet points)

Changes:
- File changes
- Config updates
- Documentation updates

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Types:** feat, fix, refactor, docs, test, chore

## Git Remotes

- `origin` - git@github.com:orgoj/tmuxx.git (main fork)
- `original` - git@github.com:nyanko3141592/tmuxx.git (upstream)
- `neon` - git@github.com:frantisek-heca/tmuxx-neon.git (tracking)

## Important Notes

- This is a FORK - changes pushed to `orgoj` branch only
- NO publishing to crates.io (that's upstream's job)
- Work on `orgoj` branch
