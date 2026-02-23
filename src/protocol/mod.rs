pub mod codec;
pub mod compress;

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_PORT: u16 = 9867;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub jpeg_quality: u8,
    pub compressed_payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseBtn {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Hello {
        version: u32,
        screen_width: u32,
        screen_height: u32,
    },
    Frame(FrameData),
    MouseMove {
        x: u16,
        y: u16,
    },
    MouseButton {
        button: MouseBtn,
        pressed: bool,
    },
    MouseScroll {
        delta_x: i16,
        delta_y: i16,
    },
    KeyEvent {
        keycode: u32,
        pressed: bool,
    },
    Ping(u64),
    Pong(u64),
    Disconnect,
}
