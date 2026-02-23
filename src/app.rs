use std::pin::Pin;

use futures::Stream;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Center, Element, Fill, Subscription, Task, Theme};
use crate::ui::theme::*;

use crate::input_handler::translate::iced_key_to_keycode;
use crate::network::client::access_client_subscription;
use crate::network::server::host_server_subscription;
use crate::network::{ConnectionHandle, NetworkEvent};
use crate::protocol::{DEFAULT_PORT, ProtocolMessage};
use crate::tailscale::TailscaleStatus;
use crate::ui::host::{HostMessage, HostState, HostStatus};
use crate::ui::login::{LoginMessage, LoginState};
use crate::ui::mode_select::{ModeSelectMessage, ModeSelectState};
use crate::ui::update::{UpdateBannerState, UpdateMessage, update_banner_view};
use crate::ui::viewer::{ViewerMessage, ViewerState};
use crate::updater::{self, ReleaseInfo, UpdateProgress};

#[derive(Debug, Clone)]
pub enum Message {
    ModeSelect(ModeSelectMessage),
    Login(LoginMessage),
    Host(HostMessage),
    Viewer(ViewerMessage),
    NetworkEvent(NetworkEvent),
    TailscaleCheck(TailscaleStatus),
    Update(UpdateMessage),
    UpdateCheckResult(Option<ReleaseInfo>),
    CopyError,
    StopComplete,
    BackToModeSelect,
    InputSent(Result<(), String>),
}

pub enum Screen {
    ModeSelect(ModeSelectState),
    Login(LoginState),
    Connecting,
    Hosting(HostState),
    Viewer(ViewerState),
    Error(String),
}

#[derive(Clone, Hash)]
struct UpdateDownloadKey {
    url: String,
}

fn download_update_stream(
    key: &UpdateDownloadKey,
) -> Pin<Box<dyn Stream<Item = UpdateMessage> + Send>> {
    use iced::futures::SinkExt;

    let url = key.url.clone();
    Box::pin(iced::stream::channel(32, async move |mut output| {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let download_handle = tokio::spawn(async move { updater::download_update(url, tx).await });

        while let Some(progress) = rx.recv().await {
            let _ = output
                .send(UpdateMessage::DownloadProgress(progress))
                .await;
        }

        match download_handle.await {
            Ok(Ok(path)) => {
                let _ = output.send(UpdateMessage::DownloadComplete(path)).await;
            }
            Ok(Err(e)) => {
                let _ = output
                    .send(UpdateMessage::DownloadProgress(UpdateProgress::Error(e)))
                    .await;
            }
            Err(e) => {
                let _ = output
                    .send(UpdateMessage::DownloadProgress(UpdateProgress::Error(
                        format!("Download task failed: {e}"),
                    )))
                    .await;
            }
        }
    }))
}

pub struct App {
    screen: Screen,
    tailscale_status: TailscaleStatus,
    hosting: bool,
    connecting: bool,
    connect_host: Option<String>,
    connect_port: u16,
    connection_handle: Option<ConnectionHandle>,
    update_banner: UpdateBannerState,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        updater::cleanup_old_update();
        updater::check_post_update_health();

        let update_task = Task::perform(
            async { updater::check_for_update().await.ok().flatten() },
            Message::UpdateCheckResult,
        );

        let tailscale_task = Task::perform(
            crate::tailscale::check_tailscale(),
            Message::TailscaleCheck,
        );

