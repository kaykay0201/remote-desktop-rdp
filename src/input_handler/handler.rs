use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

use crate::protocol::ProtocolMessage;

pub struct InputHandler {
    enigo: Enigo,
}

impl InputHandler {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create Enigo: {e}"))?;
        Ok(Self { enigo })
    }

    pub fn apply(&mut self, msg: &ProtocolMessage) {
        match msg {
            ProtocolMessage::MouseMove { x, y } => {
                let _ = self.enigo.move_mouse(*x as i32, *y as i32, Coordinate::Abs);
            }
            ProtocolMessage::MouseButton { button, pressed } => {
                if let Some(btn) = protocol_btn_to_enigo(button) {
                    let dir = if *pressed {
                        Direction::Press
                    } else {
                        Direction::Release
                    };
                    let _ = self.enigo.button(btn, dir);
                }
            }
            ProtocolMessage::MouseScroll { delta_x, delta_y } => {
                if *delta_y != 0 {
                    let _ = self
                        .enigo
                        .scroll(*delta_y as i32, enigo::Axis::Vertical);
                }
                if *delta_x != 0 {
                    let _ = self
                        .enigo
                        .scroll(*delta_x as i32, enigo::Axis::Horizontal);
                }
            }
            ProtocolMessage::KeyEvent { keycode, pressed } => {
                if let Some(key) = scancode_to_enigo_key(*keycode) {
                    let dir = if *pressed {
                        Direction::Press
                    } else {
                        Direction::Release
                    };
                    let _ = self.enigo.key(key, dir);
                }
            }
            _ => {}
        }
    }
}

fn protocol_btn_to_enigo(btn: &crate::protocol::MouseBtn) -> Option<Button> {
    match btn {
        crate::protocol::MouseBtn::Left => Some(Button::Left),
        crate::protocol::MouseBtn::Right => Some(Button::Right),
        crate::protocol::MouseBtn::Middle => Some(Button::Middle),
    }
}

fn scancode_to_enigo_key(keycode: u32) -> Option<Key> {
    match keycode {
        0x01 => Some(Key::Escape),
        0x02 => Some(Key::Unicode('1')),
        0x03 => Some(Key::Unicode('2')),
        0x04 => Some(Key::Unicode('3')),
        0x05 => Some(Key::Unicode('4')),
        0x06 => Some(Key::Unicode('5')),
        0x07 => Some(Key::Unicode('6')),
        0x08 => Some(Key::Unicode('7')),
        0x09 => Some(Key::Unicode('8')),
        0x0A => Some(Key::Unicode('9')),
        0x0B => Some(Key::Unicode('0')),
        0x0C => Some(Key::Unicode('-')),
        0x0D => Some(Key::Unicode('=')),
        0x0E => Some(Key::Backspace),
        0x0F => Some(Key::Tab),
        0x10 => Some(Key::Unicode('q')),
        0x11 => Some(Key::Unicode('w')),
        0x12 => Some(Key::Unicode('e')),
        0x13 => Some(Key::Unicode('r')),
        0x14 => Some(Key::Unicode('t')),
        0x15 => Some(Key::Unicode('y')),
        0x16 => Some(Key::Unicode('u')),
        0x17 => Some(Key::Unicode('i')),
        0x18 => Some(Key::Unicode('o')),
        0x19 => Some(Key::Unicode('p')),
        0x1A => Some(Key::Unicode('[')),
        0x1B => Some(Key::Unicode(']')),
        0x1C => Some(Key::Return),
        0x1D => Some(Key::Control),
        0x1E => Some(Key::Unicode('a')),
        0x1F => Some(Key::Unicode('s')),
        0x20 => Some(Key::Unicode('d')),
        0x21 => Some(Key::Unicode('f')),
        0x22 => Some(Key::Unicode('g')),
        0x23 => Some(Key::Unicode('h')),
        0x24 => Some(Key::Unicode('j')),
        0x25 => Some(Key::Unicode('k')),
        0x26 => Some(Key::Unicode('l')),
        0x27 => Some(Key::Unicode(';')),
        0x28 => Some(Key::Unicode('\'')),
        0x29 => Some(Key::Unicode('`')),
        0x2A => Some(Key::Shift),
        0x2B => Some(Key::Unicode('\\')),
        0x2C => Some(Key::Unicode('z')),
        0x2D => Some(Key::Unicode('x')),
        0x2E => Some(Key::Unicode('c')),
        0x2F => Some(Key::Unicode('v')),
        0x30 => Some(Key::Unicode('b')),
        0x31 => Some(Key::Unicode('n')),
        0x32 => Some(Key::Unicode('m')),
        0x33 => Some(Key::Unicode(',')),
        0x34 => Some(Key::Unicode('.')),
        0x35 => Some(Key::Unicode('/')),
        0x38 => Some(Key::Alt),
        0x39 => Some(Key::Space),
        0x3A => Some(Key::CapsLock),
        0x3B => Some(Key::F1),
        0x3C => Some(Key::F2),
        0x3D => Some(Key::F3),
        0x3E => Some(Key::F4),
        0x3F => Some(Key::F5),
        0x40 => Some(Key::F6),
        0x41 => Some(Key::F7),
        0x42 => Some(Key::F8),
        0x43 => Some(Key::F9),
        0x44 => Some(Key::F10),
        0x45 => Some(Key::Numlock),
        0x46 => Some(Key::Scroll),
        0x57 => Some(Key::F11),
        0x58 => Some(Key::F12),
        0xE037 => Some(Key::PrintScr),
        0xE047 => Some(Key::Home),
        0xE048 => Some(Key::UpArrow),
        0xE049 => Some(Key::PageUp),
        0xE04B => Some(Key::LeftArrow),
        0xE04D => Some(Key::RightArrow),
        0xE04F => Some(Key::End),
        0xE050 => Some(Key::DownArrow),
        0xE051 => Some(Key::PageDown),
        0xE052 => Some(Key::Insert),
        0xE053 => Some(Key::Delete),
        0xE11D => Some(Key::Pause),
        _ => None,
    }
}
