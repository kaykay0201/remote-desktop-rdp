pub mod client;
pub mod server;

use tokio::sync::mpsc;
use crate::protocol::ProtocolMessage;

#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    input_tx: mpsc::Sender<ProtocolMessage>,
}

impl ConnectionHandle {
    pub fn new(input_tx: mpsc::Sender<ProtocolMessage>) -> Self {
        Self { input_tx }
    }

    pub async fn send_input(&self, msg: ProtocolMessage) -> Result<(), String> {
        self.input_tx.send(msg).await.map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Listening { port: u16 },
    ClientConnected,
    Connected(ConnectionHandle),
    ClientDisconnected,
    Frame {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    },
    Error(String),
    Stopped,
}

pub enum NetworkCommand {
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_handle_creation() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let handle = ConnectionHandle::new(tx);
        let _ = format!("{handle:?}");
    }

    #[test]
    fn network_event_variants() {
        let _ = NetworkEvent::Listening { port: 9867 };
        let _ = NetworkEvent::ClientConnected;
        let _ = NetworkEvent::Error("test".to_string());
        let _ = NetworkEvent::Stopped;
    }

    #[test]
    fn default_port_value() {
        assert_eq!(crate::protocol::DEFAULT_PORT, 9867);
    }
}
