use iced::widget::{button, column, container, row, text, text_input};
use iced::{Center, Element, Fill};

use crate::config::ConnectionProfile;
use crate::protocol::DEFAULT_PORT;
use crate::ui::theme::*;

#[derive(Debug, Clone)]
pub enum LoginMessage {
    HostIpChanged(String),
    PortChanged(String),
    DisplayNameChanged(String),
    Connect,
    BackToModeSelect,
}

#[derive(Debug, Clone, Default)]
pub struct LoginState {
    pub host_ip: String,
    pub port: String,
    pub display_name: String,
}

impl LoginState {
    pub fn new() -> Self {
        Self {
            host_ip: String::new(),
            port: DEFAULT_PORT.to_string(),
            display_name: String::new(),
        }
    }

    pub fn update(&mut self, msg: LoginMessage) -> Option<ConnectionProfile> {
        match msg {
            LoginMessage::HostIpChanged(s) => self.host_ip = s,
            LoginMessage::PortChanged(s) => self.port = s,
            LoginMessage::DisplayNameChanged(s) => self.display_name = s,
            LoginMessage::Connect => {
                if self.host_ip.is_empty() {
                    return None;
                }
                let port = self.port.parse::<u16>().unwrap_or(DEFAULT_PORT);
                return Some(ConnectionProfile {
                    host_ip: self.host_ip.clone(),
                    port,
                    display_name: self.display_name.clone(),
                });
            }
            LoginMessage::BackToModeSelect => {}
        }
        None
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        let title = text("Connect to Remote").size(28).color(TEXT_PRIMARY);

        let host_ip_input = text_input("Tailscale IP (e.g. 100.64.0.1)", &self.host_ip)
            .on_input(LoginMessage::HostIpChanged)
            .style(input_style)
            .padding(10);

        let port_input = text_input("Port", &self.port)
            .on_input(LoginMessage::PortChanged)
            .style(input_style)
            .padding(10);

        let name_input = text_input("Display Name (optional)", &self.display_name)
            .on_input(LoginMessage::DisplayNameChanged)
            .style(input_style)
            .padding(10);

        let connect_button = if self.host_ip.is_empty() {
            button("Connect")
                .style(primary_button_style)
                .padding([12, 24])
        } else {
            button("Connect")
                .on_press(LoginMessage::Connect)
                .style(primary_button_style)
                .padding([12, 24])
        };

        let back_button = button("Back")
            .on_press(LoginMessage::BackToModeSelect)
            .style(secondary_button_style)
            .padding([12, 24]);

        let form = column![
            title,
            host_ip_input,
            row![port_input, name_input].spacing(10),
            row![back_button, connect_button].spacing(10),
        ]
        .spacing(12)
        .align_x(Center);

        let card = container(form)
            .style(card_container_style)
            .padding(36)
            .max_width(450);

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
    fn default_state() {
        let state = LoginState::new();
        assert!(state.host_ip.is_empty());
        assert_eq!(state.port, DEFAULT_PORT.to_string());
        assert!(state.display_name.is_empty());
    }

    #[test]
    fn update_host_ip() {
        let mut state = LoginState::new();
        let result = state.update(LoginMessage::HostIpChanged("100.64.0.1".to_string()));
        assert!(result.is_none());
        assert_eq!(state.host_ip, "100.64.0.1");
    }

    #[test]
    fn connect_with_valid_fields() {
        let mut state = LoginState::new();
        state.host_ip = "100.64.0.1".to_string();

        let result = state.update(LoginMessage::Connect);
        assert!(result.is_some());
        let profile = result.unwrap();
        assert_eq!(profile.host_ip, "100.64.0.1");
        assert_eq!(profile.port, DEFAULT_PORT);
    }

    #[test]
    fn connect_with_empty_host_ip_returns_none() {
        let mut state = LoginState::new();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }

    #[test]
    fn connect_with_custom_port() {
        let mut state = LoginState::new();
        state.host_ip = "100.64.0.1".to_string();
        state.port = "12345".to_string();

        let result = state.update(LoginMessage::Connect);
        assert!(result.is_some());
        let profile = result.unwrap();
        assert_eq!(profile.port, 12345);
    }
}
