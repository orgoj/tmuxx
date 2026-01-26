---
name: tmuxx-planning
description: Implementation planning workflow for tmuxx features - use BEFORE writing code
---

# Implementation Planning for tmuxx

**Use this skill BEFORE implementing any feature or fix!**

## META: Planning Saves Time

**User consistently finds 5-7 issues per plan during technical review.**

This is NORMAL and EXPECTED - not a sign of poor work. Planning workflow catches issues before coding.

## Planning Workflow

### Step 1: Invoke Brainstorming Skill

**For new features or complex changes:**

Use `superpowers:brainstorming` skill to:
- Ask clarifying questions ONE AT A TIME
- Present design in sections for validation
- Use multiple choice when possible
- Don't jump to implementation

### Step 2: Explore Codebase

**Use Task tool with Explore agent to:**
- Find existing patterns to follow
- Identify all files that need modification
- Understand current architecture
- Discover method signatures and APIs

**NEVER assume method exists without checking!**

### Step 3: Ask Clarifying Questions

**Before writing plan, ask:**

**Integration Strategy:**
- "Should X replace or coexist with Y?"
- "Will this break existing functionality?"
- "Should there be a config option to enable/disable?"

**UX/Behavior:**
- "What keybinding should trigger this?"
- "What should happen on error?"
- "Should this work for single agents or only multi-selection?"

**Technical:**
- "Which library should I use?"
- "Should this be async or sync?"
- "Any performance concerns?"

### Step 4: Write Implementation Plan

**Plan MUST include:**
- Explicit code patterns (not "TODO" or "placeholder")
- Actual method names (verified to exist)
- Complete code snippets (no placeholders)
- All files that need modification
- Integration points with existing code

**NO PLACEHOLDERS ALLOWED:**
```rust
// ❌ WRONG - placeholder
Key::Char(' ')  // Placeholder - need proper key conversion

// ✅ CORRECT - complete code
let input: Input = key_event.into();  // Use Into trait
modal.handle_input(input);
```

### Step 5: User Technical Review

**Expect 5-7 corrections - this is normal!**

User will catch:
- API misuse (wrong method signatures)
- Missing trait implementations (Into/From available)
- Wrong patterns (from codebase)
- Edge cases (off-by-one, error handling)
- Integration issues (conflicts with existing code)

**Apply ALL corrections before implementation begins!**

## Integration Philosophy

**Coexistence over Replacement:**

When adding new features, preserve existing functionality:

```rust
// ❌ WRONG - breaks existing workflow
fn handle_input(&mut self, key: KeyEvent) {
    // Only modal textarea, no bottom input
}

// ✅ CORRECT - both systems coexist
fn handle_input(&mut self, key: KeyEvent) {
    if self.modal_active {
        self.modal.handle_input(key);
    } else {
        self.bottom_input.handle_input(key);
    }
}
```

**Config Options Enable User Choice:**

```toml
# User can choose preferred method
hide_bottom_input = true  # Default: encourage modal usage
# hide_bottom_input = false  # User prefers bottom input
```

**Benefits:**
- No breaking changes
- Gradual migration path
- Users choose their workflow
- Easier to test (compare old vs new)

## Error Handling Patterns

**Immediate Feedback with deny_unknown_fields:**

```rust
#[serde(deny_unknown_fields)]
pub struct Config { ... }
```

Config with typos fails immediately with clear error.

**Status vs Error Distinction:**

```rust
// ✅ Success - green with ✓
state.set_status("✓ Command executed");

// ❌ Error - red with ✗
state.set_error("✗ Failed to execute command");
```

**Show Expanded Values:**

```rust
// ❌ WRONG - shows template
echo "Opening ${SESSION_DIR}/file.txt"

// ✅ CORRECT - shows actual path
echo "Opening /home/user/projects/tmuxx/file.txt"
```

## Common Planning Mistakes

### Mistake 1: Assumption-Based Planning

**Wrong:**
- Assume modal should replace existing input
- Assume keybinding without asking
- Assume synchronous execution is fine

**Correct:**
- Ask "Should modal replace or coexist?"
- Ask "What keybinding?"
- Ask "Should this be async?"

### Mistake 2: Placeholders in Plans

**Wrong:**
```rust
// TODO: need proper key conversion
let input = convert_key(key);
```

**Correct:**
```rust
let input: Input = key.into();
```

### Mistake 3: Non-Existent Methods

**Wrong:**
```rust
state.selected_agent()  // Doesn't exist!
```

**Correct:**
```rust
// First verify with: rg "fn.*agent" src/app/state.rs
state.agents.get_agent(state.selected_index)
```

### Mistake 4: Missing Trait Checks

**Wrong:**
```rust
// Manual conversion because didn't check library
let input = manual_key_conversion(key);
```

**Correct:**
```rust
// Check docs: library provides Into<Input>
let input: Input = key.into();
```

## Planning Checklist

Before submitting plan for review:

- [ ] Explored codebase for existing patterns
- [ ] Asked clarifying questions (integration, UX, technical)
- [ ] Verified all methods exist in codebase
- [ ] Checked library docs for trait implementations
- [ ] Included explicit code patterns (no placeholders)
- [ ] Listed all files to modify
- [ ] Considered config options for user choice
- [ ] Error handling strategy defined
- [ ] Ready for 5-7 corrections in review

## Remember

**Technical review is NOT criticism - it's preventing bugs!**

User catches issues because:
- They know the codebase better
- They spot anti-patterns from experience
- They remember edge cases you missed

**5-7 corrections = successful planning workflow.**

Apply all corrections, then implement with confidence.
