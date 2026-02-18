use std::path::PathBuf;

use iced::widget::{button, column, container, progress_bar, text};
use iced::{Center, Element, Fill};

use crate::cloudflared::DownloadProgress;
use crate::ui::theme::*;

#[derive(Debug, Clone)]
pub enum SetupMessage {
    StartDownload,
    DownloadProgress(DownloadProgress),
    RetryDownload,
    DownloadComplete(PathBuf),
}

#[derive(Debug, Clone)]
pub enum SetupStatus {
    Checking,
    NotFound,
    Downloading { downloaded: u64, total: u64 },
    Done,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct SetupState {
    pub status: SetupStatus,
}

impl SetupState {
    pub fn new() -> Self {
        Self {
            status: SetupStatus::NotFound,
        }
    }

    pub fn view(&self) -> Element<'_, SetupMessage> {
        let title = text("Setup Required").size(28).color(TEXT_PRIMARY);

        let inner: Element<'_, SetupMessage> = match &self.status {
            SetupStatus::Checking => column![
                title,
                text("Checking for cloudflared...").size(16).color(TEXT_SECONDARY),
            ]
            .spacing(20)
            .align_x(Center)
            .into(),
            SetupStatus::NotFound => column![
                title,
                text("cloudflared is required but was not found on this system.")
                    .size(16)
                    .color(TEXT_SECONDARY),
                text("It will be downloaded automatically from GitHub (~30 MB).")
                    .size(14)
                    .color(TEXT_SECONDARY),
                button(text("Download cloudflared").size(16))
                    .on_press(SetupMessage::StartDownload)
                    .style(primary_button_style)
                    .padding([12, 24]),
            ]
            .spacing(16)
            .align_x(Center)
            .into(),
            SetupStatus::Downloading { downloaded, total } => {
                let (mb_down, mb_total) =
                    (*downloaded as f32 / 1_048_576.0, *total as f32 / 1_048_576.0);
                let progress_text = if *total > 0 {
                    format!("{mb_down:.1} / {mb_total:.1} MB")
                } else {
                    format!("{mb_down:.1} MB downloaded")
                };
                let progress_ratio = if *total > 0 {
                    *downloaded as f32 / *total as f32 * 100.0
                } else {
                    0.0
                };

                column![
                    title,
                    text("Downloading cloudflared...").size(16).color(TEXT_SECONDARY),
                    container(
                        progress_bar(0.0..=100.0, progress_ratio).style(progress_bar_style)
                    )
                    .max_width(300),
                    text(progress_text).size(14).color(TEXT_SECONDARY),
                ]
                .spacing(16)
                .align_x(Center)
                .into()
            }
            SetupStatus::Done => column![
                title,
                text("Ready!").size(16).color(SUCCESS),
            ]
            .spacing(20)
            .align_x(Center)
            .into(),
            SetupStatus::Error(e) => column![
                title,
                text(format!("Download failed: {e}")).size(16).color(DANGER),
                button(text("Retry").size(16))
                    .on_press(SetupMessage::RetryDownload)
                    .style(primary_button_style)
                    .padding([12, 24]),
            ]
            .spacing(16)
            .align_x(Center)
            .into(),
        };

        let card = container(inner)
            .style(card_container_style)
            .padding(40)
            .max_width(480);

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
    fn setup_state_default_is_not_found() {
        let state = SetupState::new();
        assert!(matches!(state.status, SetupStatus::NotFound));
    }
}
