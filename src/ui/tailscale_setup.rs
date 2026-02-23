use iced::widget::{button, column, container, text};
use iced::{Center, Element, Fill};

use crate::ui::theme::*;

#[derive(Debug, Clone)]
pub enum TailscaleSetupMessage {
    Install,
    Recheck,
}

#[derive(Debug, Clone)]
pub enum TailscaleSetupStatus {
    NotInstalled,
    NotRunning,
    Checking,
}

pub struct TailscaleSetupState {
    pub status: TailscaleSetupStatus,
}

impl TailscaleSetupState {
    pub fn new(is_installed: bool) -> Self {
        Self {
            status: if is_installed {
                TailscaleSetupStatus::NotRunning
            } else {
                TailscaleSetupStatus::NotInstalled
            },
        }
    }

    pub fn view(&self) -> Element<'_, TailscaleSetupMessage> {
        let title = text("Tailscale Required").size(28).color(TEXT_PRIMARY);

        let (status_msg, detail_msg, show_install) = match &self.status {
            TailscaleSetupStatus::NotInstalled => (
                "Tailscale is not installed",
                "This app requires Tailscale for secure peer-to-peer connections. Install Tailscale and sign in, then click Re-check.",
                true,
            ),
            TailscaleSetupStatus::NotRunning => (
                "Tailscale is not running",
                "Tailscale is installed but not running. Start Tailscale and sign in, then click Re-check.",
                false,
            ),
            TailscaleSetupStatus::Checking => (
                "Checking Tailscale...",
                "Detecting Tailscale installation and status...",
                false,
            ),
        };

        let status_text = text(status_msg).size(18).color(DANGER);
        let detail_text = text(detail_msg).size(14).color(TEXT_SECONDARY);

        let mut col = column![title, status_text, detail_text].spacing(16).align_x(Center);

        if show_install {
            col = col.push(
                button(text("Install Tailscale"))
                    .on_press(TailscaleSetupMessage::Install)
                    .style(primary_button_style)
                    .padding([12, 24]),
            );
        }

        let recheck_btn = match &self.status {
            TailscaleSetupStatus::Checking => {
                button(text("Checking..."))
                    .style(secondary_button_style)
                    .padding([10, 20])
            }
            _ => {
                button(text("Re-check"))
                    .on_press(TailscaleSetupMessage::Recheck)
                    .style(secondary_button_style)
                    .padding([10, 20])
            }
        };
        col = col.push(recheck_btn);

        let card = container(col)
            .style(card_container_style)
            .padding(40)
            .max_width(520);

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
    fn setup_state_not_installed() {
        let state = TailscaleSetupState::new(false);
        assert!(matches!(state.status, TailscaleSetupStatus::NotInstalled));
    }

    #[test]
    fn setup_state_not_running() {
        let state = TailscaleSetupState::new(true);
        assert!(matches!(state.status, TailscaleSetupStatus::NotRunning));
    }
}
