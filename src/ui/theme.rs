use iced::widget::{button, container, progress_bar, text_input};
use iced::{color, Border, Color, Shadow, Theme, Vector};
use iced::theme::Palette;

pub const BG_DARK: Color = color!(0x0F172A);
pub const CARD_BG: Color = color!(0x1E293B);
pub const CARD_BG_HOVER: Color = color!(0x263548);
pub const BORDER_SUBTLE: Color = color!(0x334155);
pub const BORDER_INPUT: Color = color!(0x475569);

pub const TEXT_PRIMARY: Color = color!(0xF1F5F9);
pub const TEXT_SECONDARY: Color = color!(0x94A3B8);
pub const TEXT_MUTED: Color = color!(0x64748B);

pub const ACCENT: Color = color!(0x3B82F6);
pub const ACCENT_HOVER: Color = color!(0x60A5FA);
pub const ACCENT_PRESSED: Color = color!(0x2563EB);

pub const SUCCESS: Color = color!(0x22C55E);
pub const DANGER: Color = color!(0xEF4444);
pub const DANGER_HOVER: Color = color!(0xF87171);

pub fn app_theme() -> Theme {
    Theme::custom("Rust RDP".to_string(), Palette {
        background: BG_DARK,
        text: TEXT_PRIMARY,
        primary: ACCENT,
        success: SUCCESS,
        danger: DANGER,
        warning: color!(0xEAB308),
    })
}

fn btn(bg: Color, text_color: Color, radius: f32, shadow: Shadow) -> button::Style {
    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border { radius: radius.into(), ..Border::default() },
        shadow,
        snap: false,
    }
}

fn btn_bordered(bg: Color, text_color: Color, radius: f32, border_color: Color, shadow: Shadow) -> button::Style {
    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border { radius: radius.into(), width: 1.0, color: border_color },
        shadow,
        snap: false,
    }
}

fn soft_shadow(alpha: f32, y: f32, blur: f32) -> Shadow {
    Shadow {
        color: Color { a: alpha, ..Color::BLACK },
        offset: Vector::new(0.0, y),
        blur_radius: blur,
    }
}

fn ctr(bg: Option<Color>, border: Border, shadow: Shadow) -> container::Style {
    container::Style {
        background: bg.map(|c| c.into()),
        border,
        shadow,
        text_color: None,
        snap: false,
    }
}

pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => btn(ACCENT, Color::WHITE, 8.0, soft_shadow(0.2, 2.0, 4.0)),
        button::Status::Hovered => btn(ACCENT_HOVER, Color::WHITE, 8.0, soft_shadow(0.3, 4.0, 8.0)),
        button::Status::Pressed => btn(ACCENT_PRESSED, Color::WHITE, 8.0, soft_shadow(0.1, 1.0, 2.0)),
        button::Status::Disabled => btn(Color { a: 0.4, ..ACCENT }, Color { a: 0.5, ..Color::WHITE }, 8.0, soft_shadow(0.2, 2.0, 4.0)),
    }
}

pub fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => btn_bordered(Color::TRANSPARENT, TEXT_SECONDARY, 8.0, BORDER_SUBTLE, Shadow::default()),
        button::Status::Hovered => btn_bordered(CARD_BG, TEXT_PRIMARY, 8.0, ACCENT, Shadow::default()),
        button::Status::Pressed => btn_bordered(CARD_BG_HOVER, TEXT_PRIMARY, 8.0, ACCENT_PRESSED, Shadow::default()),
        button::Status::Disabled => btn_bordered(Color::TRANSPARENT, TEXT_MUTED, 8.0, Color { a: 0.3, ..BORDER_SUBTLE }, Shadow::default()),
    }
}

pub fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => btn(DANGER, Color::WHITE, 8.0, soft_shadow(0.2, 2.0, 4.0)),
        button::Status::Hovered => btn(DANGER_HOVER, Color::WHITE, 8.0, soft_shadow(0.3, 4.0, 8.0)),
        button::Status::Pressed => btn(color!(0xD13030), Color::WHITE, 8.0, soft_shadow(0.1, 1.0, 2.0)),
        button::Status::Disabled => btn(Color { a: 0.4, ..DANGER }, Color { a: 0.5, ..Color::WHITE }, 8.0, soft_shadow(0.2, 2.0, 4.0)),
    }
}

pub fn card_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => btn_bordered(CARD_BG, TEXT_PRIMARY, 12.0, BORDER_SUBTLE, soft_shadow(0.15, 2.0, 6.0)),
        button::Status::Hovered => btn_bordered(CARD_BG_HOVER, TEXT_PRIMARY, 12.0, ACCENT, soft_shadow(0.25, 6.0, 12.0)),
        button::Status::Pressed => btn_bordered(CARD_BG, TEXT_PRIMARY, 12.0, ACCENT_PRESSED, soft_shadow(0.1, 1.0, 2.0)),
        button::Status::Disabled => btn_bordered(Color { a: 0.5, ..CARD_BG }, TEXT_MUTED, 12.0, BORDER_SUBTLE, soft_shadow(0.15, 2.0, 6.0)),
    }
}

pub fn card_container_style(_theme: &Theme) -> container::Style {
    ctr(
        Some(CARD_BG),
        Border { radius: 12.0.into(), width: 1.0, color: BORDER_SUBTLE },
        soft_shadow(0.2, 4.0, 12.0),
    )
}

pub fn toolbar_container_style(_theme: &Theme) -> container::Style {
    ctr(
        Some(CARD_BG),
        Border::default(),
        soft_shadow(0.3, 2.0, 6.0),
    )
}

pub fn banner_container_style(_theme: &Theme) -> container::Style {
    ctr(
        Some(Color { a: 0.15, ..ACCENT }),
        Border { radius: 8.0.into(), width: 1.0, color: ACCENT },
        Shadow::default(),
    )
}

pub fn url_container_style(_theme: &Theme) -> container::Style {
    ctr(
        Some(BG_DARK),
        Border { radius: 6.0.into(), width: 1.0, color: BORDER_SUBTLE },
        Shadow::default(),
    )
}

pub fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: BG_DARK.into(),
        border: Border { radius: 6.0.into(), width: 1.0, color: BORDER_INPUT },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: Color { a: 0.3, ..ACCENT },
    };
    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border { color: TEXT_SECONDARY, ..base.border },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border { color: ACCENT, width: 2.0, ..base.border },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Color { a: 0.5, ..BG_DARK }.into(),
            value: TEXT_MUTED,
            ..base
        },
    }
}

pub fn progress_bar_style(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: CARD_BG.into(),
        bar: ACCENT.into(),
        border: Border { radius: 4.0.into(), ..Border::default() },
    }
}
