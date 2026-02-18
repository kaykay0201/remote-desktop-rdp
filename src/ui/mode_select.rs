use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill, Length};

use crate::ui::theme::*;
use crate::updater::ReleaseInfo;

#[derive(Debug, Clone)]
pub enum ModeSelectMessage {
    ConnectSelected,
    HostSelected,
    UpdateClicked,
}

#[derive(Debug, Clone)]
pub struct ModeSelectState {
    pub available_update: Option<ReleaseInfo>,
}

impl ModeSelectState {
    pub fn new() -> Self {
        Self {
            available_update: None,
        }
    }

    pub fn view(&self) -> Element<'_, ModeSelectMessage> {
        let title = text("Rust RDP").size(40).color(TEXT_PRIMARY);
        let subtitle = text("Choose a mode to get started").size(16).color(TEXT_SECONDARY);

        let update_banner: Element<'_, ModeSelectMessage> =
            if let Some(ref release) = self.available_update {
                container(
                    button(
                        text(format!(
                            "Update {} available — click to update",
                            release.version
                        ))
                        .size(14)
                        .color(ACCENT_HOVER),
                    )
                    .on_press(ModeSelectMessage::UpdateClicked)
                    .style(secondary_button_style)
                    .padding(10),
                )
                .style(banner_container_style)
                .padding(4)
                .into()
            } else {
                column![].into()
            };

        let connect_card = button(
            column![
                text("Connect to Remote").size(20).color(TEXT_PRIMARY),
                text("Join a remote machine via tunnel URL").size(13).color(TEXT_SECONDARY),
            ]
            .spacing(8)
            .align_x(Center)
            .padding(24),
        )
        .on_press(ModeSelectMessage::ConnectSelected)
        .style(card_button_style)
        .width(Length::Fixed(260.0));

        let host_card = button(
            column![
                text("Host This Machine").size(20).color(TEXT_PRIMARY),
                text("Expose local RDP via Cloudflare tunnel").size(13).color(TEXT_SECONDARY),
            ]
            .spacing(8)
            .align_x(Center)
            .padding(24),
        )
        .on_press(ModeSelectMessage::HostSelected)
        .style(card_button_style)
        .width(Length::Fixed(260.0));

        let cards = row![connect_card, host_card].spacing(30);

        let version = text(format!("v{}", env!("CARGO_PKG_VERSION")))
            .size(12)
            .color(TEXT_MUTED);

        let content = column![title, subtitle, update_banner, cards, version]
            .spacing(24)
            .align_x(Center);

        container(content)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_select_state_default() {
        let _state = ModeSelectState::new();
    }
}
