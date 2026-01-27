---
name: tmuxx-bumping-versions
description: "Use this to coordinate, update, and audit project version increments across Cargo.toml, CHANGELOG.md, and TODO.md before a release."
---

# Bumping Project Version

You use this skill when preparing a new release. It ensures all version strings are synchronized and that the changelog correctly reflects the transition from `[Unreleased]` to a stable version.

## Requirements
- Must be in the root of the git repository.
- All features for the release must be completed and documented in `[Unreleased]`.

## Steps

### 1. Analysis
- Verify the current version in `Cargo.toml`.
- Review `CHANGELOG.md`'s `[Unreleased]` section to determine the next version (Patch, Minor, or Major) based on SemVer.

### 2. Execution

#### A. Update Manifests
- Edit `Cargo.toml` and set the new version.
- Run `cargo check` to update `Cargo.lock`.

#### B. Update CHANGELOG.md
- Rename the `[Unreleased]` header to `[NEW_VERSION] - YYYY-MM-DD`.
- Insert a new empty `[Unreleased]` section at the top.
- **STRICT**: Do not modify existing release sections.

#### C. Clean TODO.md
- Remove completed tasks (lines starting with `- [x]`) to keep the roadmap clean for the next cycle.

### 3. Verification
- Run `cargo run -- --version` and verify it matches the expected new version.
- Audit `CHANGELOG.md` to ensure the formatting is correct and no old releases were touched.
- Use `tmuxx-committing-changes` to commit the bump with message `chore: Bump version to X.Y.Z`.
