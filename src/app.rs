use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;

use futures::Stream;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Center, Element, Fill, Subscription, Task, Theme};
use crate::ui::theme::*;

use crate::cloudflared::{self, DownloadProgress};
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
use crate::ui::setup::{SetupMessage, SetupState, SetupStatus};
use crate::ui::update::{UpdateBannerState, UpdateMessage, update_banner_view};
use crate::ui::viewer::{ViewerMessage, ViewerState};
use crate::updater::{self, ReleaseInfo, UpdateProgress};

#[derive(Debug, Clone)]
pub enum Message {
    Setup(SetupMessage),
    ModeSelect(ModeSelectMessage),
    Login(LoginMessage),
    Host(HostMessage),
    Viewer(ViewerMessage),
    RdpEvent(RdpEvent),
    TunnelEvent(TunnelEvent),
    ClientTunnelEvent(TunnelEvent),
    Update(UpdateMessage),
    UpdateCheckResult(Option<ReleaseInfo>),
    CopyError,
    ClientTunnelReady,
    StopComplete,
    BackToLogin,
    InputSent(bool),
}

pub enum Screen {
    Setup(SetupState),
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
        self.0.username.hash(state);
        self.0.proxy_port.hash(state);
    }
}

fn build_rdp_stream(
    profile: &HashableProfile,
) -> Pin<Box<dyn Stream<Item = RdpEvent> + Send>> {
    Box::pin(rdp_subscription(profile.0.clone()))
}

#[derive(Clone, Hash)]
struct DownloadKey;

