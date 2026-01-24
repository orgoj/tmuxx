---
name: tmuxcc-gemini-review
description: Use Gemini CLI for code review in tmuxcc project
---

# Gemini Code Review for tmuxcc

**Use this skill when you need AI-powered code review before committing changes.**

## Prerequisites

- Gemini CLI installed (`gemini` command available)
- Project uses git for version control

## Quick Start

### Review git diff (all changes)
```bash
git diff | gemini -p "Code review: bugs, issues, style. Be concise."
```

### Review specific file
```bash
gemini review --file src/app/actions.rs
```

### Review staged changes
```bash
git diff --staged | gemini -p "Review staged changes"
```

### Review last commit
```bash
git diff HEAD~1 | gemini -p "Review last commit"
```

## Review Areas

### General Code Review
```bash
git diff | gemini -p "Code review: bugs, issues, style. Be concise."
```

### Security Focus
```bash
git diff | gemini -p "Security review: check for vulnerabilities, unsafe code, and security best practices"
```

### Performance Focus
```bash
git diff | gemini -p "Performance review: check for inefficiencies, bottlenecks, and optimization opportunities"
```

### Bug Detection
```bash
git diff | gemini -p "Bug detection: find potential bugs, edge cases, and logic errors"
```

### Code Style
```bash
git diff | gemini -p "Style review: check Rust conventions, naming, and idiomatic code"
```

## Interactive vs One-Shot

### One-shot mode (default, `-p` flag)
```bash
# Runs once and exits
git diff | gemini -p "Review this"
```

### Interactive mode (`-i` flag)
```bash
# Starts interactive session
git diff | gemini -i "Review this"
# Then continue conversation with follow-up questions
```

## Tips

1. **Be specific with prompts** - Tell Gemini what to focus on
2. **Use English prompts** - Gemini works better with English
3. **Keep it concise** - Long prompts don't always mean better results
4. **Pipe from git** - Always use `git diff | gemini` for context
5. **Review in chunks** - For large diffs, review specific files instead

## Common Workflows

### Before commit review
```bash
# Review all unstaged changes
git diff | gemini -p "Code review: bugs, style, issues. Concise."
```

### Review specific component
```bash
# Review only parsers
git diff src/parsers/ | gemini -p "Review parser changes: correctness, edge cases, regex patterns"
```

### Get implementation suggestions (when needed)
```bash
git diff | gemini -p "Review and suggest improvements. Include code examples."
```
