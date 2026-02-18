use std::path::PathBuf;

use iced::widget::{button, column, container, progress_bar, text};
use iced::{Center, Element, Fill};

use crate::ui::theme::*;
use crate::updater::{ReleaseInfo, UpdateProgress};

#[derive(Debug, Clone)]
pub enum UpdateMessage {
    StartUpdate,
    DownloadProgress(UpdateProgress),
    DownloadComplete(PathBuf),
    ApplyAndRestart,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Available,
    Downloading { downloaded: u64, total: u64 },
    ReadyToInstall(PathBuf),
    Applying,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct UpdateState {
    pub release: ReleaseInfo,
    pub status: UpdateStatus,
}

impl UpdateState {
    pub fn new(release: ReleaseInfo) -> Self {
        Self {
            release,
            status: UpdateStatus::Available,
        }
    }

    pub fn view(&self) -> Element<'_, UpdateMessage> {
        let title = text("Update Available").size(28).color(TEXT_PRIMARY);

        let inner: Element<'_, UpdateMessage> = match &self.status {
            UpdateStatus::Available => column![
                title,
                text(format!("Version {} is available", self.release.version))
                    .size(16)
                    .color(TEXT_SECONDARY),
                text(&self.release.body).size(14).color(TEXT_SECONDARY),
                button(text("Update Now").size(16))
                    .on_press(UpdateMessage::StartUpdate)
                    .style(primary_button_style)
                    .padding([12, 24]),
                button(text("Later").size(16))
                    .on_press(UpdateMessage::Cancel)
                    .style(secondary_button_style)
                    .padding([12, 24]),
            ]
            .spacing(16)
            .align_x(Center)
            .into(),
            UpdateStatus::Downloading { downloaded, total } => {
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
                    text("Downloading update...").size(16).color(TEXT_SECONDARY),
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
            UpdateStatus::ReadyToInstall(_) => column![
                title,
                text("Update downloaded successfully!").size(16).color(SUCCESS),
                button(text("Restart & Update").size(16))
                    .on_press(UpdateMessage::ApplyAndRestart)
                    .style(primary_button_style)
                    .padding([12, 24]),
                button(text("Later").size(16))
                    .on_press(UpdateMessage::Cancel)
                    .style(secondary_button_style)
                    .padding([12, 24]),
            ]
            .spacing(16)
            .align_x(Center)
            .into(),
            UpdateStatus::Applying => column![
                title,
                text("Applying update...").size(16).color(TEXT_SECONDARY),
            ]
            .spacing(16)
            .align_x(Center)
            .into(),
            UpdateStatus::Error(e) => column![
                title,
                text(format!("Update failed: {e}")).size(16).color(DANGER),
                button(text("Retry").size(16))
                    .on_press(UpdateMessage::StartUpdate)
                    .style(primary_button_style)
                    .padding([12, 24]),
                button(text("Cancel").size(16))
                    .on_press(UpdateMessage::Cancel)
                    .style(secondary_button_style)
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
