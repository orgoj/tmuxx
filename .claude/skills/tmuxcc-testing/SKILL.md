---
name: tmuxcc-testing
description: Testing workflow and tmux safety rules for tmuxcc development
---

# Testing Workflow for tmuxcc

**CRITICAL: Always use this skill when testing tmuxcc!**

## META RULE: WRITE FIRST, DO LATER!

- When user teaches you something new → WRITE IT TO CLAUDE.md IMMEDIATELY
- When user corrects you → WRITE THE CORRECTION TO CLAUDE.md FIRST
- When you learn a rule → WRITE IT DOWN BEFORE using it
- **NEVER do things first and write later!**
- **If electricity fails, you lose everything not written down!**
- **Next session you won't remember anything not in CLAUDE.md!**

## Test Sessions Structure

- `ct-test` - Session where tmuxcc runs and DISPLAYS other sessions
- `ct-multi` - Test session with 5 windows for multi-window testing
- Other sessions: `cc-test`, `cc-tmuxcc`, `cc-MOP`, `cc-tmp`

## ONLY Session for send-keys: `ct-test`

- This is THE ONLY session where you send keys for testing
- ct-test is where you create test content that tmuxcc monitors
- ❌ WRONG: `tmux send-keys -t ct-test:0 "command"`
- ✅ CORRECT: `tmux send-keys -t ct-test "command"`
- **NEVER add :0 or :1 or any window number!**

## CRITICAL: Send Keys ONE AT A TIME

**NEVER send multiple keys without checking between each one!**

```bash
# ❌ WRONG - multiple keys in loop, if tmuxcc crashes keys go to bash and delete things!
for i in {1..30}; do
  tmux send-keys -t ct-test "" Enter
done

# ✅ CORRECT - ONE key, then CHECK what happened
tmux send-keys -t ct-test "echo 'test'" Enter
sleep 0.5
tmux capture-pane -t ct-test -p  # CHECK result!

# Then next key...
tmux send-keys -t ct-test "" Enter
sleep 0.5
tmux capture-pane -t ct-test -p  # CHECK again!
```

**Why:** If tmuxcc crashes/exits, subsequent keys go to bash and can execute destructive commands!

## Test Scripts

- `scripts/reload-test.sh` - Reload tmuxcc in ct-test session **(USE THIS)**
- `scripts/start-test-session.sh` - Start ct-test session
- `scripts/setup-multi-test.sh` - Setup ct-multi session with multiple windows
- `scripts/cp-bin.sh` - Install tmuxcc to ~/bin (DON'T USE - user has working version!)

## Testing Workflow

1. Use `./target/release/tmuxcc` for testing (never cp-bin.sh)
2. Use `scripts/reload-test.sh` to reload tmuxcc in ct-test session
3. **INVOKE tmux-automation skill with Skill tool** - don't skip this step!
4. Use tmux-automation skill to interact with TUI and verify behavior
5. **NEVER ask user to test** - testing is YOUR responsibility
6. **NEVER claim completion without runtime verification** - visual verification mandatory for UI features
7. **NEVER kill test sessions!** Use scripts to reload, not kill and recreate

## CRITICAL TMUX SAFETY RULES (NON-NEGOTIABLE)

### 1. NEVER use tail/head with capture-pane!

```bash
# ❌ WRONG
tmux capture-pane -t session -p | tail -30

# ✅ CORRECT
tmux capture-pane -t session -p
```

**Why:** Line 31 could be `reboot` or other destructive command!

### 2. Empty capture = ERROR state → STOP IMMEDIATELY!

- If `capture-pane -p` returns empty → DON'T send any commands
- Check session exists, check for errors
- **NEVER proceed without visible output**

### 3. No bash prompt = ERROR state → STOP IMMEDIATELY!

- If you don't see `$`, `>`, or clear input prompt → DON'T send commands
- Something is wrong with the session
- **NEVER blindly send Enter or other keys**

### 4. ALWAYS capture FULL screen first to understand state

```bash
tmux capture-pane -t ct-test -p  # Full screen, no tail!
```

### 5. Check what you're doing BEFORE sending keys

- Capture full screen
- Verify prompt is visible
- Verify expected state
- ONLY THEN send commands

### 6. NEVER send keys to session where tmuxcc is RUNNING!

- ❌ FATAL: `tmux send-keys -t ct-test "y"` when tmuxcc runs there
- **Why:** tmuxcc forwards keys to monitored sessions → unintended approvals!
- ✅ CORRECT: Use dedicated test session WITHOUT tmuxcc for interactive testing
- **Testing tmuxcc:** Only capture output, NEVER send keys to ct-test!
