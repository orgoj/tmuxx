---
name: tmuxx-add-new-feature
description: Strict workflow for adding interactive features (Backend -> Config -> Action -> UI -> Docs)
---

# Adding New Features to Tmuxx

This skill outlines the strict workflow for adding new interactive features to tmuxx.
**CRITICAL**: Tmuxx is a **config-driven** application. Every action must be mappable via `config.toml`.

## Workflow Checklist

### 1. Core Implementation (The "Doer")
Implement the low-level logic first.
- If it's a tmux command: Edit `src/tmux/client.rs`.
- If it's purely internal state: Edit `src/app/state.rs`.

### 2. Configuration Schema (The "Interface")
Expose the feature to the configuration system so users can bind keys to it.
- **File**: `src/app/key_binding.rs`
- **Action**: Add a new variant to `enum KeyAction`.
  ```rust
  pub enum KeyAction {
      // ...
      MyNewFeature, // Add this
  }
  ```

### 3. Application Action (The "Messenger")
Define the internal action used within the app event loop.
- **File**: `src/app/actions.rs`
- **Action**:
    1. Add variant to `enum Action`.
    2. Update `impl Action { fn description() }` to provide a help text.

### 4. Default Binding (The "Config")
Bind the feature to a default key.
- **File**: `src/config/defaults.toml`
- **Action**: Add the entry under `[key_bindings]`.
  ```toml
  "Key" = "my_snake_case_feature_name"
  ```
  *(Note: Ensure the key isn't already taken or provides a sensible fallback).*

### 5. UI Integration (The "Controller")
Wire everything together in the main event loop.
- **File**: `src/ui/app.rs`
- **Step A (`map_key_to_action`)**: Map the `KeyAction` (from config) to `Action` (internal).
- **Step B (`run_loop`)**: Handle the `Action` execution.
    - If it requires a popup:
        1. Add `PopupType` in `src/app/state.rs`.
        2. Set `state.popup_input` in `run_loop`.
        3. Handle `Action::PopupInputSubmit` for the execution logic.

### 6. Documentation (The "Paperwork")
**Mandatory** for every visible feature.
- **CHANGELOG.md**: Add entry under `[Unreleased]` -> `### Added` or `### Fixed`.
- **README.md**:
    - Update the **Key Bindings** table.
    - If it's a complex feature, add a section in **Features**.
- **TODO.md**: Check off the relevant task.

## Example: Adding "Kill Session"

1. **`src/tmux/client.rs`**: Added `kill_session()` method.
2. **`src/app/key_binding.rs`**: Added `KeyAction::KillSession`.
3. **`src/app/actions.rs`**: Added `Action::KillSession`.
4. **`src/config/defaults.toml`**: Added `"X" = "kill_session"`.
5. **`src/ui/app.rs`**:
    - Mapped `KeyAction::KillSession` -> `Action::KillSession`.
    - Handled `Action::KillSession` -> Show `PopupType::KillConfirmation`.
    - Handled `PopupInputSubmit` -> Call `client.kill_session()`.
6. **Docs**: Updated CHANGELOG and README tables.
