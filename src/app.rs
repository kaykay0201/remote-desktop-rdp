use std::hash::{Hash, Hasher};
use std::pin::Pin;

use futures::Stream;
use iced::widget::{button, column, container, text};
use iced::{Center, Element, Fill, Subscription, Task, Theme};

use crate::config::ConnectionProfile;
use crate::rdp::input::iced_key_to_scancode;
use crate::rdp::session::rdp_subscription;
use crate::rdp::{InputCommand, MouseButtonKind, RdpEvent};
use crate::ui::login::{LoginMessage, LoginState};
use crate::ui::viewer::{ViewerMessage, ViewerState};

#[derive(Debug, Clone)]
pub enum Message {
    Login(LoginMessage),
    Viewer(ViewerMessage),
    RdpEvent(RdpEvent),
    BackToLogin,
    InputSent(bool),
}

pub enum Screen {
    Login(LoginState),
    Connecting(ConnectionProfile),
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
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Login(LoginState::new()),
                profile: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Login(msg) => {
                if let Screen::Login(state) = &mut self.screen
                    && let Some(profile) = state.update(msg)
                {
                    self.profile = Some(profile.clone());
                    self.screen = Screen::Connecting(profile);
                }
            }
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
                    self.screen = Screen::Error(e);
                }
                RdpEvent::Disconnected => {
                    self.profile = None;
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
                self.screen = Screen::Login(LoginState::new());
            }
            Message::InputSent(_) => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Login(state) => state.view().map(Message::Login),
            Screen::Connecting(_) => container(text("Connecting...").size(24))
                .center_x(Fill)
                .center_y(Fill)
                .into(),
            Screen::Viewer(state) => state.view().map(Message::Viewer),
            Screen::Error(e) => container(
                column![
                    text(format!("Error: {}", e)).size(18),
                    button("Back to Login").on_press(Message::BackToLogin),
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

        Subscription::batch([rdp_sub, keyboard_sub])
    }

    pub fn theme(&self) -> Theme {
        crate::ui::theme::app_theme()
    }
}
