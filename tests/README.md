# Tmuxx Regression Tests

Directory: `tests/fixtures/<agent_id>/`

## How to Add a Test (Fixture)

The easiest way is to use the **capture script** while an instance is running:

```bash
# ./tests/capture.sh <agent_id> <target_pane> <expected_status> [description]
./tests/capture.sh claude cc-ai-maestro idle "main_menu"
```

This will create a file like `case_idle_main_menu.txt` in `tests/fixtures/claude/`.

### Naming Convention

The test runner (`tmuxx test`) uses the filename to determine the expected state.
Format: `case_<STATUS>_<DESC>.txt`

**Valid STATUS values** (must match strings in `defaults.toml`):
- `idle`
- `processing`
- `awaiting_approval`
- `awaiting_input`
- `error`

Examples:
- `case_idle_clean_prompt.txt`
- `case_awaiting_approval_delete_file.txt`
- `case_processing_thinking.txt`

## File Structure

The file contains raw text output from `tmux capture-pane`.
TUI artifacts (like border pipes `│`) are automatically cleaned by the capture script for cleaner testing, but regexes are generally designed to handle raw output as well.

## Running Tests

```bash
cargo run -- test --dir tests/fixtures/claude
```
