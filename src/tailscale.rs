use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct TailscaleStatus {
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
    let output = match tokio::process::Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .await
    {
        Ok(o) if o.status.success() => o.stdout,
        _ => return TailscaleStatus::default(),
    };

    let parsed: TailscaleStatusJson = match serde_json::from_slice(&output) {
        Ok(p) => p,
        Err(_) => return TailscaleStatus::default(),
    };

    match parsed.self_node {
        Some(node) => TailscaleStatus {
            is_running: true,
            ip: node.tailscale_ips.and_then(|ips| ips.into_iter().next()),
            hostname: node.host_name,
        },
        None => TailscaleStatus {
            is_running: true,
            ip: None,
            hostname: None,
        },
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
        assert!(!status.is_running);
        assert!(status.ip.is_none());
        assert!(status.hostname.is_none());
    }
}
