use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill};

#[derive(Debug, Clone)]
pub enum HostMessage {
    CopyUrl,
    StopHosting,
}

#[derive(Debug, Clone)]
pub enum HostStatus {
    Starting,
    Active,
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
        let title = text("Host Mode").size(28);

        let status_text = match &self.status {
            HostStatus::Starting => text("Starting tunnel...").size(16),
            HostStatus::Active => text("Tunnel active").size(16),
            HostStatus::Error(e) => text(format!("Error: {e}")).size(16),
        };

        let url_display = if let Some(ref url) = self.tunnel_url {
            column![text(url.as_str()).size(18),]
        } else {
            column![text("Waiting for tunnel URL...").size(14),]
        };

        let copy_label = if self.copied { "Copied!" } else { "Copy URL" };

        let copy_button = if self.tunnel_url.is_some() {
            button(text(copy_label)).on_press(HostMessage::CopyUrl)
        } else {
            button(text(copy_label))
        };

        let stop_button = button(text("Stop Hosting"))
            .on_press(HostMessage::StopHosting)
            .padding(10);

        let buttons = row![copy_button, stop_button].spacing(10);

        let content = column![title, status_text, url_display, buttons]
            .spacing(20)
            .align_x(Center)
            .padding(30)
            .max_width(600);

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
