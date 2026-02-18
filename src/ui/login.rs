use iced::widget::{button, column, container, row, text, text_input};
use iced::{Center, Element, Fill};

use crate::config::ConnectionProfile;

#[derive(Debug, Clone)]
pub enum LoginMessage {
    HostnameChanged(String),
    PortChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    WidthChanged(String),
    HeightChanged(String),
    ProxyPortChanged(String),
    Connect,
}

#[derive(Debug, Clone, Default)]
pub struct LoginState {
    pub hostname: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub width: String,
    pub height: String,
    pub proxy_port: String,
}

impl LoginState {
    pub fn new() -> Self {
        let defaults = ConnectionProfile::default();
        Self {
            hostname: String::new(),
            port: defaults.port.to_string(),
            username: String::new(),
            password: String::new(),
            width: defaults.width.to_string(),
            height: defaults.height.to_string(),
            proxy_port: defaults.proxy_port.to_string(),
        }
    }

    pub fn update(&mut self, msg: LoginMessage) -> Option<ConnectionProfile> {
        match msg {
            LoginMessage::HostnameChanged(s) => self.hostname = s,
            LoginMessage::PortChanged(s) => self.port = s,
            LoginMessage::UsernameChanged(s) => self.username = s,
            LoginMessage::PasswordChanged(s) => self.password = s,
            LoginMessage::WidthChanged(s) => self.width = s,
            LoginMessage::HeightChanged(s) => self.height = s,
            LoginMessage::ProxyPortChanged(s) => self.proxy_port = s,
            LoginMessage::Connect => {
                if self.hostname.is_empty() {
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
                let proxy_port = match self.proxy_port.parse::<u16>() {
                    Ok(p) => p,
                    Err(_) => return None,
                };
                return Some(ConnectionProfile {
                    hostname: self.hostname.clone(),
                    port,
                    username: self.username.clone(),
                    password: self.password.clone(),
                    width,
                    height,
                    proxy_port,
                });
            }
        }
        None
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        let title = text("Rust RDP Client").size(28);

        let hostname_input = text_input("Hostname", &self.hostname)
            .on_input(LoginMessage::HostnameChanged)
            .padding(8);

        let port_input = text_input("Port", &self.port)
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

        let proxy_port_input = text_input("Proxy Port", &self.proxy_port)
            .on_input(LoginMessage::ProxyPortChanged)
            .padding(8);

        let connect_button = if self.hostname.is_empty() {
            button("Connect").padding(10)
        } else {
            button("Connect")
                .on_press(LoginMessage::Connect)
                .padding(10)
        };

        let form = column![
            title,
            hostname_input,
            row![port_input, proxy_port_input].spacing(10),
            username_input,
            password_input,
            row![width_input, height_input].spacing(10),
            connect_button,
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
        assert!(state.hostname.is_empty());
        assert_eq!(state.port, "3389");
        assert!(state.username.is_empty());
        assert!(state.password.is_empty());
        assert_eq!(state.width, "1920");
        assert_eq!(state.height, "1080");
        assert_eq!(state.proxy_port, "3390");
    }

    #[test]
    fn update_hostname() {
        let mut state = LoginState::new();
        let result = state.update(LoginMessage::HostnameChanged("test.com".to_string()));
        assert!(result.is_none());
        assert_eq!(state.hostname, "test.com");
    }

    #[test]
    fn connect_with_valid_fields() {
        let mut state = LoginState::new();
        state.hostname = "test.com".to_string();
        state.username = "admin".to_string();

        let result = state.update(LoginMessage::Connect);
        assert!(result.is_some());
        let profile = result.unwrap();
        assert_eq!(profile.hostname, "test.com");
        assert_eq!(profile.username, "admin");
        assert_eq!(profile.port, 3389);
    }

    #[test]
    fn connect_with_empty_hostname_returns_none() {
        let mut state = LoginState::new();
        state.username = "admin".to_string();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }

    #[test]
    fn connect_with_empty_username_returns_none() {
        let mut state = LoginState::new();
        state.hostname = "test.com".to_string();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }

    #[test]
    fn connect_with_invalid_port_returns_none() {
        let mut state = LoginState::new();
        state.hostname = "test.com".to_string();
        state.username = "admin".to_string();
        state.port = "not_a_number".to_string();
        let result = state.update(LoginMessage::Connect);
        assert!(result.is_none());
    }
}
