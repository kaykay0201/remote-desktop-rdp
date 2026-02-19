use std::path::PathBuf;

use iced::widget::{button, container, progress_bar, row, text, Space};
use iced::{Center, Element, Fill, Length};

use crate::ui::theme::*;
use crate::updater::{ReleaseInfo, UpdateProgress};

#[derive(Debug, Clone)]
pub enum UpdateMessage {
    StartDownload,
    DownloadProgress(UpdateProgress),
    DownloadComplete(PathBuf),
    VerifyComplete(Result<PathBuf, String>),
    ApplyAndRestart,
    Dismiss,
    Retry,
}

#[derive(Debug, Clone)]
pub enum UpdateBannerState {
    Hidden,
    Available(ReleaseInfo),
    Downloading {
        release: ReleaseInfo,
        downloaded: u64,
        total: u64,
    },
    Verifying,
    Ready(PathBuf),
    Applying,
    Error(String),
    Dismissed,
}

pub fn update_banner_view(state: &UpdateBannerState) -> Element<'_, UpdateMessage> {
    match state {
        UpdateBannerState::Hidden | UpdateBannerState::Dismissed => {
            Space::new().into()
        }
        UpdateBannerState::Available(release) => {
            let content = row![
                text(format!("Update {} available", release.version))
                    .size(14)
                    .color(TEXT_PRIMARY),
                Space::new().width(Length::Fill),
                button(text("Update Now").size(13))
                    .on_press(UpdateMessage::StartDownload)
                    .style(primary_button_style)
                    .padding([6, 16]),
                button(text("Later").size(13))
                    .on_press(UpdateMessage::Dismiss)
                    .style(secondary_button_style)
                    .padding([6, 16]),
            ]
            .spacing(12)
            .align_y(Center);

            container(content)
                .style(banner_container_style)
                .padding([8, 16])
                .width(Fill)
                .into()
        }
        UpdateBannerState::Downloading {
            downloaded, total, ..
        } => {
            let (mb_down, mb_total) =
                (*downloaded as f32 / 1_048_576.0, *total as f32 / 1_048_576.0);
            let progress_text = if *total > 0 {
                format!("{mb_down:.1} / {mb_total:.1} MB")
            } else {
                format!("{mb_down:.1} MB")
            };
            let progress_ratio = if *total > 0 {
                *downloaded as f32 / *total as f32 * 100.0
            } else {
                0.0
            };

            let content = row![
                text("Downloading update...").size(14).color(TEXT_PRIMARY),
                container(progress_bar(0.0..=100.0, progress_ratio).style(progress_bar_style))
                    .max_width(200),
                text(progress_text).size(13).color(TEXT_SECONDARY),
            ]
            .spacing(12)
            .align_y(Center);

            container(content)
                .style(banner_container_style)
                .padding([8, 16])
                .width(Fill)
                .into()
        }
        UpdateBannerState::Verifying => {
            let content = row![text("Verifying update...").size(14).color(TEXT_PRIMARY),]
                .spacing(12)
                .align_y(Center);

            container(content)
                .style(banner_container_style)
                .padding([8, 16])
                .width(Fill)
                .into()
        }
        UpdateBannerState::Ready(_) => {
            let content = row![
                text("Update ready!").size(14).color(SUCCESS),
                Space::new().width(Length::Fill),
                button(text("Restart Now").size(13))
                    .on_press(UpdateMessage::ApplyAndRestart)
                    .style(primary_button_style)
                    .padding([6, 16]),
                button(text("Later").size(13))
                    .on_press(UpdateMessage::Dismiss)
                    .style(secondary_button_style)
                    .padding([6, 16]),
            ]
            .spacing(12)
            .align_y(Center);

            container(content)
                .style(banner_container_style)
                .padding([8, 16])
                .width(Fill)
                .into()
        }
        UpdateBannerState::Applying => {
            let content = row![text("Applying update...").size(14).color(TEXT_PRIMARY),]
                .spacing(12)
                .align_y(Center);

            container(content)
                .style(banner_container_style)
                .padding([8, 16])
                .width(Fill)
                .into()
        }
        UpdateBannerState::Error(e) => {
            let content = row![
                text(format!("Update failed: {e}")).size(14).color(DANGER),
                Space::new().width(Length::Fill),
                button(text("Retry").size(13))
                    .on_press(UpdateMessage::Retry)
                    .style(primary_button_style)
                    .padding([6, 16]),
                button(text("Dismiss").size(13))
                    .on_press(UpdateMessage::Dismiss)
                    .style(secondary_button_style)
                    .padding([6, 16]),
            ]
            .spacing(12)
            .align_y(Center);

            container(content)
                .style(banner_container_style)
                .padding([8, 16])
                .width(Fill)
                .into()
        }
    }
}
