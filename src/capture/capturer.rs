use std::time::Duration;
use tokio::sync::mpsc;

use crate::capture::encoder::encode_frame;
use crate::capture::{CaptureCommand, CaptureConfig, CaptureEvent};

pub fn capture_loop(
    config: CaptureConfig,
    event_tx: mpsc::Sender<CaptureEvent>,
    cmd_rx: mpsc::Receiver<CaptureCommand>,
) {
    let mut cmd_rx = cmd_rx;
    let mut jpeg_quality = config.jpeg_quality;

    let display = match scrap::Display::primary() {
        Ok(d) => d,
        Err(e) => {
            let _ = event_tx.blocking_send(CaptureEvent::Error(e.to_string()));
            return;
        }
    };
    let width = display.width() as u32;
    let height = display.height() as u32;
    let mut capturer = match scrap::Capturer::new(display) {
        Ok(c) => c,
        Err(e) => {
            let _ = event_tx.blocking_send(CaptureEvent::Error(e.to_string()));
            return;
        }
    };
    let _ = event_tx.blocking_send(CaptureEvent::Started { width, height });

    let frame_interval = Duration::from_secs(1) / config.fps;

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                CaptureCommand::SetQuality(q) => jpeg_quality = q,
                CaptureCommand::Stop => {
                    let _ = event_tx.blocking_send(CaptureEvent::Stopped);
                    return;
                }
            }
        }

        let start = std::time::Instant::now();
        match capturer.frame() {
            Ok(frame) => {
                let stride = frame.len() / height as usize;
                let bgra = if stride == width as usize * 4 {
                    frame.to_vec()
                } else {
                    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
                    for y in 0..height as usize {
                        let row_start = y * stride;
                        let row_end = row_start + width as usize * 4;
                        pixels.extend_from_slice(&frame[row_start..row_end]);
                    }
                    pixels
                };

                match encode_frame(&bgra, width, height, jpeg_quality) {
                    Ok(frame_data) => {
                        if event_tx.blocking_send(CaptureEvent::Frame(frame_data)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.blocking_send(CaptureEvent::Error(e));
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                let _ = event_tx.blocking_send(CaptureEvent::Error(e.to_string()));
                break;
            }
        }

        let elapsed = start.elapsed();
        if elapsed < frame_interval {
            std::thread::sleep(frame_interval - elapsed);
        }
    }
    let _ = event_tx.blocking_send(CaptureEvent::Stopped);
}
