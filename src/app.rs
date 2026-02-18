use std::hash::{Hash, Hasher};
use std::pin::Pin;

use futures::Stream;
use iced::widget::{button, column, container, text};
use iced::{Center, Element, Fill, Subscription, Task, Theme};

use crate::config::ConnectionProfile;
use crate::rdp::input::iced_key_to_scancode;
use crate::rdp::session::rdp_subscription;
use crate::rdp::{InputCommand, MouseButtonKind, RdpEvent};
use crate::tunnel::{
    client_tunnel_subscription, host_tunnel_subscription, ClientTunnelKey, HostTunnelKey,
    TunnelEvent, TunnelHandle,
};
use crate::ui::host::{HostMessage, HostState, HostStatus};
use crate::ui::login::{LoginMessage, LoginState};
use crate::ui::mode_select::{ModeSelectMessage, ModeSelectState};
use crate::ui::viewer::{ViewerMessage, ViewerState};

#[derive(Debug, Clone)]
pub enum Message {
    ModeSelect(ModeSelectMessage),
    Login(LoginMessage),
    Host(HostMessage),
    Viewer(ViewerMessage),
    RdpEvent(RdpEvent),
    TunnelEvent(TunnelEvent),
    ClientTunnelReady,
    BackToLogin,
    InputSent(bool),
}

pub enum Screen {
    ModeSelect(ModeSelectState),
    Login(LoginState),
    Connecting(ConnectionProfile),
    Hosting(HostState),
    Viewer(ViewerState),
    Error(String),
}

#[derive(Clone)]
struct HashableProfile(ConnectionProfile);

impl Hash for HashableProfile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hostname.hash(state);
        self.0.port.hash(state);
        self.0.username.hash(state);
        self.0.proxy_port.hash(state);
    }
}

fn build_rdp_stream(
    profile: &HashableProfile,
) -> Pin<Box<dyn Stream<Item = RdpEvent> + Send>> {
    Box::pin(rdp_subscription(profile.0.clone()))
}

