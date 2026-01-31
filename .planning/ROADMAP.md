# Roadmap: tmuxx

## Overview

This roadmap establishes general development practices for tmuxx, focusing on code quality, extensibility, and maintainability. These are ongoing goals rather than feature deliverables, providing a foundation for future milestone-specific work.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Code Quality Foundation** - Refactor and clean codebase
- [ ] **Phase 2: Extensibility Improvements** - Enable easy customization and extensions
- [ ] **Phase 3: Maintenance Excellence** - Establish sustainable development practices

## Phase Details

### Phase 1: Code Quality Foundation
**Goal**: Codebase is clean, well-structured, and maintainable
**Depends on**: Nothing (first phase)
**Requirements**: QUAL-01, QUAL-02, QUAL-03
**Success Criteria** (what must be TRUE):
  1. All source files under 1000 lines (ui/app.rs, app/state.rs, app/config.rs refactored)
  2. Zero code duplication detected across modules
  3. `cargo clippy` runs with zero warnings
  4. Modules have clear, focused responsibilities
**Plans**: TBD

Plans:
- TBD (will be defined when planning this phase)

### Phase 2: Extensibility Improvements
**Goal**: New agent types and patterns can be added without core changes
**Depends on**: Phase 1
**Requirements**: EXT-01, EXT-02, EXT-03
**Success Criteria** (what must be TRUE):
  1. New agent type can be added by creating single parser file (no core edits)
  2. Custom detection patterns configurable via defaults.toml
  3. Parser registry discovers parsers automatically (no manual registration)
  4. Documentation explains how to add custom agents
**Plans**: TBD

Plans:
- TBD (will be defined when planning this phase)

### Phase 3: Maintenance Excellence
**Goal**: Development practices ensure stability and compatibility
**Depends on**: Phase 2
**Requirements**: MAINT-01, MAINT-02, MAINT-03
**Success Criteria** (what must be TRUE):
  1. 100% regression test pass rate maintained (no commits break tests)
  2. All modules have inline documentation explaining purpose and usage
  3. Configuration file format remains backward compatible across versions
  4. CHANGELOG.md accurately reflects all user-facing changes
**Plans**: TBD

Plans:
- TBD (will be defined when planning this phase)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Code Quality Foundation | 0/TBD | Not started | - |
| 2. Extensibility Improvements | 0/TBD | Not started | - |
| 3. Maintenance Excellence | 0/TBD | Not started | - |

---
*Roadmap created: 2026-01-30*
*Last updated: 2026-01-30*
