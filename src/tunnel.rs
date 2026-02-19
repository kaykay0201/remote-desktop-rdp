use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::pin::Pin;

use futures::Stream;
use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::StreamExt;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub enum TunnelEvent {
    HandleReady(TunnelHandle),
    UrlReady(String),
    Output(String),
    Error(String),
    Stopped,
}

#[derive(Debug, Clone)]
pub enum TunnelCommand {
    Stop,
}

#[derive(Debug, Clone)]
pub struct TunnelHandle {
    sender: mpsc::Sender<TunnelCommand>,
}

impl TunnelHandle {
    pub async fn stop(&mut self) {
        let _ = self.sender.send(TunnelCommand::Stop).await;
    }
}

#[derive(Clone)]
pub struct HostTunnelKey {
    pub cloudflared_path: PathBuf,
}

impl Hash for HostTunnelKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "host-tunnel".hash(state);
    }
}

#[derive(Clone)]
pub struct ClientTunnelKey {
    pub tunnel_url: String,
    pub local_port: u16,
    pub cloudflared_path: PathBuf,
}

impl Hash for ClientTunnelKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "client-tunnel".hash(state);
        self.tunnel_url.hash(state);
        self.local_port.hash(state);
    }
}

pub fn extract_tunnel_url(line: &str) -> Option<String> {
    let start = line.find("https://")?;
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '>' || c == '|')
        .unwrap_or(rest.len());
    let url = &rest[..end];
    if url.contains("trycloudflare.com") {
        Some(url.to_string())
    } else {
        None
    }
}

pub fn host_tunnel_subscription(
    key: &HostTunnelKey,
) -> Pin<Box<dyn Stream<Item = TunnelEvent> + Send>> {
    let cloudflared_path = key.cloudflared_path.clone();
    Box::pin(iced::stream::channel(100, async move |mut output| {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<TunnelCommand>(10);
        let _ = output
            .send(TunnelEvent::HandleReady(TunnelHandle { sender: cmd_tx }))
            .await;

        let mut cmd = Command::new(&cloudflared_path);
        cmd.args(["tunnel", "--url", "tcp://localhost:3389"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = output
                    .send(TunnelEvent::Error(format!(
                        "Failed to start cloudflared: {e}"
                    )))
                    .await;
                return;
            }
        };
        #[cfg(windows)]
        {
            if let Some(handle) = child.raw_handle() {
                crate::process::assign_child_to_job(handle);
            }
        }

        let stderr = child.stderr.take().unwrap();
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();

        let mut url_found = false;

        loop {
            tokio::select! {
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(ref line)) => {
                            info!("cloudflared: {}", line);
                            if !url_found
                                && let Some(url) = extract_tunnel_url(line)
                            {
                                url_found = true;
                                let _ = output.send(TunnelEvent::UrlReady(url)).await;
                            }
                            let _ = output.send(TunnelEvent::Output(line.clone())).await;
                        }
                        Ok(None) => {
                            info!("cloudflared stderr closed");
                            break;
                        }
                        Err(e) => {
                            error!("cloudflared read error: {e}");
                            let _ = output.send(TunnelEvent::Error(format!("Read error: {e}"))).await;
                            break;
                        }
                    }
                }
                cmd = cmd_rx.next() => {
                    match cmd {
                        Some(TunnelCommand::Stop) | None => {
                            info!("Stopping host tunnel");
                            let _ = child.kill().await;
                            break;
                        }
                    }
                }
            }
        }

        let _ = output.send(TunnelEvent::Stopped).await;
    }))
}

