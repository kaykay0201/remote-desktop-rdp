use iced::widget::{button, column, container, row, text, text_input};
use iced::{Center, Element, Fill};

use crate::config::ConnectionProfile;

const LOCAL_TUNNEL_PORT: u16 = 13389;

#[derive(Debug, Clone)]
pub enum LoginMessage {
    TunnelUrlChanged(String),
    PortChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    WidthChanged(String),
    HeightChanged(String),
    Connect,
    BackToModeSelect,
}

#[derive(Debug, Clone, Default)]
pub struct LoginState {
    pub tunnel_url: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub width: String,
    pub height: String,
}

impl LoginState {
    pub fn new() -> Self {
        let defaults = ConnectionProfile::default();
        Self {
            tunnel_url: String::new(),
            port: defaults.port.to_string(),
            username: String::new(),
            password: String::new(),
            width: defaults.width.to_string(),
            height: defaults.height.to_string(),
        }
    }

    pub fn update(&mut self, msg: LoginMessage) -> Option<(String, ConnectionProfile)> {
        match msg {
            LoginMessage::TunnelUrlChanged(s) => self.tunnel_url = s,
            LoginMessage::PortChanged(s) => self.port = s,
            LoginMessage::UsernameChanged(s) => self.username = s,
            LoginMessage::PasswordChanged(s) => self.password = s,
            LoginMessage::WidthChanged(s) => self.width = s,
            LoginMessage::HeightChanged(s) => self.height = s,
            LoginMessage::Connect => {
                if self.tunnel_url.is_empty() {
                    return None;
                }
                let port = match self.port.parse::<u16>() {
                    Ok(p) => p,
                    Err(_) => return None,
                };
                if self.username.is_empty() {
                    return None;
                }
                let width = match self.width.parse::<u16>() {
                    Ok(w) => w,
                    Err(_) => return None,
                };
                let height = match self.height.parse::<u16>() {
                    Ok(h) => h,
                    Err(_) => return None,
                };
                let tunnel_url = self.tunnel_url.clone();
                let profile = ConnectionProfile {
                    hostname: "localhost".to_string(),
                    port,
                    username: self.username.clone(),
                    password: self.password.clone(),
                    width,
                    height,
                    proxy_port: LOCAL_TUNNEL_PORT,
                };
                return Some((tunnel_url, profile));
            }
            LoginMessage::BackToModeSelect => {}
        }
        None
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        let title = text("Connect to Remote").size(28);

        let tunnel_url_input = text_input("Tunnel URL (https://xxx.trycloudflare.com)", &self.tunnel_url)
            .on_input(LoginMessage::TunnelUrlChanged)
            .padding(8);

        let port_input = text_input("RDP Port", &self.port)
            .on_input(LoginMessage::PortChanged)
            .padding(8);

        let username_input = text_input("Username", &self.username)
            .on_input(LoginMessage::UsernameChanged)
            .padding(8);

        let password_input = text_input("Password", &self.password)
            .on_input(LoginMessage::PasswordChanged)
            .secure(true)
            .padding(8);

        let width_input = text_input("Width", &self.width)
            .on_input(LoginMessage::WidthChanged)
            .padding(8);

        let height_input = text_input("Height", &self.height)
            .on_input(LoginMessage::HeightChanged)
            .padding(8);

        let connect_button = if self.tunnel_url.is_empty() {
            button("Connect").padding(10)
        } else {
            button("Connect")
                .on_press(LoginMessage::Connect)
                .padding(10)
        };

        let back_button = button("Back")
            .on_press(LoginMessage::BackToModeSelect)
            .padding(10);

        let form = column![
            title,
            tunnel_url_input,
            port_input,
            username_input,
            password_input,
            row![width_input, height_input].spacing(10),
            row![back_button, connect_button].spacing(10),
        ]
        .spacing(12)
        .padding(30)
        .max_width(450)
        .align_x(Center);

        container(form)
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
        assert!(state.tunnel_url.is_empty());
        assert_eq!(state.port, "3389");
        assert!(state.username.is_empty());
        assert!(state.password.is_empty());
        assert_eq!(state.width, "1920");
        assert_eq!(state.height, "1080");
    }

    #[test]
    fn update_tunnel_url() {
        let mut state = LoginState::new();
        let result = state.update(LoginMessage::TunnelUrlChanged(
            "https://test.trycloudflare.com".to_string(),
        ));
        assert!(result.is_none());
        assert_eq!(state.tunnel_url, "https://test.trycloudflare.com");
    }

    #[test]
    fn connect_with_valid_fields() {
        let mut state = LoginState::new();
        state.tunnel_url = "https://test.trycloudflare.com".to_string();
        state.username = "admin".to_string();

        let result = state.update(LoginMessage::Connect);
        assert!(result.is_some());
        let (tunnel_url, profile) = result.unwrap();
        assert_eq!(tunnel_url, "https://test.trycloudflare.com");
        assert_eq!(profile.hostname, "localhost");
        assert_eq!(profile.proxy_port, LOCAL_TUNNEL_PORT);
        assert_eq!(profile.username, "admin");
        assert_eq!(profile.port, 3389);
    }

    #[test]
    fn connect_with_empty_tunnel_url_returns_none() {
        let mut state = LoginState::new();
        state.username = "admin".to_string();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }

    #[test]
    fn connect_with_empty_username_returns_none() {
        let mut state = LoginState::new();
        state.tunnel_url = "https://test.trycloudflare.com".to_string();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }

    #[test]
    fn connect_with_invalid_port_returns_none() {
        let mut state = LoginState::new();
        state.tunnel_url = "https://test.trycloudflare.com".to_string();
        state.username = "admin".to_string();
        state.port = "not_a_number".to_string();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }
}
