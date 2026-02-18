use ironrdp::input::{Database, MouseButton, MousePosition, Operation, Scancode, WheelRotations};
use ironrdp::pdu::input::fast_path::FastPathInputEvent;
use smallvec::SmallVec;

use crate::rdp::{InputCommand, MouseButtonKind};

pub fn translate_command(
    db: &mut Database,
    cmd: InputCommand,
) -> SmallVec<[FastPathInputEvent; 2]> {
    let ops: Vec<Operation> = match cmd {
        InputCommand::KeyPressed { scancode } => {
            vec![Operation::KeyPressed(Scancode::from_u16(scancode))]
        }
        InputCommand::KeyReleased { scancode } => {
            vec![Operation::KeyReleased(Scancode::from_u16(scancode))]
        }
        InputCommand::MouseMoved { x, y } => {
            vec![Operation::MouseMove(MousePosition { x, y })]
        }
        InputCommand::MouseButtonPressed(kind) => {
            vec![Operation::MouseButtonPressed(convert_mouse_button(kind))]
        }
        InputCommand::MouseButtonReleased(kind) => {
            vec![Operation::MouseButtonReleased(convert_mouse_button(kind))]
        }
        InputCommand::MouseWheel { vertical, delta } => {
            vec![Operation::WheelRotations(WheelRotations {
                is_vertical: vertical,
                rotation_units: delta,
            })]
        }
        InputCommand::Disconnect => return SmallVec::new(),
    };
    db.apply(ops)
}

fn convert_mouse_button(kind: MouseButtonKind) -> MouseButton {
    match kind {
        MouseButtonKind::Left => MouseButton::Left,
        MouseButtonKind::Middle => MouseButton::Middle,
        MouseButtonKind::Right => MouseButton::Right,
    }
}

pub fn iced_key_to_scancode(key: &iced::keyboard::Key) -> Option<u16> {
    match key {
        iced::keyboard::Key::Named(named) => named_key_to_scancode(named),
        iced::keyboard::Key::Character(c) => char_to_scancode(c.as_str()),
        iced::keyboard::Key::Unidentified => None,
    }
}

fn named_key_to_scancode(key: &iced::keyboard::key::Named) -> Option<u16> {
    use iced::keyboard::key::Named;
    let code = match key {
        Named::Escape => 0x01,
        Named::F1 => 0x3B,
        Named::F2 => 0x3C,
        Named::F3 => 0x3D,
        Named::F4 => 0x3E,
        Named::F5 => 0x3F,
        Named::F6 => 0x40,
        Named::F7 => 0x41,
        Named::F8 => 0x42,
        Named::F9 => 0x43,
        Named::F10 => 0x44,
        Named::F11 => 0x57,
        Named::F12 => 0x58,
        Named::Backspace => 0x0E,
        Named::Tab => 0x0F,
        Named::Enter => 0x1C,
        Named::Shift => 0x2A,
        Named::Control => 0x1D,
        Named::Alt => 0x38,
        Named::CapsLock => 0x3A,
        Named::Space => 0x39,
        Named::PageUp => 0xE049,
        Named::PageDown => 0xE051,
        Named::End => 0xE04F,
        Named::Home => 0xE047,
        Named::ArrowLeft => 0xE04B,
        Named::ArrowUp => 0xE048,
        Named::ArrowRight => 0xE04D,
        Named::ArrowDown => 0xE050,
        Named::Insert => 0xE052,
        Named::Delete => 0xE053,
        Named::NumLock => 0x45,
        Named::ScrollLock => 0x46,
        Named::PrintScreen => 0xE037,
        Named::Pause => 0xE11D,
        _ => return None,
    };
    Some(code)
}

fn char_to_scancode(s: &str) -> Option<u16> {
    if s.len() != 1 {
        return None;
    }
    let ch = s.chars().next()?;
    let code = match ch.to_ascii_lowercase() {
        'a' => 0x1E,
        'b' => 0x30,
        'c' => 0x2E,
        'd' => 0x20,
        'e' => 0x12,
        'f' => 0x21,
        'g' => 0x22,
        'h' => 0x23,
        'i' => 0x17,
        'j' => 0x24,
        'k' => 0x25,
        'l' => 0x26,
        'm' => 0x32,
        'n' => 0x31,
        'o' => 0x18,
        'p' => 0x19,
        'q' => 0x10,
        'r' => 0x13,
        's' => 0x1F,
        't' => 0x14,
        'u' => 0x16,
        'v' => 0x2F,
        'w' => 0x11,
        'x' => 0x2D,
        'y' => 0x15,
        'z' => 0x2C,
        '1' => 0x02,
        '2' => 0x03,
        '3' => 0x04,
        '4' => 0x05,
        '5' => 0x06,
        '6' => 0x07,
        '7' => 0x08,
        '8' => 0x09,
        '9' => 0x0A,
        '0' => 0x0B,
        '-' => 0x0C,
        '=' => 0x0D,
        '[' => 0x1A,
        ']' => 0x1B,
        '\\' => 0x2B,
        ';' => 0x27,
        '\'' => 0x28,
        '`' => 0x29,
        ',' => 0x33,
        '.' => 0x34,
        '/' => 0x35,
        _ => return None,
    };
    Some(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::Key;
    use iced::keyboard::key::Named;

    #[test]
    fn scancode_enter() {
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::Enter)), Some(0x1C));
    }

    #[test]
    fn scancode_escape() {
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::Escape)), Some(0x01));
    }

    #[test]
    fn scancode_arrow_keys() {
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::ArrowUp)), Some(0xE048));
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::ArrowDown)), Some(0xE050));
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::ArrowLeft)), Some(0xE04B));
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::ArrowRight)), Some(0xE04D));
    }

    #[test]
    fn scancode_character_a() {
        let key = Key::Character("a".into());
        assert_eq!(iced_key_to_scancode(&key), Some(0x1E));
    }

    #[test]
    fn scancode_character_z() {
        let key = Key::Character("z".into());
        assert_eq!(iced_key_to_scancode(&key), Some(0x2C));
    }

    #[test]
    fn scancode_digit_0() {
        let key = Key::Character("0".into());
        assert_eq!(iced_key_to_scancode(&key), Some(0x0B));
    }

    #[test]
    fn scancode_digit_1() {
        let key = Key::Character("1".into());
        assert_eq!(iced_key_to_scancode(&key), Some(0x02));
    }

    #[test]
    fn scancode_unidentified_returns_none() {
        assert_eq!(iced_key_to_scancode(&Key::Unidentified), None);
    }

    #[test]
    fn scancode_f_keys() {
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::F1)), Some(0x3B));
        assert_eq!(iced_key_to_scancode(&Key::Named(Named::F12)), Some(0x58));
    }

    #[test]
    fn translate_disconnect_returns_empty() {
        let mut db = Database::new();
        let result = translate_command(&mut db, InputCommand::Disconnect);
        assert!(result.is_empty());
    }

    #[test]
    fn translate_mouse_move() {
        let mut db = Database::new();
        let result = translate_command(&mut db, InputCommand::MouseMoved { x: 100, y: 200 });
        assert!(!result.is_empty());
    }

    #[test]
    fn translate_key_press() {
        let mut db = Database::new();
        let result = translate_command(&mut db, InputCommand::KeyPressed { scancode: 0x1E });
        assert!(!result.is_empty());
    }

    #[test]
    fn translate_mouse_button() {
        let mut db = Database::new();
        let result = translate_command(&mut db, InputCommand::MouseButtonPressed(MouseButtonKind::Left));
        assert!(!result.is_empty());
    }
}
