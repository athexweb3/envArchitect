use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub source: PluginSource,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginSource {
    Registry { name: String },
    Url { url: String },
    Path { path: String },
}
