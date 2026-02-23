use crate::protocol::MouseBtn;

pub fn iced_key_to_keycode(key: &iced::keyboard::Key) -> Option<u32> {
    match key {
        iced::keyboard::Key::Named(named) => named_key_to_keycode(named),
        iced::keyboard::Key::Character(c) => char_to_keycode(c.as_str()),
        iced::keyboard::Key::Unidentified => None,
    }
}

fn named_key_to_keycode(key: &iced::keyboard::key::Named) -> Option<u32> {
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

fn char_to_keycode(s: &str) -> Option<u32> {
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

pub fn mouse_button_to_protocol(btn: &iced::mouse::Button) -> Option<MouseBtn> {
    match btn {
        iced::mouse::Button::Left => Some(MouseBtn::Left),
        iced::mouse::Button::Right => Some(MouseBtn::Right),
        iced::mouse::Button::Middle => Some(MouseBtn::Middle),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::Named;
    use iced::keyboard::Key;

    #[test]
    fn keycode_enter() {
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::Enter)),
            Some(0x1C)
        );
    }

    #[test]
    fn keycode_escape() {
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::Escape)),
            Some(0x01)
        );
    }

    #[test]
    fn keycode_arrow_keys() {
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::ArrowUp)),
            Some(0xE048)
        );
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::ArrowDown)),
            Some(0xE050)
        );
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::ArrowLeft)),
            Some(0xE04B)
        );
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::ArrowRight)),
            Some(0xE04D)
        );
    }

    #[test]
    fn keycode_character_a() {
        let key = Key::Character("a".into());
        assert_eq!(iced_key_to_keycode(&key), Some(0x1E));
    }

    #[test]
    fn keycode_character_z() {
        let key = Key::Character("z".into());
        assert_eq!(iced_key_to_keycode(&key), Some(0x2C));
    }

    #[test]
    fn keycode_digit_0() {
        let key = Key::Character("0".into());
        assert_eq!(iced_key_to_keycode(&key), Some(0x0B));
    }

    #[test]
    fn keycode_digit_1() {
        let key = Key::Character("1".into());
        assert_eq!(iced_key_to_keycode(&key), Some(0x02));
    }

    #[test]
    fn keycode_unidentified_returns_none() {
        assert_eq!(iced_key_to_keycode(&Key::Unidentified), None);
    }

    #[test]
    fn keycode_f_keys() {
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::F1)),
            Some(0x3B)
        );
        assert_eq!(
            iced_key_to_keycode(&Key::Named(Named::F12)),
            Some(0x58)
        );
    }

    #[test]
    fn mouse_button_left() {
        assert_eq!(
            mouse_button_to_protocol(&iced::mouse::Button::Left),
            Some(MouseBtn::Left)
        );
    }

    #[test]
    fn mouse_button_right() {
        assert_eq!(
            mouse_button_to_protocol(&iced::mouse::Button::Right),
            Some(MouseBtn::Right)
        );
    }

    #[test]
    fn mouse_button_middle() {
        assert_eq!(
            mouse_button_to_protocol(&iced::mouse::Button::Middle),
            Some(MouseBtn::Middle)
        );
    }

    #[test]
    fn mouse_button_other_returns_none() {
        assert_eq!(
            mouse_button_to_protocol(&iced::mouse::Button::Other(4)),
            None
        );
    }
}
