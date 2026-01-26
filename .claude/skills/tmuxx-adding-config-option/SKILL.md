---
name: tmuxx-adding-config-option
description: Use when adding a new config option to tmuxx (bool, string, number)
---

# Adding Config Option to Tmuxx

Pattern for adding new config options with --set CLI override support.

## Files to Modify

| File | What to Add |
|------|-------------|
| `src/app/config.rs` | Field + default function + Default impl |
| `src/app/config_override.rs` | Enum variant + parse case + apply case |
| `README.md` | Config section + CLI override section |
| `CHANGELOG.md` | Feature description |

## Pattern

### 1. Config Field (`src/app/config.rs`)

```rust
/// Description
#[serde(default = "default_option_name")]
pub option_name: bool,

fn default_option_name() -> bool {
    true  // or false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... existing
            option_name: default_option_name(),
        }
    }
}
```

### 2. Override Support (`src/app/config_override.rs`)

```rust
pub enum ConfigOverride {
    // ... existing
    OptionName(bool),
}

// In parse() match:
"optionname" | "shortname" => {
    let val = parse_bool(value)?;
    Ok(ConfigOverride::OptionName(val))
}

// In apply():
ConfigOverride::OptionName(val) => config.option_name = val,
```

### 3. Use in TmuxClient (if needed)

```rust
pub struct TmuxClient {
    option_name: bool,
}

pub fn from_config(config: &Config) -> Self {
    Self {
        // ... existing
        option_name: config.option_name,
    }
}
```

## Example

See `show_detached_sessions` implementation:
- Commit: `feat: Add config override system with show_detached_sessions option`
- Files: config.rs, config_override.rs, client.rs
- Runtime tested in tmux

Follow this exact pattern.

## CRITICAL: Use serde deny_unknown_fields

**ALWAYS add `deny_unknown_fields` to Config struct to catch typos early!**

```rust
#[serde(default, deny_unknown_fields)]
pub struct Config {
    // ...
}
```

**Why:**
- **Immediate feedback**: Config with typos fails at load time, not silently ignored
- **User-friendly**: User gets error: "unknown field `collor` (did you mean `color`?)"
- **Without this**: Typos silently ignored, feature "doesn't work" with no error message

**Example of failure without deny_unknown_fields:**
```toml
# User types "collor" instead of "color"
status_collor = "blue"  # Typo!

# Without deny_unknown_fields: Silently ignored, status stays default color
# With deny_unknown_fields: Error at startup, user sees typo immediately
```

**Testing:**
```bash
# Add typo to config.toml
echo "wrong_field = true" >> ~/.config/tmuxx/config.toml

# Test with deny_unknown_fields
./target/release/tmuxx  # Should error: "unknown field `wrong_field`"

# Remove typo
sed -i '/wrong_field/d' ~/.config/tmuxx/config.toml
```
