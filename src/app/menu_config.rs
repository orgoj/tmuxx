use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuConfig {
    #[serde(default)]
    pub items: Vec<MenuItem>,
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
}
