use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuConfig {
    #[serde(default)]
    pub items: Vec<MenuItem>,

    /// If true, these items will be appended to the existing/default items instead of replacing them
    #[serde(default)]
    pub merge_with_defaults: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuItem {
    pub name: String,

    #[serde(default)]
    pub description: Option<String>,

    pub execute_command: Option<crate::app::key_binding::CommandConfig>,

    pub text: Option<String>,

    #[serde(default)]
    pub items: Vec<MenuItem>,

    /// Variables to prompt for before executing command
    /// Key: variable name (will be used in command as ${VAR_NAME})
    /// Value: prompt text to display to user
    #[serde(default)]
    pub variables: HashMap<String, String>,
}
