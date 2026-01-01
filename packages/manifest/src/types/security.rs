use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Security capabilities requested by the plugin/environment.
/// These define what system resources the code can access.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Allow network outbound access to specific hosts.
    Network(Vec<String>),

    /// Allow read access to specific filesystem paths.
    FsRead(Vec<String>),

    /// Allow write access to specific filesystem paths.
    FsWrite(Vec<String>),

    /// Allow access to system devices (e.g., `/dev/ttyUSB0`).
    Device(Vec<String>),

    /// Allow interaction with the user (prompts, confirmation).
    UiInteract,

    /// Allow requesting secrets (masked input) from the user.
    UiSecret,

    /// Allow controlling specific background services (systemd, launchd).
    ServiceControl(Vec<String>),
}

pub fn deserialize_capability_list<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<Capability>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CapMapOrSeq {
        Seq(Vec<Capability>),
        Map(HashMap<String, serde_json::Value>),
    }

    match Option::<CapMapOrSeq>::deserialize(deserializer)? {
        None => Ok(None),
        Some(CapMapOrSeq::Seq(s)) => Ok(Some(s)),
        Some(CapMapOrSeq::Map(m)) => {
            let mut caps = Vec::new();
            for (k, v) in m {
                let cap = match k.as_str() {
                    "network" => {
                        let hosts = serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                        Capability::Network(hosts)
                    }
                    "fs-read" => {
                        let paths = serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                        Capability::FsRead(paths)
                    }
                    "fs-write" => {
                        let paths = serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                        Capability::FsWrite(paths)
                    }
                    "device" => {
                        let devices =
                            serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                        Capability::Device(devices)
                    }
                    "ui-interact" => {
                        if v.as_bool().unwrap_or(false) {
                            Capability::UiInteract
                        } else {
                            continue;
                        }
                    }
                    "ui-secret" => {
                        if v.as_bool().unwrap_or(false) {
                            Capability::UiSecret
                        } else {
                            continue;
                        }
                    }
                    "service-control" => {
                        let services =
                            serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                        Capability::ServiceControl(services)
                    }
                    _ => continue,
                };
                caps.push(cap);
            }
            Ok(Some(caps))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_capabilities_seq() {
        let json = r#"{
            "capabilities": [
                "ui-interact",
                {"network": ["github.com"]}
            ]
        }"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_capability_list")]
            capabilities: Option<Vec<Capability>>,
        }
        let t: Test = serde_json::from_str(json).unwrap();
        let caps = t.capabilities.unwrap();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&Capability::UiInteract));
        assert!(caps.contains(&Capability::Network(vec!["github.com".to_string()])));
    }

    #[test]
    fn test_deserialize_capabilities_map() {
        let json = r#"{
            "capabilities": {
                "ui-interact": true,
                "network": ["github.com"]
            }
        }"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_capability_list")]
            capabilities: Option<Vec<Capability>>,
        }
        let t: Test = serde_json::from_str(json).unwrap();
        let caps = t.capabilities.unwrap();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&Capability::UiInteract));
        assert!(caps.contains(&Capability::Network(vec!["github.com".to_string()])));
    }
}
