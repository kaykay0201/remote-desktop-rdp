use iced::widget::{button, column, container, image, mouse_area, row, text};
use iced::{Element, Fill};

use crate::rdp::RdpConnection;
use crate::ui::theme::*;

#[derive(Debug, Clone)]
pub enum ViewerMessage {
    MouseMoved(iced::Point),
    MousePressed(iced::mouse::Button),
    MouseReleased(iced::mouse::Button),
    MouseWheel(f32),
    KeyPressed(iced::keyboard::Key),
    KeyReleased(iced::keyboard::Key),
    Disconnect,
}

pub struct ViewerState {
    pub connection: RdpConnection,
    pub frame_width: u32,
    pub frame_height: u32,
    pub frame_pixels: Vec<u8>,
}

impl ViewerState {
    pub fn new(connection: RdpConnection, width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            connection,
            frame_width: width,
            frame_height: height,
            frame_pixels: vec![0; size],
        }
    }

    pub fn update_frame(&mut self, width: u32, height: u32, pixels: Vec<u8>) {
        self.frame_width = width;
        self.frame_height = height;
        self.frame_pixels = pixels;
    }

    pub fn view(&self) -> Element<'_, ViewerMessage> {
        let handle = image::Handle::from_rgba(
            self.frame_width,
            self.frame_height,
            self.frame_pixels.clone(),
        );

        let image_widget = image(handle).width(Fill).height(Fill);

        let viewer_area = mouse_area(image_widget)
            .on_press(ViewerMessage::MousePressed(iced::mouse::Button::Left))
            .on_release(ViewerMessage::MouseReleased(iced::mouse::Button::Left))
            .on_move(ViewerMessage::MouseMoved)
            .on_scroll(|delta| {
                let y = match delta {
                    iced::mouse::ScrollDelta::Lines { y, .. } => y,
                    iced::mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                ViewerMessage::MouseWheel(y)
            });

        let toolbar = container(
            row![
                text("Connected").size(14).color(SUCCESS),
                button("Disconnect")
                    .on_press(ViewerMessage::Disconnect)
                    .style(danger_button_style)
                    .padding([4, 12]),
            ]
            .spacing(10)
            .padding(6),
        )
        .style(toolbar_container_style)
        .width(Fill);

        let content = column![toolbar, viewer_area].spacing(0);

        container(content).width(Fill).height(Fill).into()
    }
}
