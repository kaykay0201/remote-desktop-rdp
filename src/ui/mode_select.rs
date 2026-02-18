use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill, Length};

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
        let title = text("Rust RDP").size(36);
        let subtitle = text("Choose a mode to get started").size(16);

        let update_banner: Element<'_, ModeSelectMessage> =
            if let Some(ref release) = self.available_update {
                button(
                    text(format!(
                        "Update {} available — click to update",
                        release.version
                    ))
                    .size(14),
                )
                .on_press(ModeSelectMessage::UpdateClicked)
                .padding(10)
                .into()
            } else {
                column![].into()
            };

        let connect_card = button(
            column![
                text("Connect to Remote").size(20),
                text("Join a remote machine via tunnel URL").size(13),
            ]
            .spacing(8)
            .align_x(Center)
            .padding(20),
        )
        .on_press(ModeSelectMessage::ConnectSelected)
        .width(Length::Fixed(250.0))
        .padding(10);

        let host_card = button(
            column![
                text("Host This Machine").size(20),
                text("Expose local RDP via Cloudflare tunnel").size(13),
            ]
            .spacing(8)
            .align_x(Center)
            .padding(20),
        )
        .on_press(ModeSelectMessage::HostSelected)
        .width(Length::Fixed(250.0))
        .padding(10);

        let cards = row![connect_card, host_card].spacing(30);

        let content = column![title, subtitle, update_banner, cards]
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
