use std::collections::VecDeque;
use std::time::Instant;

use iced::widget::{button, column, container, image, mouse_area, row, text};
use iced::{Color, Element, Fill};

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
    pub frame_width: u32,
    pub frame_height: u32,
    pub frame_pixels: Vec<u8>,
    frame_times: VecDeque<Instant>,
    pub fps: f32,
    pub latency_ms: Option<u64>,
}

impl ViewerState {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            frame_width: width,
            frame_height: height,
            frame_pixels: vec![0; size],
            frame_times: VecDeque::new(),
            fps: 0.0,
            latency_ms: None,
        }
    }

    pub fn update_frame(&mut self, width: u32, height: u32, pixels: Vec<u8>) {
        self.frame_width = width;
        self.frame_height = height;
        self.frame_pixels = pixels;

        let now = Instant::now();
        self.frame_times.push_back(now);
        while let Some(&front) = self.frame_times.front() {
            if now.duration_since(front).as_secs_f32() > 1.0 {
                self.frame_times.pop_front();
            } else {
                break;
            }
        }
        self.fps = self.frame_times.len() as f32;
    }

    pub fn update_latency(&mut self, rtt_ms: u64) {
        self.latency_ms = Some(rtt_ms);
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

        let fps_color = if self.fps > 20.0 {
            SUCCESS
        } else if self.fps > 10.0 {
            Color::from_rgb(1.0, 0.8, 0.0)
        } else {
            DANGER
        };

        let latency_text = match self.latency_ms {
            Some(ms) => format!("{ms}ms"),
            None => "-- ms".to_string(),
        };

        let resolution_text = format!("{}x{}", self.frame_width, self.frame_height);

        let toolbar = container(
            row![
                text("Connected").size(14).color(SUCCESS),
                text(format!("{:.0} FPS", self.fps)).size(14).color(fps_color),
                text(latency_text).size(14).color(TEXT_SECONDARY),
                text(resolution_text).size(14).color(TEXT_SECONDARY),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_state_creation() {
        let state = ViewerState::new(1920, 1080);
        assert_eq!(state.frame_width, 1920);
        assert_eq!(state.frame_height, 1080);
        assert_eq!(state.fps, 0.0);
        assert!(state.latency_ms.is_none());
    }

    #[test]
    fn fps_tracking() {
        let mut state = ViewerState::new(100, 100);
        let pixels = vec![0u8; 100 * 100 * 4];
        for _ in 0..10 {
            state.update_frame(100, 100, pixels.clone());
        }
        assert!(state.fps >= 1.0);
    }

    #[test]
    fn latency_update() {
        let mut state = ViewerState::new(100, 100);
        assert!(state.latency_ms.is_none());
        state.update_latency(42);
        assert_eq!(state.latency_ms, Some(42));
    }
}
