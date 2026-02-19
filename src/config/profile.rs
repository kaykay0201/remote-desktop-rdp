use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{RdpError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub hostname: String,
    pub username: String,
    #[serde(skip)]
    pub password: String,
    #[serde(default = "default_width")]
    pub width: u16,
    #[serde(default = "default_height")]
    pub height: u16,
    #[serde(default = "default_proxy_port")]
    pub proxy_port: u16,
}

fn default_width() -> u16 {
    1920
}

fn default_height() -> u16 {
    1080
}

fn default_proxy_port() -> u16 {
    3390
}

impl Default for ConnectionProfile {
    fn default() -> Self {
        Self {
            hostname: String::new(),
            username: String::new(),
            password: String::new(),
            width: default_width(),
            height: default_height(),
            proxy_port: default_proxy_port(),
        }
    }
}

impl ConnectionProfile {
    pub fn server_addr(&self) -> String {
        format!("localhost:{}", self.proxy_port)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| RdpError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let profile: Self =
            toml::from_str(&content).map_err(|e| RdpError::Config(e.to_string()))?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_values() {
        let profile = ConnectionProfile::default();
        assert_eq!(profile.width, 1920);
        assert_eq!(profile.height, 1080);
        assert_eq!(profile.proxy_port, 3390);
        assert!(profile.hostname.is_empty());
        assert!(profile.username.is_empty());
        assert!(profile.password.is_empty());
    }

    #[test]
    fn server_addr_format() {
        let mut profile = ConnectionProfile::default();
        profile.proxy_port = 5555;
        assert_eq!(profile.server_addr(), "localhost:5555");
    }

    #[test]
    fn serialize_excludes_password() {
        let mut profile = ConnectionProfile::default();
        profile.hostname = "test.example.com".to_string();
        profile.password = "secret123".to_string();
        let serialized = toml::to_string(&profile).unwrap();
        assert!(!serialized.contains("secret123"));
        assert!(serialized.contains("test.example.com"));
    }

    #[test]
    fn deserialize_round_trip() {
        let mut profile = ConnectionProfile::default();
        profile.hostname = "myhost".to_string();
        profile.username = "admin".to_string();
        profile.width = 1280;
        profile.height = 720;
        profile.proxy_port = 3391;

        let serialized = toml::to_string(&profile).unwrap();
        let deserialized: ConnectionProfile = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.hostname, "myhost");
        assert_eq!(deserialized.username, "admin");
        assert_eq!(deserialized.width, 1280);
        assert_eq!(deserialized.height, 720);
        assert_eq!(deserialized.proxy_port, 3391);
        assert!(deserialized.password.is_empty());
    }
}
