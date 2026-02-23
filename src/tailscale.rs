use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct TailscaleStatus {
    pub is_installed: bool,
    pub is_running: bool,
    pub ip: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Deserialize)]
struct TailscaleStatusJson {
    #[serde(rename = "Self")]
    self_node: Option<SelfNode>,
}

#[derive(Deserialize)]
struct SelfNode {
    #[serde(rename = "TailscaleIPs")]
    tailscale_ips: Option<Vec<String>>,
    #[serde(rename = "HostName")]
    host_name: Option<String>,
}

pub async fn check_tailscale() -> TailscaleStatus {
    let version_check = tokio::process::Command::new("tailscale")
        .arg("version")
        .output()
        .await;

    let is_installed = matches!(version_check, Ok(ref o) if o.status.success());

    if !is_installed {
        return TailscaleStatus {
            is_installed: false,
            ..Default::default()
        };
    }

    let output = match tokio::process::Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .await
    {
        Ok(o) if o.status.success() => o.stdout,
        _ => {
            return TailscaleStatus {
                is_installed: true,
                is_running: false,
                ..Default::default()
            }
        }
    };

    let parsed: TailscaleStatusJson = match serde_json::from_slice(&output) {
        Ok(p) => p,
        Err(_) => {
            return TailscaleStatus {
                is_installed: true,
                is_running: false,
                ..Default::default()
            }
        }
    };

    match parsed.self_node {
        Some(node) => TailscaleStatus {
            is_installed: true,
            is_running: true,
            ip: node.tailscale_ips.and_then(|ips| ips.into_iter().next()),
            hostname: node.host_name,
        },
        None => TailscaleStatus {
            is_installed: true,
            is_running: true,
            ip: None,
            hostname: None,
        },
    }
}

pub fn open_install_page() {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "https://tailscale.com/download/windows"])
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_tailscale_json() {
        let json = r#"{
            "Self": {
                "TailscaleIPs": ["100.64.0.1", "fd7a:115c:a1e0::1"],
                "HostName": "my-machine"
            }
        }"#;
        let parsed: TailscaleStatusJson = serde_json::from_str(json).unwrap();
        let node = parsed.self_node.unwrap();
        assert_eq!(node.tailscale_ips.unwrap()[0], "100.64.0.1");
        assert_eq!(node.host_name.unwrap(), "my-machine");
    }

    #[test]
    fn parse_invalid_json_returns_default() {
        let result: Result<TailscaleStatusJson, _> = serde_json::from_str("not json");
        assert!(result.is_err());
    }

    #[test]
    fn default_status_is_not_running() {
        let status = TailscaleStatus::default();
        assert!(!status.is_installed);
        assert!(!status.is_running);
        assert!(status.ip.is_none());
        assert!(status.hostname.is_none());
    }

    #[test]
    fn status_with_installed_not_running() {
        let status = TailscaleStatus {
            is_installed: true,
            is_running: false,
            ..Default::default()
        };
        assert!(status.is_installed);
        assert!(!status.is_running);
        assert!(status.ip.is_none());
    }

    #[test]
    fn status_fully_connected() {
        let status = TailscaleStatus {
            is_installed: true,
            is_running: true,
            ip: Some("100.64.0.1".to_string()),
            hostname: Some("my-pc".to_string()),
        };
        assert!(status.is_installed);
        assert!(status.is_running);
        assert_eq!(status.ip.as_deref(), Some("100.64.0.1"));
    }
}
