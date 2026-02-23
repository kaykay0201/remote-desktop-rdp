use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{AppError, Result};
use crate::protocol::DEFAULT_PORT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub host_ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub display_name: String,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

impl Default for ConnectionProfile {
    fn default() -> Self {
        Self {
            host_ip: String::new(),
            port: default_port(),
            display_name: String::new(),
        }
    }
}

impl ConnectionProfile {
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host_ip, self.port)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let profile: Self =
            toml::from_str(&content).map_err(|e| AppError::Config(e.to_string()))?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_values() {
        let profile = ConnectionProfile::default();
        assert_eq!(profile.port, DEFAULT_PORT);
        assert!(profile.host_ip.is_empty());
        assert!(profile.display_name.is_empty());
    }

    #[test]
    fn server_addr_format() {
        let mut profile = ConnectionProfile::default();
        profile.host_ip = "100.64.0.1".to_string();
        profile.port = 9867;
        assert_eq!(profile.server_addr(), "100.64.0.1:9867");
    }

    #[test]
    fn serialize_round_trip() {
        let mut profile = ConnectionProfile::default();
        profile.host_ip = "100.64.0.1".to_string();
        profile.display_name = "My PC".to_string();
        profile.port = 9867;

        let serialized = toml::to_string(&profile).unwrap();
        let deserialized: ConnectionProfile = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.host_ip, "100.64.0.1");
        assert_eq!(deserialized.display_name, "My PC");
        assert_eq!(deserialized.port, 9867);
    }

    #[test]
    fn deserialize_with_defaults() {
        let toml_str = r#"host_ip = "10.0.0.1""#;
        let profile: ConnectionProfile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.host_ip, "10.0.0.1");
        assert_eq!(profile.port, DEFAULT_PORT);
        assert!(profile.display_name.is_empty());
    }
}
