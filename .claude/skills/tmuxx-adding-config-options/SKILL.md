---
name: tmuxx-adding-config-options
description: "Use this to assess, implement, and verify new configuration options in tmuxx. Covers modifying config.rs, adding CLI overrides (via generic set), updating defaults, and documenting."
---

# Adding Config Options to Tmuxx

You use this skill when you need to add a new configuration setting to the application. This ensures the setting is properly defined, can be overridden via CLI **(using the generic `--set` mechanism)**, is documented, and follows the project's strict type safety rules.

## Parameters
- `option_name`: The name of the setting in snake_case.
- `type`: The Rust type (e.g., `bool`, `String`, `u16`).
- `default_value`: The value used if not specified in config.

## Steps

### 1. Analysis
- Identify where the new option will be used in the logic.
- **CRITICAL**: Do NOT add new top-level CLI arguments to `main.rs`. We use a generic `--set key=value` system.
- Check `src/app/config.rs` for existing patterns.

### 2. Execution

#### A. Define Field in `src/app/config.rs`
Add the field to the `Config` struct with a default function.
```rust
/// Description of the option
#[serde(default = "default_option_name")]
pub option_name: bool,

fn default_option_name() -> bool {
    true
}
```
Update `impl Default for Config` to use the new default function.

#### B. Update Defaults in `src/config/defaults.toml`
- Add the new option with its default value and a comment.
- This serves as the source of truth for the default configuration.

#### C. Add Override Support in `src/app/config_override.rs`
1. Add a variant to `enum ConfigOverride`.
2. Update the `parse` method to handle the string key (include short names/aliases if relevant).
   - **Note**: This hooks into the generic `--set` CLI argument.
3. Update the `apply` method to transfer the value to the `Config` struct.

#### D. Use in Application
Pass the new config value from `Config` to components (`TmuxClient`, `run_loop`, etc.).

#### E. Documentation
- Add the new option to the example `config.toml` in `README.md` (Configuration section).
- Add a "Changed" or "Added" entry to `CHANGELOG.md` under `[Unreleased]`.

### 3. Verification

#### A. Typos and Validation
Ensure the `Config` struct has `#[serde(deny_unknown_fields)]`.

#### B. Runtime Test
```bash
cargo build
# Test CLI override using the generic set argument
./target/debug/tmuxx --set option_name=false
```
Verify the behavior in a test tmux session.
