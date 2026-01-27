---
name: tmuxx-adding-new-features
description: "Use this to assess, create, and review new interactive features in tmuxx. Covers the full lifecycle from backend implementation to configuration mapping and UI integration."
---

# Adding New Features to Tmuxx

You use this skill to implement new user-facing functionality. Tmuxx is a **config-driven** application, meaning every new action must be mappable via `config.toml` so users can bind keys to it.

## Steps

### 1. Analysis
- Determine if the feature requires a new tmux command or purely internal state changes.
- Identify the necessary configuration changes (key bindings, new config options).
- Verify if any existing libraries can be used (consult `tmuxx-researching-libraries`).

### 2. Execution

#### A. Core Implementation
Implement the low-level logic first.
- **Tmux commands**: `src/tmux/client.rs`
- **Internal state**: `src/app/state.rs`

#### B. Configuration Schema
Expose the feature to the configuration system.
- **File**: `src/app/key_binding.rs`
- **Action**: Add a new variant to `enum KeyAction`.

#### C. Application Action
Define the internal action used within the event loop.
- **File**: `src/app/actions.rs`
- Add variant to `enum Action` and update `impl Action { fn description() }`.

#### D. Default Binding
- **File**: `src/config/defaults.toml`
- Add the entry under `[key_bindings]`.

#### E. UI Integration
- **File**: `src/ui/app.rs`
- Map `KeyAction` to `Action` in `map_key_to_action`.
- Handle the execution logic in the main `run_loop`.

#### F. Documentation
- Update `CHANGELOG.md` under `[Unreleased]`.
- Update `README.md` (Key Bindings table and Features section).

### 3. Verification
- Use `tmuxx-testing` skill to verify the feature in a live tmux session.
- Ensure `cargo clippy` and `cargo fmt` are run.
- Check that the new action is listed in the help UI or described correctly in `Action::description()`.
