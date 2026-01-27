---
name: tmuxx-planning
description: "Use this to assess, create, and review implementation plans for tmuxx features or fixes. Mandatory before writing any code."
---

# Implementation Planning for Tmuxx

You use this skill to create a detailed technical roadmap for a task. Planning prevents bugs and ensures alignment with the project's architecture.

## Steps

### 1. Analysis
- Explore the codebase to find existing patterns and verify method signatures.
- **NEVER** assume a method exists without checking with `rg` or `read`.
- Identify all files that will be touched.

### 2. Execution

#### A. Brainstorming
Ask the user clarifying questions about:
- **Integration Strategy**: Should this replace or coexist with old logic?
- **UX/Behavior**: What are the trigger keys and expected visual results?
- **Technical**: Which libraries or traits should be used?

#### B. Drafting the Plan
The plan **MUST** include:
- Explicit code patterns (no placeholders).
- Actual verified method names.
- A list of all files to be modified.
- Error handling strategy (e.g., using `deny_unknown_fields`).

### 3. Verification (Technical Review)
- Present the plan to the user.
- **Expect and embrace corrections** (typically 5-7).
- Apply all corrections to the plan before proceeding to implementation.
- Confirm the plan respects the "Coexistence over Replacement" philosophy.
