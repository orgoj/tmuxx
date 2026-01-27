---
name: tmuxx-bump-version
description: Synchronize version across Cargo.toml, CHANGELOG.md, and other files. STRICT: Never modify existing releases.
---

# Bump Project Version

Use this skill when preparing a new release.
**CRITICAL RULE**: Never modify an existing version header in `CHANGELOG.md`. Released versions are immutable.

## 1. Verify Current State
Check the current version in `Cargo.toml` and the latest version in `CHANGELOG.md`.
```bash
grep "^version" Cargo.toml
head -n 20 CHANGELOG.md
```

## 2. Determine New Version
Decide the next version number based on [SemVer](https://semver.org/) and the changes in `[Unreleased]`.
- **Patch** (0.0.x): Bug fixes, internal tweaks.
- **Minor** (0.x.0): New features, config options.
- **Major** (x.0.0): Breaking changes.

## 3. Update Cargo.toml
Edit `Cargo.toml` and set the **NEW** version.
```toml
[package]
version = "NEW_VERSION"
```

## 4. Update Cargo.lock
Run `cargo check` to automatically update `Cargo.lock`.
```bash
cargo check
```

## 5. Update CHANGELOG.md
Strict workflow:
1.  **Rename** the current `[Unreleased]` header to `[NEW_VERSION] - YYYY-MM-DD`.
2.  **Create** a new empty `[Unreleased]` section at the very top.
3.  **NEVER** merge changes into an existing version block.

Example structure:
```markdown
## [Unreleased]

## [0.2.1] - 2026-01-27
### Added
- New feature...

## [0.2.0] - 2026-01-26
...
```

## 6. Clean TODO.md
Remove completed tasks to keep the roadmap clean.
1.  Open `TODO.md`.
2.  Remove all lines starting with `- [x]`.
3.  Ensure the file structure remains valid.

## 7. Verify
Ensure the application reports the correct version.
```bash
cargo run -- --version
```

## 8. Commit
Use `tmuxx-commit` to commit the version bump.
Message: `chore: Bump version to X.Y.Z`
