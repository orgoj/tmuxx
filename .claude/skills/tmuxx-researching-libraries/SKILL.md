---
name: tmuxx-researching-libraries
description: "Use this to assess, check, and review third-party libraries before implementing new functionality. Prevents reinventing the wheel and ensures modern standards."
---

# Researching Libraries

You use this skill before implementing any significant new feature to find existing Rust libraries that can handle the heavy lifting.

## Steps

### 1. Analysis
- Define the core functionality needed (e.g., "modal text editor", "fuzzy finder").
- Identify constraints (e.g., must be compatible with `ratatui 0.29`, must be thread-safe).

### 2. Execution

#### A. Web Search
Search for modern libraries (targeting current year 2026).
Example: `WebSearch: "rust ratatui text editor widget library 2026"`

#### B. Documentation Review
Use tools like `rtfmbro` to read READMEs and docs.
```bash
mcp__rtfmbro__get_readme package="owner/repo" version="*" ecosystem="gh"
```

#### C. Pattern Exploration
Look at library examples and check trait implementations (e.g., `From`/`Into` for key events).

### 3. Verification
- Verify the library is maintained and compatible with the project's dependencies.
- Confirm it follows the project's UI patterns (e.g., centered popups using `Flex::Center`).
- Present the findings to the user for a final decision before adding to `Cargo.toml`.
