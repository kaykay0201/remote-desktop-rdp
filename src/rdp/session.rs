use std::time::Duration;

use futures::Stream;
use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::StreamExt;
use ironrdp::session::image::DecodedImage;
use ironrdp::session::ActiveStage;
use ironrdp::session::ActiveStageOutput;
use ironrdp_tokio::FramedWrite;
use tracing::{error, info};

use crate::config::ConnectionProfile;
use crate::rdp::input::translate_command;
use crate::rdp::{ConnectionStatus, InputCommand, RdpConnection, RdpEvent};

pub fn rdp_subscription(profile: ConnectionProfile) -> impl Stream<Item = RdpEvent> {
    iced::stream::channel(100, async move |mut output| {
        let _ = output
            .send(RdpEvent::StatusChanged(ConnectionStatus::Connecting))
            .await;

        let (framed, connection_result) = match tokio::time::timeout(
            Duration::from_secs(30),
            crate::rdp::connection::establish_connection(&profile),
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let _ = output
                    .send(RdpEvent::Error(format!("Connection failed: {e}")))
                    .await;
                return;
            }
            Err(_) => {
                let _ = output
                    .send(RdpEvent::Error(
                        "Connection timed out after 30 seconds".to_string(),
                    ))
                    .await;
                return;
            }
        };

        let (input_tx, mut input_rx) = mpsc::channel::<InputCommand>(100);
        let _ = output
            .send(RdpEvent::Connected(RdpConnection::new(input_tx)))
            .await;

        let desktop_size = connection_result.desktop_size;
        let mut active_stage = ActiveStage::new(connection_result);
        let mut image = DecodedImage::new(
            ironrdp::graphics::image_processing::PixelFormat::RgbA32,
            desktop_size.width,
            desktop_size.height,
        );

        let mut input_db = ironrdp::input::Database::new();

        let (mut framed_read, mut framed_write) = ironrdp_tokio::split_tokio_framed(framed);

        info!("RDP session active, entering main loop");

        loop {
            tokio::select! {
                pdu_result = tokio::time::timeout(Duration::from_secs(60), framed_read.read_pdu()) => {
                    match pdu_result {
                        Ok(Ok((action, payload))) => {
                            match active_stage.process(&mut image, action, &payload) {
                                Ok(outputs) => {
                                    let mut frame_updated = false;
                                    for stage_output in outputs {
                                        match stage_output {
                                            ActiveStageOutput::ResponseFrame(frame) => {
                                                if !frame.is_empty()
                                                    && let Err(e) = framed_write.write_all(&frame).await
                                                {
                                                    error!("Failed to write response frame: {e}");
                                                    let _ = output.send(RdpEvent::Error(
                                                        format!("Write error: {e}"),
                                                    )).await;
                                                    return;
                                                }
                                            }
                                            ActiveStageOutput::GraphicsUpdate(_) => {
                                                frame_updated = true;
                                            }
                                            ActiveStageOutput::Terminate(reason) => {
                                                info!("Server terminated session: {reason}");
                                                let _ = output.send(RdpEvent::Disconnected).await;
                                                return;
                                            }
                                            ActiveStageOutput::DeactivateAll(_) => {
                                                info!("Deactivation-reactivation sequence");
                                            }
                                            _ => {}
                                        }
                                    }
                                    if frame_updated {
                                        let _ = output.send(RdpEvent::Frame {
                                            width: u32::from(image.width()),
                                            height: u32::from(image.height()),
                                            pixels: image.data().to_vec(),
                                        }).await;
                                    }
                                }
                                Err(e) => {
                                    error!("Session processing error: {e}");
                                    let _ = output.send(RdpEvent::Error(format!("Session error: {e}"))).await;
                                    return;
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Read PDU error: {e}");
                            let _ = output.send(RdpEvent::Error(format!("Read error: {e}"))).await;
                            return;
                        }
                        Err(_) => {
                            error!("Read PDU timed out after 60 seconds");
                            let _ = output.send(RdpEvent::Error(
                                "Connection timed out — no data received for 60 seconds".to_string(),
                            )).await;
                            return;
                        }
                    }
                }
                cmd = input_rx.next() => {
                    match cmd {
                        Some(InputCommand::Disconnect) => {
                            info!("User requested disconnect");
                            let _ = output.send(RdpEvent::Disconnected).await;
                            return;
                        }
                        Some(cmd) => {
                            let events = translate_command(&mut input_db, cmd);
                            if !events.is_empty() {
                                match active_stage.process_fastpath_input(&mut image, &events) {
                                    Ok(outputs) => {
                                        for stage_output in outputs {
                                            if let ActiveStageOutput::ResponseFrame(frame) = stage_output
                                                && !frame.is_empty()
                                                && let Err(e) = framed_write.write_all(&frame).await
                                            {
                                                error!("Failed to send input: {e}");
                                                let _ = output.send(RdpEvent::Error(
                                                    format!("Input send error: {e}"),
                                                )).await;
                                                return;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Input processing error: {e}");
                                    }
                                }
                            }
                        }
                        None => {
                            info!("Input channel closed, disconnecting");
                            let _ = output.send(RdpEvent::Disconnected).await;
                            return;
                        }
                    }
                }
            }
        }
    })
}