pub struct App {
    screen: Screen,
    profile: Option<ConnectionProfile>,
    tunnel_handle: Option<TunnelHandle>,
    tunnel_url: Option<String>,
    hosting: bool,
    client_tunnel_active: bool,
    pending_profile: Option<ConnectionProfile>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::ModeSelect(ModeSelectState::new()),
                profile: None,
                tunnel_handle: None,
                tunnel_url: None,
                hosting: false,
                client_tunnel_active: false,
                pending_profile: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ModeSelect(msg) => match msg {
                ModeSelectMessage::ConnectSelected => {
                    self.screen = Screen::Login(LoginState::new());
                }
                ModeSelectMessage::HostSelected => {
                    self.hosting = true;
                    self.screen = Screen::Hosting(HostState::new());
                }
            },
            Message::Login(msg) => {
                let is_back = matches!(msg, LoginMessage::BackToModeSelect);
                if is_back {
                    self.screen = Screen::ModeSelect(ModeSelectState::new());
                    return Task::none();
                }
                if let Screen::Login(state) = &mut self.screen
                    && let Some((tunnel_url, profile)) = state.update(msg)
                {
                    self.tunnel_url = Some(tunnel_url);
                    self.pending_profile = Some(profile.clone());
                    self.client_tunnel_active = true;
                    self.screen = Screen::Connecting(profile);
                    return Task::perform(
                        async {
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        },
                        |_| Message::ClientTunnelReady,
                    );
                }
            }
            Message::ClientTunnelReady => {
                if let Some(profile) = self.pending_profile.take() {
                    self.profile = Some(profile);
                }
            }
            Message::Host(msg) => match msg {
                HostMessage::CopyUrl => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.copied = true;
                        if let Some(ref url) = state.tunnel_url {
                            return iced::clipboard::write(url.clone());
                        }
                    }
                }
                HostMessage::StopHosting => {
                    if let Some(mut handle) = self.tunnel_handle.take() {
                        drop(tokio::spawn(async move { handle.stop().await }));
                    }
                    self.hosting = false;
                    self.screen = Screen::ModeSelect(ModeSelectState::new());
                }
            },
            Message::TunnelEvent(event) => match event {
                TunnelEvent::HandleReady(handle) => {
                    self.tunnel_handle = Some(handle);
                }
                TunnelEvent::UrlReady(url) => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.tunnel_url = Some(url);
                        state.status = HostStatus::Active;
                    }
                }
                TunnelEvent::Error(e) => {
                    if let Some(mut handle) = self.tunnel_handle.take() {
                        drop(tokio::spawn(async move { handle.stop().await }));
                    }
                    self.hosting = false;
                    self.client_tunnel_active = false;
                    self.screen = Screen::Error(e);
                }
                TunnelEvent::Stopped => {
                    self.tunnel_handle = None;
                    if self.hosting {
                        self.hosting = false;
                        self.screen = Screen::ModeSelect(ModeSelectState::new());
                    }
                }
                TunnelEvent::Output(_) => {}
            },
            Message::RdpEvent(event) => match event {
                RdpEvent::Connected(conn) => {
                    let (w, h) = match &self.screen {
                        Screen::Connecting(p) => (p.width as u32, p.height as u32),
                        _ => (1920, 1080),
                    };
                    self.screen = Screen::Viewer(ViewerState::new(conn, w, h));
                }
                RdpEvent::Frame {
                    width,
                    height,
                    pixels,
                } => {
                    if let Screen::Viewer(state) = &mut self.screen {
                        state.update_frame(width, height, pixels);
                    }
                }
                RdpEvent::Error(e) => {
                    self.profile = None;
                    self.client_tunnel_active = false;
                    if let Some(mut handle) = self.tunnel_handle.take() {
                        drop(tokio::spawn(async move { handle.stop().await }));
                    }
                    self.screen = Screen::Error(e);
                }
                RdpEvent::Disconnected => {
                    self.profile = None;
                    self.client_tunnel_active = false;
                    if let Some(mut handle) = self.tunnel_handle.take() {
                        drop(tokio::spawn(async move { handle.stop().await }));
                    }
                    self.screen = Screen::Login(LoginState::new());
                }
                RdpEvent::StatusChanged(_) => {}
            },
            Message::Viewer(msg) => {
                if let Screen::Viewer(state) = &mut self.screen {
                    match &msg {
                        ViewerMessage::Disconnect => {
                            let mut conn = state.connection.clone();
                            self.profile = None;
                            self.client_tunnel_active = false;
                            if let Some(mut handle) = self.tunnel_handle.take() {
                                drop(tokio::spawn(async move { handle.stop().await }));
                            }
                            self.screen = Screen::Login(LoginState::new());
                            return Task::perform(
                                async move {
                                    conn.send(InputCommand::Disconnect).await
                                },
                                Message::InputSent,
                            );
                        }
                        ViewerMessage::MouseMoved(point) => {
                            let mut conn = state.connection.clone();
                            let x = point.x as u16;
                            let y = point.y as u16;
                            return Task::perform(
                                async move {
                                    conn.send(InputCommand::MouseMoved { x, y }).await
                                },
                                Message::InputSent,
                            );
                        }
                        ViewerMessage::MousePressed(btn) => {
                            let kind = match btn {
                                iced::mouse::Button::Left => MouseButtonKind::Left,
                                iced::mouse::Button::Right => MouseButtonKind::Right,
                                iced::mouse::Button::Middle => MouseButtonKind::Middle,
                                _ => return Task::none(),
                            };
                            let mut conn = state.connection.clone();
                            return Task::perform(
                                async move {
                                    conn.send(InputCommand::MouseButtonPressed(kind))
                                        .await
                                },
                                Message::InputSent,
                            );
                        }
                        ViewerMessage::MouseReleased(btn) => {
                            let kind = match btn {
                                iced::mouse::Button::Left => MouseButtonKind::Left,
                                iced::mouse::Button::Right => MouseButtonKind::Right,
                                iced::mouse::Button::Middle => MouseButtonKind::Middle,
                                _ => return Task::none(),
                            };
                            let mut conn = state.connection.clone();
                            return Task::perform(
                                async move {
                                    conn.send(InputCommand::MouseButtonReleased(kind))
                                        .await
                                },
                                Message::InputSent,
                            );
                        }
                        ViewerMessage::MouseWheel(delta) => {
                            let d = *delta as i16;
                            let mut conn = state.connection.clone();
                            return Task::perform(
                                async move {
                                    conn.send(InputCommand::MouseWheel {
                                        vertical: true,
                                        delta: d,
                                    })
                                    .await
                                },
                                Message::InputSent,
                            );
                        }
                        ViewerMessage::KeyPressed(key) => {
                            if let Some(scancode) = iced_key_to_scancode(key) {
                                let mut conn = state.connection.clone();
                                return Task::perform(
                                    async move {
                                        conn.send(InputCommand::KeyPressed { scancode }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                        ViewerMessage::KeyReleased(key) => {
                            if let Some(scancode) = iced_key_to_scancode(key) {
                                let mut conn = state.connection.clone();
                                return Task::perform(
                                    async move {
                                        conn.send(InputCommand::KeyReleased { scancode }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                    }
                }
            }
            Message::BackToLogin => {
                self.profile = None;
                self.client_tunnel_active = false;
                if let Some(mut handle) = self.tunnel_handle.take() {
                    drop(tokio::spawn(async move { handle.stop().await }));
                }
                self.screen = Screen::ModeSelect(ModeSelectState::new());
            }
            Message::InputSent(_) => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::ModeSelect(state) => state.view().map(Message::ModeSelect),
            Screen::Login(state) => state.view().map(Message::Login),
            Screen::Connecting(_) => container(text("Connecting...").size(24))
                .center_x(Fill)
                .center_y(Fill)
                .into(),
            Screen::Hosting(state) => state.view().map(Message::Host),
            Screen::Viewer(state) => state.view().map(Message::Viewer),
            Screen::Error(e) => container(
                column![
                    text(format!("Error: {}", e)).size(18),
                    button("Back").on_press(Message::BackToLogin),
                ]
                .spacing(20)
                .align_x(Center),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let host_tunnel_sub = if self.hosting {
            Subscription::run_with(HostTunnelKey, host_tunnel_subscription)
                .map(Message::TunnelEvent)
        } else {
            Subscription::none()
        };

        let client_tunnel_sub = if self.client_tunnel_active {
            if let Some(ref url) = self.tunnel_url {
                let key = ClientTunnelKey {
                    tunnel_url: url.clone(),
                    local_port: 13389,
                };
                Subscription::run_with(key, client_tunnel_subscription)
                    .map(Message::TunnelEvent)
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        let rdp_sub = if let Some(profile) = &self.profile {
            Subscription::run_with(HashableProfile(profile.clone()), build_rdp_stream)
                .map(Message::RdpEvent)
        } else {
            Subscription::none()
        };

        let keyboard_sub = match &self.screen {
            Screen::Viewer(_) => iced::keyboard::listen()
                .map(|event| match event {
                    iced::keyboard::Event::KeyPressed { key, .. } => {
                        Message::Viewer(ViewerMessage::KeyPressed(key))
                    }
                    iced::keyboard::Event::KeyReleased { key, .. } => {
                        Message::Viewer(ViewerMessage::KeyReleased(key))
                    }
                    iced::keyboard::Event::ModifiersChanged(_) => Message::InputSent(true),
                }),
            _ => Subscription::none(),
        };

        Subscription::batch([host_tunnel_sub, client_tunnel_sub, rdp_sub, keyboard_sub])
    }

    pub fn theme(&self) -> Theme {
        crate::ui::theme::app_theme()
    }
}