        (
            Self {
                screen: Screen::ModeSelect(ModeSelectState::new()),
                tailscale_status: TailscaleStatus::default(),
                hosting: false,
                connecting: false,
                connect_host: None,
                connect_port: DEFAULT_PORT,
                connection_handle: None,
                update_banner: UpdateBannerState::Hidden,
            },
            Task::batch([update_task, tailscale_task]),
        )
    }

    fn mode_select_screen(&self) -> Screen {
        Screen::ModeSelect(ModeSelectState::new())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TailscaleCheck(status) => {
                self.tailscale_status = status;
            }
            Message::UpdateCheckResult(opt) => {
                if let Some(release) = opt {
                    self.update_banner = UpdateBannerState::Available(release);
                }
            }
            Message::Update(msg) => match msg {
                UpdateMessage::StartDownload => {
                    if let UpdateBannerState::Available(ref release) = self.update_banner {
                        self.update_banner = UpdateBannerState::Downloading {
                            release: release.clone(),
                            downloaded: 0,
                            total: 0,
                        };
                    }
                }
                UpdateMessage::Retry => {
                    self.update_banner = UpdateBannerState::Hidden;
                    return Task::perform(
                        async { updater::check_for_update().await.ok().flatten() },
                        Message::UpdateCheckResult,
                    );
                }
                UpdateMessage::DownloadProgress(progress) => {
                    match &progress {
                        UpdateProgress::Started { total_bytes } => {
                            if let UpdateBannerState::Downloading { ref release, .. } =
                                self.update_banner
                            {
                                self.update_banner = UpdateBannerState::Downloading {
                                    release: release.clone(),
                                    downloaded: 0,
                                    total: *total_bytes,
                                };
                            }
                        }
                        UpdateProgress::Progress { downloaded, total } => {
                            if let UpdateBannerState::Downloading { ref release, .. } =
                                self.update_banner
                            {
                                self.update_banner = UpdateBannerState::Downloading {
                                    release: release.clone(),
                                    downloaded: *downloaded,
                                    total: *total,
                                };
                            }
                        }
                        UpdateProgress::Verifying => {
                            self.update_banner = UpdateBannerState::Verifying;
                        }
                        UpdateProgress::Finished(_) => {}
                        UpdateProgress::Error(e) => {
                            self.update_banner = UpdateBannerState::Error(e.clone());
                        }
                    }
                }
                UpdateMessage::DownloadComplete(path) => {
                    let checksum_url = match &self.update_banner {
                        UpdateBannerState::Downloading { release, .. } => {
                            release.checksum_url.clone()
                        }
                        _ => None,
                    };

                    self.update_banner = UpdateBannerState::Verifying;

                    if let Some(url) = checksum_url {
                        let exe_path = path.clone();
                        return Task::perform(
                            async move {
                                updater::verify_checksum(&exe_path, &url).await?;
                                Ok(exe_path)
                            },
                            |result| Message::Update(UpdateMessage::VerifyComplete(result)),
                        );
                    } else {
                        self.update_banner = UpdateBannerState::Ready(path);
                    }
                }
                UpdateMessage::VerifyComplete(result) => match result {
                    Ok(path) => {
                        self.update_banner = UpdateBannerState::Ready(path);
                    }
                    Err(e) => {
                        self.update_banner = UpdateBannerState::Error(e);
                    }
                },
                UpdateMessage::ApplyAndRestart => {
                    if let UpdateBannerState::Ready(ref path) = self.update_banner {
                        let path = path.clone();
                        self.update_banner = UpdateBannerState::Applying;
                        if let Err(e) = updater::apply_update(&path) {
                            self.update_banner = UpdateBannerState::Error(e);
                        } else {
                            std::process::exit(0);
                        }
                    }
                }
                UpdateMessage::Dismiss => {
                    self.update_banner = UpdateBannerState::Dismissed;
                }
            },
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
                    self.screen = self.mode_select_screen();
                    return Task::none();
                }
                if let Screen::Login(state) = &mut self.screen
                    && let Some(profile) = state.update(msg)
                {
                    self.connect_host = Some(profile.host_ip.clone());
                    self.connect_port = profile.port;
                    self.connecting = true;
                    self.screen = Screen::Connecting;
                }
            }
            Message::Host(msg) => match msg {
                HostMessage::CopyUrl => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.copied = true;
                        if let Some(ref ip) = self.tailscale_status.ip {
                            let addr = format!("{ip}:{}", DEFAULT_PORT);
                            return iced::clipboard::write(addr);
                        }
                    }
                }
                HostMessage::StopHosting => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.status = HostStatus::Stopping;
                    }
                    self.hosting = false;
                    return Task::perform(
                        async { tokio::time::sleep(std::time::Duration::from_secs(1)).await },
                        |_| Message::StopComplete,
                    );
                }
            },
            Message::NetworkEvent(event) => match event {
                NetworkEvent::Listening { port } => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        if let Some(ref ip) = self.tailscale_status.ip {
                            state.tunnel_url = Some(format!("{ip}:{port}"));
                        } else {
                            state.tunnel_url = Some(format!("0.0.0.0:{port}"));
                        }
                        state.status = HostStatus::Active;
                    }
                }
                NetworkEvent::ClientConnected => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.status = HostStatus::Active;
                    }
                }
                NetworkEvent::Connected(handle) => {
                    self.connection_handle = Some(handle);
                    let w = 1920u32;
                    let h = 1080u32;
                    self.screen = Screen::Viewer(ViewerState::new(w, h));
                }
                NetworkEvent::Frame { width, height, pixels } => {
                    if let Screen::Viewer(state) = &mut self.screen {
                        state.update_frame(width, height, pixels);
                    }
                }
                NetworkEvent::ClientDisconnected => {
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.status = HostStatus::Active;
                    }
                }
                NetworkEvent::Error(e) => {
                    self.connecting = false;
                    self.hosting = false;
                    self.connection_handle = None;
                    self.connect_host = None;
                    self.screen = Screen::Error(e);
                }
                NetworkEvent::Stopped => {
                    if self.connecting {
                        self.connecting = false;
                        self.connection_handle = None;
                        self.connect_host = None;
                        self.screen = Screen::Error("Connection closed".to_string());
                    }
                }
            },
            Message::Viewer(msg) => {
                if let Screen::Viewer(_state) = &mut self.screen {
                    match &msg {
                        ViewerMessage::Disconnect => {
                            if let Some(handle) = &self.connection_handle {
                                let handle = handle.clone();
                                drop(tokio::spawn(async move {
                                    let _ = handle.send_input(ProtocolMessage::Disconnect).await;
                                }));
                            }
                            self.connecting = false;
                            self.connection_handle = None;
                            self.connect_host = None;
                            self.screen = Screen::Login(LoginState::new());
                        }
                        ViewerMessage::MouseMoved(point) => {
                            if let Some(handle) = &self.connection_handle {
                                let handle = handle.clone();
                                let x = point.x as u16;
                                let y = point.y as u16;
                                return Task::perform(
                                    async move {
                                        handle.send_input(ProtocolMessage::MouseMove { x, y }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                        ViewerMessage::MousePressed(btn) => {
                            if let Some(protocol_btn) = crate::input_handler::translate::mouse_button_to_protocol(btn)
                                && let Some(handle) = &self.connection_handle
                            {
                                let handle = handle.clone();
                                return Task::perform(
                                    async move {
                                        handle.send_input(ProtocolMessage::MouseButton {
                                            button: protocol_btn,
                                            pressed: true,
                                        }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                        ViewerMessage::MouseReleased(btn) => {
                            if let Some(protocol_btn) = crate::input_handler::translate::mouse_button_to_protocol(btn)
                                && let Some(handle) = &self.connection_handle
                            {
                                let handle = handle.clone();
                                return Task::perform(
                                    async move {
                                        handle.send_input(ProtocolMessage::MouseButton {
                                            button: protocol_btn,
                                            pressed: false,
                                        }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                        ViewerMessage::MouseWheel(delta) => {
                            if let Some(handle) = &self.connection_handle {
                                let handle = handle.clone();
                                let d = *delta as i16;
                                return Task::perform(
                                    async move {
                                        handle.send_input(ProtocolMessage::MouseScroll {
                                            delta_x: 0,
                                            delta_y: d,
                                        }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                        ViewerMessage::KeyPressed(key) => {
                            if let Some(keycode) = iced_key_to_keycode(key)
                                && let Some(handle) = &self.connection_handle
                            {
                                let handle = handle.clone();
                                return Task::perform(
                                    async move {
                                        handle.send_input(ProtocolMessage::KeyEvent {
                                            keycode,
                                            pressed: true,
                                        }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                        ViewerMessage::KeyReleased(key) => {
                            if let Some(keycode) = iced_key_to_keycode(key)
                                && let Some(handle) = &self.connection_handle
                            {
                                let handle = handle.clone();
                                return Task::perform(
                                    async move {
                                        handle.send_input(ProtocolMessage::KeyEvent {
                                            keycode,
                                            pressed: false,
                                        }).await
                                    },
                                    Message::InputSent,
                                );
                            }
                        }
                    }
                }
            }
            Message::StopComplete => {
                self.screen = self.mode_select_screen();
            }
            Message::CopyError => {
                if let Screen::Error(ref e) = self.screen {
                    return iced::clipboard::write(e.clone());
                }
            }
            Message::BackToModeSelect => {
                self.connecting = false;
                self.hosting = false;
                self.connection_handle = None;
                self.connect_host = None;
                self.screen = self.mode_select_screen();
            }
            Message::InputSent(_) => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let banner = update_banner_view(&self.update_banner).map(Message::Update);

        let screen_content: Element<'_, Message> = match &self.screen {
            Screen::ModeSelect(state) => state.view().map(Message::ModeSelect),
            Screen::Login(state) => state.view().map(Message::Login),
            Screen::Connecting => {
                let inner = column![
                    text("Connecting...").size(24).color(TEXT_PRIMARY),
                    text("Establishing connection via Tailscale...").size(14).color(TEXT_SECONDARY),
                ]
                .spacing(12)
                .align_x(Center);

                let card = container(inner)
                    .style(card_container_style)
                    .padding(40)
                    .max_width(400);

                container(card)
                    .center_x(Fill)
                    .center_y(Fill)
                    .into()
            }
            Screen::Hosting(state) => state.view().map(Message::Host),
            Screen::Viewer(state) => state.view().map(Message::Viewer),
            Screen::Error(e) => {
                let error_text = scrollable(
                    container(text(e.to_string()).size(14).color(TEXT_SECONDARY))
                        .padding([12, 16])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(BG_DARK.into()),
                            border: iced::Border {
                                radius: 6.0.into(),
                                width: 1.0,
                                color: BORDER_SUBTLE,
                            },
                            ..Default::default()
                        }),
                )
                .height(iced::Length::Shrink);

                let buttons = row![
                    button("Copy Error")
                        .on_press(Message::CopyError)
                        .style(secondary_button_style)
                        .padding([10, 20]),
                    button("Back")
                        .on_press(Message::BackToModeSelect)
                        .style(secondary_button_style)
                        .padding([10, 20]),
                ]
                .spacing(12)
                .align_y(Center);

                let inner = column![
                    text("Error").size(28).color(DANGER),
                    error_text,
                    buttons,
                ]
                .spacing(20)
                .align_x(Center);

                let card = container(inner)
                    .style(card_container_style)
                    .padding(40)
                    .max_width(520);

                container(card)
                    .center_x(Fill)
                    .center_y(Fill)
                    .into()
            }
        };

        column![banner, screen_content].into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let host_sub = if self.hosting {
            host_server_subscription(DEFAULT_PORT).map(Message::NetworkEvent)
        } else {
            Subscription::none()
        };

        let client_sub = if self.connecting {
            if let Some(ref host) = self.connect_host {
                access_client_subscription(host.clone(), self.connect_port)
                    .map(Message::NetworkEvent)
            } else {
                Subscription::none()
            }
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
                    iced::keyboard::Event::ModifiersChanged(_) => Message::InputSent(Ok(())),
                }),
            _ => Subscription::none(),
        };

        let update_download_sub =
            if let UpdateBannerState::Downloading { ref release, .. } = self.update_banner {
                Subscription::run_with(
                    UpdateDownloadKey {
                        url: release.download_url.clone(),
                    },
                    download_update_stream,
                )
                .map(Message::Update)
            } else {
                Subscription::none()
            };

        Subscription::batch([
            host_sub,
            client_sub,
            keyboard_sub,
            update_download_sub,
        ])
    }

    pub fn theme(&self) -> Theme {
        crate::ui::theme::app_theme()
    }
}
