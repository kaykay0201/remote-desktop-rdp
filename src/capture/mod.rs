pub mod capturer;
pub mod encoder;

use crate::protocol::FrameData;

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub fps: u32,
    pub jpeg_quality: u8,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            jpeg_quality: 75,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CaptureEvent {
    Started { width: u32, height: u32 },
    Frame(FrameData),
    Error(String),
    Stopped,
}
