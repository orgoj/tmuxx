---
name: tmuxx-adding-config-options
description: "Use this to assess, implement, and verify new configuration options in tmuxx. Covers modifying config.rs, adding CLI overrides, updating README, and verifying with 'deny_unknown_fields'."
---

# Adding Config Options to Tmuxx

You use this skill when you need to add a new configuration setting to the application. This ensures the setting is properly defined, can be overridden via CLI, is documented, and follows the project's strict type safety rules.

## Parameters
- `option_name`: The name of the setting in snake_case.
- `type`: The Rust type (e.g., `bool`, `String`, `u16`).
- `default_value`: The value used if not specified in config.

## Steps

### 1. Analysis
- Identify where the new option will be used in the logic.
- Verify if the option should support CLI overrides (most should).
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

#### B. Add Override Support in `src/app/config_override.rs`
1. Add a variant to `enum ConfigOverride`.
2. Update the `parse` method to handle the string key (include short names if relevant).
3. Update the `apply` method to transfer the value to the `Config` struct.

#### C. Use in Application
If needed, pass the new config value from `Config` to `TmuxClient` or other components in `src/app/mod.rs` or `src/ui/app.rs`.

#### D. Documentation
- Add the new option to the example `config.toml` in `README.md`.
- Add to the "Available config keys" list in `README.md`.
- Add a "Changed" or "Added" entry to `CHANGELOG.md` under `[Unreleased]`.

### 3. Verification

#### A. Typos and Validation
Ensure the `Config` struct has `#[serde(deny_unknown_fields)]`.
Test by adding a typo to your local `config.toml` and running the app; it should error out immediately.

#### B. Runtime Test
```bash
cargo build
# Test CLI override
./target/debug/tmuxx --set option_name=false
```
Verify the behavior in a test tmux session (e.g., `ct-test`).
