use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill};

use crate::ui::theme::*;

#[derive(Debug, Clone)]
pub enum HostMessage {
    CopyUrl,
    StopHosting,
}

#[derive(Debug, Clone)]
pub enum HostStatus {
    Starting,
    Active,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct HostState {
    pub tunnel_url: Option<String>,
    pub status: HostStatus,
    pub copied: bool,
}

impl HostState {
    pub fn new() -> Self {
        Self {
            tunnel_url: None,
            status: HostStatus::Starting,
            copied: false,
        }
    }

    pub fn view(&self) -> Element<'_, HostMessage> {
        let title = text("Host Mode").size(28).color(TEXT_PRIMARY);

        let stopping = matches!(self.status, HostStatus::Stopping);

        let status_text = match &self.status {
            HostStatus::Starting => text("Starting tunnel...").size(16).color(TEXT_SECONDARY),
            HostStatus::Active => text("Tunnel active").size(16).color(SUCCESS),
            HostStatus::Stopping => text("Stopping tunnel...").size(16).color(TEXT_SECONDARY),
            HostStatus::Error(e) => text(format!("Error: {e}")).size(16).color(DANGER),
        };

        let url_display: Element<'_, HostMessage> = if let Some(ref url) = self.tunnel_url {
            container(
                text(url.as_str()).size(16).color(ACCENT_HOVER),
            )
            .style(url_container_style)
            .padding([8, 16])
            .into()
        } else {
            text("Waiting for tunnel URL...").size(14).color(TEXT_MUTED).into()
        };

        let copy_label = if self.copied { "Copied!" } else { "Copy URL" };

        let copy_button = if self.tunnel_url.is_some() && !stopping {
            button(text(copy_label))
                .on_press(HostMessage::CopyUrl)
                .style(primary_button_style)
                .padding([10, 20])
        } else {
            button(text(copy_label))
                .style(primary_button_style)
                .padding([10, 20])
        };

        let mut stop_button = button(text("Stop Hosting"))
            .style(danger_button_style)
            .padding([10, 20]);
        if matches!(self.status, HostStatus::Active) {
            stop_button = stop_button.on_press(HostMessage::StopHosting);
        }

        let buttons = row![copy_button, stop_button].spacing(10);

        let inner = column![title, status_text, url_display, buttons]
            .spacing(20)
            .align_x(Center);

        let card = container(inner)
            .style(card_container_style)
            .padding(36)
            .max_width(600);

        container(card)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_state_default() {
        let state = HostState::new();
        assert!(state.tunnel_url.is_none());
        assert!(!state.copied);
        assert!(matches!(state.status, HostStatus::Starting));
    }

    #[test]
    fn host_state_with_url() {
        let mut state = HostState::new();
        state.tunnel_url = Some("https://test.trycloudflare.com".to_string());
        state.status = HostStatus::Active;
        assert!(state.tunnel_url.is_some());
        assert!(matches!(state.status, HostStatus::Active));
    }

    #[test]
    fn host_state_error() {
        let mut state = HostState::new();
        state.status = HostStatus::Error("test error".to_string());
        assert!(matches!(state.status, HostStatus::Error(_)));
    }
}
