use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiDocument {
    pub version: u32,
    #[serde(default)]
    pub screens: Vec<UiScreen>,
    #[serde(default)]
    pub theme: UiTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiTheme {
    #[serde(default = "default_primary")]
    pub primary: String,
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_text")]
    pub text: String,
}

fn default_primary() -> String {
    "#4a90d9".into()
}

fn default_bg() -> String {
    "#1a1a2e".into()
}

fn default_text() -> String {
    "#ffffff".into()
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            primary: default_primary(),
            background: default_bg(),
            text: default_text(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiScreen {
    pub id: String,
    #[serde(default = "default_layer")]
    pub layer: String,
    pub root: UiWidget,
}

fn default_layer() -> String {
    "overlay".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiWidget {
    #[serde(rename = "type")]
    pub widget_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UiWidget>,
}

impl UiDocument {
    pub fn default_empty() -> Self {
        Self {
            version: 1,
            screens: vec![],
            theme: UiTheme::default(),
        }
    }

    pub fn new_widget_id(prefix: &str) -> String {
        format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
    }
}