pub fn client_tunnel_subscription(
    key: &ClientTunnelKey,
) -> Pin<Box<dyn Stream<Item = TunnelEvent> + Send>> {
    let tunnel_url = key.tunnel_url.clone();
    let local_port = key.local_port;
    let cloudflared_path = key.cloudflared_path.clone();

    Box::pin(iced::stream::channel(100, async move |mut output| {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<TunnelCommand>(10);
        let _ = output
            .send(TunnelEvent::HandleReady(TunnelHandle { sender: cmd_tx }))
            .await;

        let local_url = format!("localhost:{local_port}");
        let mut cmd = Command::new(&cloudflared_path);
        cmd.args(["access", "tcp", "--hostname", &tunnel_url, "--url", &local_url])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = output
                    .send(TunnelEvent::Error(format!(
                        "Failed to start cloudflared: {e}"
                    )))
                    .await;
                return;
            }
        };
        #[cfg(windows)]
        {
            if let Some(handle) = child.raw_handle() {
                crate::process::assign_child_to_job(handle);
            }
        }

        let stderr = child.stderr.take().unwrap();
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();

        loop {
            tokio::select! {
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(ref line)) => {
                            info!("cloudflared client: {}", line);
                            if line.contains(" ERR ") || line.contains("\"level\":\"error\"") || line.contains("\"level\":\"fatal\"") {
                                let _ = output.send(TunnelEvent::Error(line.clone())).await;
                            } else {
                                let _ = output.send(TunnelEvent::Output(line.clone())).await;
                            }
                        }
                        Ok(None) => {
                            info!("cloudflared client stderr closed");
                            break;
                        }
                        Err(e) => {
                            error!("cloudflared client read error: {e}");
                            let _ = output.send(TunnelEvent::Error(format!("Read error: {e}"))).await;
                            break;
                        }
                    }
                }
                cmd = cmd_rx.next() => {
                    match cmd {
                        Some(TunnelCommand::Stop) | None => {
                            info!("Stopping client tunnel");
                            let _ = child.kill().await;
                            break;
                        }
                    }
                }
            }
        }

        let _ = output.send(TunnelEvent::Stopped).await;
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_url_from_typical_line() {
        let line =
            "2024-01-15T10:00:00Z INF +-------------------------------------------+";
        assert!(extract_tunnel_url(line).is_none());

        let line = "2024-01-15T10:00:00Z INF |  https://foo-bar-baz.trycloudflare.com  |";
        let url = extract_tunnel_url(line).unwrap();
        assert_eq!(url, "https://foo-bar-baz.trycloudflare.com");
    }

    #[test]
    fn extract_url_from_json_log() {
        let line = r#"{"level":"info","url":"https://abc-123.trycloudflare.com","time":"2024"}"#;
        let url = extract_tunnel_url(line).unwrap();
        assert_eq!(url, "https://abc-123.trycloudflare.com");
    }

    #[test]
    fn no_url_in_line() {
        assert!(extract_tunnel_url("Starting tunnel...").is_none());
        assert!(extract_tunnel_url("").is_none());
        assert!(extract_tunnel_url("https://example.com").is_none());
    }

    #[test]
    fn url_at_end_of_line() {
        let line = "Tunnel URL: https://my-tunnel.trycloudflare.com";
        let url = extract_tunnel_url(line).unwrap();
        assert_eq!(url, "https://my-tunnel.trycloudflare.com");
    }

    #[test]
    fn host_tunnel_key_hash_stable() {
        use std::collections::hash_map::DefaultHasher;

        let key1 = HostTunnelKey {
            cloudflared_path: PathBuf::from("cloudflared"),
        };
        let key2 = HostTunnelKey {
            cloudflared_path: PathBuf::from("cloudflared"),
        };

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        key1.hash(&mut h1);
        key2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn client_tunnel_key_hash_differs_by_url() {
        use std::collections::hash_map::DefaultHasher;

        let key1 = ClientTunnelKey {
            tunnel_url: "https://a.trycloudflare.com".to_string(),
            local_port: 13389,
            cloudflared_path: PathBuf::from("cloudflared"),
        };
        let key2 = ClientTunnelKey {
            tunnel_url: "https://b.trycloudflare.com".to_string(),
            local_port: 13389,
            cloudflared_path: PathBuf::from("cloudflared"),
        };

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        key1.hash(&mut h1);
        key2.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }
}
