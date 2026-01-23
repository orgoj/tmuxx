---
name: tmuxcc-changelog
description: TODO.md and CHANGELOG.md management workflow for tmuxcc
---

# TODO.md and CHANGELOG.md Management

**CRITICAL: Keep TODO.md clean - completed tasks don't belong there!**

## When a Task is Completed

1. **Move to CHANGELOG.md** - Document what was done with proper detail
2. **Delete from TODO.md** - Don't leave completed tasks in TODO
3. **Mark as ✅ COMPLETED** only temporarily if needs verification, then move to CHANGELOG

## Why This Matters

- **TODO.md is for ACTIVE work** - what needs doing
- Completed tasks haunting TODO confuse future sessions
- **CHANGELOG.md is the proper place** for completed work history
- Keep TODO focused on next steps, not past achievements

## Workflow

```
Task done → Update CHANGELOG.md → Delete from TODO.md → Git commit
```

## Don't Do This

- ❌ Leave tasks marked "✅ COMPLETED" in TODO.md long-term
- ❌ Accumulate completed tasks at the top of TODO.md
- ❌ Use TODO.md as a changelog

## Do This

- ✅ Move completed work to CHANGELOG.md immediately
- ✅ Keep TODO.md focused on current/upcoming work
- ✅ Use "Completed Tasks ✅" section only as temporary staging before CHANGELOG move

## CHANGELOG.md Format

Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [Unreleased]

### Added
- New features

### Fixed
- Bug fixes

### Changed
- Changes to existing functionality

### Removed
- Removed features
```

## Example Entry

```markdown
### Fixed
- **Preview Empty Display** - Fixed issue where pane preview showed nothing when content had trailing empty lines
  - Preview now trims trailing empty lines before displaying content
  - Affected both summary and detailed preview views
  - Ensures actual content is visible even when tmux pane has many blank lines at bottom
```