fn download_cloudflared_stream(
    _key: &DownloadKey,
) -> Pin<Box<dyn Stream<Item = SetupMessage> + Send>> {
    use iced::futures::SinkExt;

    Box::pin(iced::stream::channel(32, async move |mut output| {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let download_handle =
            tokio::spawn(async move { cloudflared::download_cloudflared(tx).await });

        while let Some(progress) = rx.recv().await {
            let _ = output
                .send(SetupMessage::DownloadProgress(progress))
                .await;
        }

        match download_handle.await {
            Ok(Ok(path)) => {
                let _ = output.send(SetupMessage::DownloadComplete(path)).await;
            }
            Ok(Err(e)) => {
                let _ = output
                    .send(SetupMessage::DownloadProgress(DownloadProgress::Error(e)))
                    .await;
            }
            Err(e) => {
                let _ = output
                    .send(SetupMessage::DownloadProgress(DownloadProgress::Error(
                        format!("Download task failed: {e}"),
                    )))
                    .await;
            }
        }
    }))
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
    cloudflared_path: Option<PathBuf>,
    downloading_cloudflared: bool,
    profile: Option<ConnectionProfile>,
    tunnel_handle: Option<TunnelHandle>,
    tunnel_url: Option<String>,
    hosting: bool,
    client_tunnel_active: bool,
    pending_profile: Option<ConnectionProfile>,
    update_banner: UpdateBannerState,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        updater::cleanup_old_update();
        updater::check_post_update_health();

        let cloudflared_path = cloudflared::cloudflared_path();
        let screen = if cloudflared_path.is_some() {
            Screen::ModeSelect(ModeSelectState::new())
        } else {
            Screen::Setup(SetupState::new())
        };

        let update_task = Task::perform(
            async { updater::check_for_update().await.ok().flatten() },
            Message::UpdateCheckResult,
        );

        (
            Self {
                screen,
                cloudflared_path,
                downloading_cloudflared: false,
                profile: None,
                tunnel_handle: None,
                tunnel_url: None,
                hosting: false,
                client_tunnel_active: false,
                pending_profile: None,
                update_banner: UpdateBannerState::Hidden,
            },
            update_task,
        )
    }

    fn mode_select_screen(&self) -> Screen {
        Screen::ModeSelect(ModeSelectState::new())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Setup(msg) => match msg {
                SetupMessage::StartDownload | SetupMessage::RetryDownload => {
                    if let Screen::Setup(state) = &mut self.screen {
                        state.status = SetupStatus::Downloading {
                            downloaded: 0,
                            total: 0,
                        };
                    }
                    self.downloading_cloudflared = true;
                }
                SetupMessage::DownloadProgress(progress) => {
                    if let Screen::Setup(state) = &mut self.screen {
                        match &progress {
                            DownloadProgress::Started { total_bytes } => {
                                state.status = SetupStatus::Downloading {
                                    downloaded: 0,
                                    total: *total_bytes,
                                };
                            }
                            DownloadProgress::Progress { downloaded, total } => {
                                state.status =
                                    SetupStatus::Downloading { downloaded: *downloaded, total: *total };
                            }
                            DownloadProgress::Finished(path) => {
                                self.cloudflared_path = Some(path.clone());
                                self.downloading_cloudflared = false;
                                self.screen = self.mode_select_screen();
                            }
                            DownloadProgress::Error(e) => {
                                state.status = SetupStatus::Error(e.clone());
                                self.downloading_cloudflared = false;
                            }
                        }
                    }
                }
                SetupMessage::DownloadComplete(path) => {
                    self.cloudflared_path = Some(path);
                    self.downloading_cloudflared = false;
                    self.screen = self.mode_select_screen();
                }
            },
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
                    && let Some((tunnel_url, profile)) = state.update(msg)
                {
                    self.tunnel_url = Some(tunnel_url);
                    self.pending_profile = Some(profile.clone());
                    self.client_tunnel_active = true;
                    let proxy_port = profile.proxy_port;
                    self.screen = Screen::Connecting(profile);
                    return Task::perform(
                        async move {
                            let addr = format!("localhost:{proxy_port}");
                            let deadline =
                                tokio::time::Instant::now() + Duration::from_secs(15);
                            loop {
                                if tokio::net::TcpStream::connect(&addr).await.is_ok() {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    return Ok(());
                                }
                                if tokio::time::Instant::now() > deadline {
                                    return Err(
                                        "Tunnel proxy did not start within 15 seconds"
                                            .to_string(),
                                    );
                                }
                                tokio::time::sleep(Duration::from_millis(300)).await;
                            }
                        },
                        |result: Result<(), String>| match result {
                            Ok(()) => Message::ClientTunnelReady,
                            Err(e) => Message::ClientTunnelEvent(TunnelEvent::Error(e)),
                        },
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
                    if let Screen::Hosting(state) = &mut self.screen {
                        state.status = HostStatus::Stopping;
                    }
                    if let Some(mut handle) = self.tunnel_handle.take() {
                        drop(tokio::spawn(async move { handle.stop().await }));
                    }
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
                    self.screen = Screen::Error(e);
                }
                TunnelEvent::Stopped => {
                    self.tunnel_handle = None;
                    self.hosting = false;
                    return Task::perform(
                        async { tokio::time::sleep(Duration::from_secs(1)).await },
                        |_| Message::StopComplete,
                    );
                }
                TunnelEvent::Output(_) => {}
            },
            Message::ClientTunnelEvent(event) => match event {
                TunnelEvent::HandleReady(handle) => {
                    self.tunnel_handle = Some(handle);
                }
                TunnelEvent::Error(e) => {
                    self.client_tunnel_active = false;
                    self.pending_profile = None;
                    if let Some(mut handle) = self.tunnel_handle.take() {
                        drop(tokio::spawn(async move { handle.stop().await }));
                    }
                    self.screen = Screen::Error(e);
                }
                TunnelEvent::Stopped => {
                    self.client_tunnel_active = false;
                    self.tunnel_handle = None;
                    if matches!(self.screen, Screen::Connecting(_)) {
                        self.pending_profile = None;
                        self.screen = Screen::Error(
                            "Tunnel connection closed unexpectedly".to_string(),
                        );
                    }
                }
                TunnelEvent::UrlReady(_) | TunnelEvent::Output(_) => {}
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
            Message::StopComplete => {
                self.screen = self.mode_select_screen();
            }
            Message::CopyError => {
                if let Screen::Error(ref e) = self.screen {
                    return iced::clipboard::write(e.clone());
                }
            }
            Message::BackToLogin => {
                self.profile = None;
                self.client_tunnel_active = false;
                if let Some(mut handle) = self.tunnel_handle.take() {
                    drop(tokio::spawn(async move { handle.stop().await }));
                }
                self.screen = self.mode_select_screen();
            }
            Message::InputSent(_) => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let banner = update_banner_view(&self.update_banner).map(Message::Update);

        let screen_content: Element<'_, Message> = match &self.screen {
            Screen::Setup(state) => state.view().map(Message::Setup),
            Screen::ModeSelect(state) => state.view().map(Message::ModeSelect),
            Screen::Login(state) => state.view().map(Message::Login),
            Screen::Connecting(_) => {
                let inner = column![
                    text("Connecting...").size(24).color(TEXT_PRIMARY),
                    text("Starting tunnel proxy...").size(14).color(TEXT_SECONDARY),
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
                        .on_press(Message::BackToLogin)
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
        let host_tunnel_sub = if self.hosting {
            if let Some(ref path) = self.cloudflared_path {
                let key = HostTunnelKey {
                    cloudflared_path: path.clone(),
                };
                Subscription::run_with(key, host_tunnel_subscription)
                    .map(Message::TunnelEvent)
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        let client_tunnel_sub = if self.client_tunnel_active {
            if let (Some(url), Some(path)) = (&self.tunnel_url, &self.cloudflared_path) {
                let key = ClientTunnelKey {
                    tunnel_url: url.clone(),
                    local_port: 13389,
                    cloudflared_path: path.clone(),
                };
                Subscription::run_with(key, client_tunnel_subscription)
                    .map(Message::ClientTunnelEvent)
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

        let download_sub = if self.downloading_cloudflared {
            Subscription::run_with(DownloadKey, download_cloudflared_stream)
                .map(Message::Setup)
        } else {
            Subscription::none()
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
            download_sub,
            update_download_sub,
            host_tunnel_sub,
            client_tunnel_sub,
            rdp_sub,
            keyboard_sub,
        ])
    }

    pub fn theme(&self) -> Theme {
        crate::ui::theme::app_theme()
    }
}
