use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use super::ProtocolMessage;

pub struct MessageCodec;

impl Decoder for MessageCodec {
    type Item = ProtocolMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;

        if src.len() < 4 + len {
            src.reserve(4 + len - src.len());
            return Ok(None);
        }

        src.advance(4);
        let data = src.split_to(len);

        let (msg, _): (ProtocolMessage, usize) =
            bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        Ok(Some(msg))
    }
}

impl Encoder<ProtocolMessage> for MessageCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: ProtocolMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = bincode::serde::encode_to_vec(&item, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        dst.reserve(4 + encoded.len());
        dst.put_u32(encoded.len() as u32);
        dst.put_slice(&encoded);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{FrameData, MouseBtn, PROTOCOL_VERSION};

    fn roundtrip(msg: ProtocolMessage) -> ProtocolMessage {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::new();
        codec.encode(msg, &mut buf).unwrap();
        codec.decode(&mut buf).unwrap().unwrap()
    }

    #[test]
    fn roundtrip_hello() {
        let msg = ProtocolMessage::Hello {
            version: PROTOCOL_VERSION,
            screen_width: 1920,
            screen_height: 1080,
        };
        let decoded = roundtrip(msg);
        match decoded {
            ProtocolMessage::Hello { version, screen_width, screen_height } => {
                assert_eq!(version, PROTOCOL_VERSION);
                assert_eq!(screen_width, 1920);
                assert_eq!(screen_height, 1080);
            }
            _ => panic!("expected Hello"),
        }
    }

    #[test]
    fn roundtrip_frame() {
        let msg = ProtocolMessage::Frame(FrameData {
            width: 800,
            height: 600,
            jpeg_quality: 75,
            compressed_payload: vec![1, 2, 3, 4],
        });
        let decoded = roundtrip(msg);
        match decoded {
            ProtocolMessage::Frame(f) => {
                assert_eq!(f.width, 800);
                assert_eq!(f.height, 600);
                assert_eq!(f.jpeg_quality, 75);
                assert_eq!(f.compressed_payload, vec![1, 2, 3, 4]);
            }
            _ => panic!("expected Frame"),
        }
    }

    #[test]
    fn roundtrip_mouse_move() {
        let msg = ProtocolMessage::MouseMove { x: 100, y: 200 };
        let decoded = roundtrip(msg);
        match decoded {
            ProtocolMessage::MouseMove { x, y } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
            }
            _ => panic!("expected MouseMove"),
        }
    }

    #[test]
    fn roundtrip_mouse_button() {
        let msg = ProtocolMessage::MouseButton { button: MouseBtn::Right, pressed: true };
        let decoded = roundtrip(msg);
        match decoded {
            ProtocolMessage::MouseButton { button, pressed } => {
                assert_eq!(button, MouseBtn::Right);
                assert!(pressed);
            }
            _ => panic!("expected MouseButton"),
        }
    }

    #[test]
    fn roundtrip_mouse_scroll() {
        let msg = ProtocolMessage::MouseScroll { delta_x: -5, delta_y: 10 };
        let decoded = roundtrip(msg);
        match decoded {
            ProtocolMessage::MouseScroll { delta_x, delta_y } => {
                assert_eq!(delta_x, -5);
                assert_eq!(delta_y, 10);
            }
            _ => panic!("expected MouseScroll"),
        }
    }

    #[test]
    fn roundtrip_key_event() {
        let msg = ProtocolMessage::KeyEvent { keycode: 0x41, pressed: true };
        let decoded = roundtrip(msg);
        match decoded {
            ProtocolMessage::KeyEvent { keycode, pressed } => {
                assert_eq!(keycode, 0x41);
                assert!(pressed);
            }
            _ => panic!("expected KeyEvent"),
        }
    }

    #[test]
    fn roundtrip_ping_pong() {
        let ping = roundtrip(ProtocolMessage::Ping(12345));
        match ping {
            ProtocolMessage::Ping(ts) => assert_eq!(ts, 12345),
            _ => panic!("expected Ping"),
        }

        let pong = roundtrip(ProtocolMessage::Pong(67890));
        match pong {
            ProtocolMessage::Pong(ts) => assert_eq!(ts, 67890),
            _ => panic!("expected Pong"),
        }
    }

    #[test]
    fn roundtrip_disconnect() {
        let decoded = roundtrip(ProtocolMessage::Disconnect);
        assert!(matches!(decoded, ProtocolMessage::Disconnect));
    }

    #[test]
    fn partial_read_returns_none() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::new();

        let msg = ProtocolMessage::Hello {
            version: PROTOCOL_VERSION,
            screen_width: 1920,
            screen_height: 1080,
        };
        codec.encode(msg, &mut buf).unwrap();

        let full = buf.clone();
        let partial_len = full.len() / 2;

        let mut partial = BytesMut::from(&full[..partial_len]);
        assert!(codec.decode(&mut partial).unwrap().is_none());

        let mut too_short = BytesMut::from(&full[..2]);
        assert!(codec.decode(&mut too_short).unwrap().is_none());
    }

    #[test]
    fn multiple_messages() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::new();

        let msg1 = ProtocolMessage::Ping(111);
        let msg2 = ProtocolMessage::Disconnect;

        codec.encode(msg1, &mut buf).unwrap();
        codec.encode(msg2, &mut buf).unwrap();

        let decoded1 = codec.decode(&mut buf).unwrap().unwrap();
        let decoded2 = codec.decode(&mut buf).unwrap().unwrap();

        assert!(matches!(decoded1, ProtocolMessage::Ping(111)));
        assert!(matches!(decoded2, ProtocolMessage::Disconnect));
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }
}
