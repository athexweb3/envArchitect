use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub version_req: Option<String>,
    pub package_name: Option<String>, // Override if different from name
}

impl Tool {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version_req: None,
            package_name: None,
        }
    }
}
