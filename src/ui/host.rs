use std::time::Instant;

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

pub struct HostState {
    pub tunnel_url: Option<String>,
    pub status: HostStatus,
    pub copied: bool,
    pub client_addr: Option<String>,
    pub connected_since: Option<Instant>,
}

impl HostState {
    pub fn new() -> Self {
        Self {
            tunnel_url: None,
            status: HostStatus::Starting,
            copied: false,
            client_addr: None,
            connected_since: None,
        }
    }

    pub fn view(&self) -> Element<'_, HostMessage> {
        let title = text("Host Mode").size(28).color(TEXT_PRIMARY);

        let stopping = matches!(self.status, HostStatus::Stopping);

        let status_text = match &self.status {
            HostStatus::Starting => text("Starting server...").size(16).color(TEXT_SECONDARY),
            HostStatus::Active => text("Server active — accepting connections").size(16).color(SUCCESS),
            HostStatus::Stopping => text("Stopping server...").size(16).color(TEXT_SECONDARY),
            HostStatus::Error(e) => text(format!("Error: {e}")).size(16).color(DANGER),
        };

        let url_display: Element<'_, HostMessage> = if let Some(ref addr) = self.tunnel_url {
            container(
                text(addr.as_str()).size(16).color(ACCENT_HOVER),
            )
            .style(url_container_style)
            .padding([8, 16])
            .into()
        } else {
            text("Waiting for server to start...").size(14).color(TEXT_MUTED).into()
        };

        let client_info: Element<'_, HostMessage> = if let Some(ref addr) = self.client_addr {
            let duration_text = if let Some(since) = self.connected_since {
                let elapsed = since.elapsed();
                let secs = elapsed.as_secs();
                if secs >= 60 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{secs}s")
                }
            } else {
                "0s".to_string()
            };

            column![
                text(format!("Client connected: {addr}")).size(14).color(TEXT_SECONDARY),
                text(format!("Connected for: {duration_text}")).size(14).color(TEXT_SECONDARY),
            ]
            .spacing(4)
            .into()
        } else {
            text("No client connected").size(14).color(TEXT_MUTED).into()
        };

        let copy_label = if self.copied { "Copied!" } else { "Copy Address" };

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

        let inner = column![title, status_text, url_display, client_info, buttons]
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
        assert!(state.client_addr.is_none());
        assert!(state.connected_since.is_none());
    }

    #[test]
    fn host_state_with_address() {
        let mut state = HostState::new();
        state.tunnel_url = Some("100.64.0.1:9867".to_string());
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

    #[test]
    fn host_state_with_client() {
        let mut state = HostState::new();
        state.client_addr = Some("100.64.0.1:12345".to_string());
        state.connected_since = Some(Instant::now());
        assert!(state.client_addr.is_some());
        assert!(state.connected_since.is_some());
    }
}
