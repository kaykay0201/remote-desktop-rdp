pub mod connection;
pub mod input;
pub mod session;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;

#[derive(Debug, Clone)]
pub enum RdpEvent {
    Connected(RdpConnection),
    Frame {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    },
    StatusChanged(ConnectionStatus),
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    TlsUpgrade,
    Authenticating,
    Active,
}

#[derive(Debug, Clone)]
pub enum InputCommand {
    KeyPressed { scancode: u16 },
    KeyReleased { scancode: u16 },
    MouseMoved { x: u16, y: u16 },
    MouseButtonPressed(MouseButtonKind),
    MouseButtonReleased(MouseButtonKind),
    MouseWheel { vertical: bool, delta: i16 },
    Disconnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonKind {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone)]
pub struct RdpConnection {
    sender: mpsc::Sender<InputCommand>,
}

impl RdpConnection {
    pub fn new(sender: mpsc::Sender<InputCommand>) -> Self {
        Self { sender }
    }

    pub async fn send(&mut self, cmd: InputCommand) -> bool {
        self.sender.send(cmd).await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_button_kind_equality() {
        assert_eq!(MouseButtonKind::Left, MouseButtonKind::Left);
        assert_ne!(MouseButtonKind::Left, MouseButtonKind::Right);
    }

    #[test]
    fn connection_status_equality() {
        assert_eq!(ConnectionStatus::Connecting, ConnectionStatus::Connecting);
        assert_ne!(ConnectionStatus::Connecting, ConnectionStatus::Active);
    }

    #[test]
    fn rdp_connection_send() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = RdpConnection::new(tx);
        let _ = conn;
    }

    #[test]
    fn rdp_connection_clone() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = RdpConnection::new(tx);
        let _conn2 = conn.clone();
    }
}
