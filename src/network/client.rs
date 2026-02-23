use std::pin::Pin;
use std::time::Duration;
use futures::Stream;
use tokio::net::TcpStream;
use tokio::time;
use tokio_util::codec::Framed;
use futures::StreamExt;
use futures::SinkExt;
use crate::protocol::{ProtocolMessage, PROTOCOL_VERSION};
use crate::protocol::codec::MessageCodec;
use super::{NetworkEvent, ConnectionHandle};

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn access_client_subscription(host: String, port: u16) -> iced::Subscription<NetworkEvent> {
    iced::Subscription::run_with(
        (host.clone(), port),
        move |(host, port)| access_client_stream(host.clone(), *port),
    )
}

fn access_client_stream(host: String, port: u16) -> Pin<Box<dyn Stream<Item = NetworkEvent> + Send>> {
    Box::pin(iced::stream::channel(100, move |mut output: futures::channel::mpsc::Sender<NetworkEvent>| async move {
        let addr = format!("{host}:{port}");

        let (sw, sh) = scrap::Display::primary()
            .map(|d| (d.width() as u32, d.height() as u32))
            .unwrap_or((1920, 1080));

        let mut framed = None;
        let max_attempts = 3u32;

        for attempt in 1..=max_attempts {
            let stream = match time::timeout(
                Duration::from_secs(10),
                TcpStream::connect(&addr),
            ).await {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    if attempt < max_attempts {
                        let _ = output.send(NetworkEvent::Error(
                            format!("Connect failed: {e} — Retrying (attempt {attempt}/{max_attempts})...")
                        )).await;
                        time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                        continue;
                    }
                    let _ = output.send(NetworkEvent::Error(format!("Connect failed after {max_attempts} attempts: {e}"))).await;
                    let _ = output.send(NetworkEvent::Stopped).await;
                    std::future::pending::<()>().await;
                    return;
                }
                Err(_) => {
                    if attempt < max_attempts {
                        let _ = output.send(NetworkEvent::Error(
                            format!("Connection timed out — Retrying (attempt {attempt}/{max_attempts})...")
                        )).await;
                        time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                        continue;
                    }
                    let _ = output.send(NetworkEvent::Error(format!("Connection timed out after {max_attempts} attempts"))).await;
                    let _ = output.send(NetworkEvent::Stopped).await;
                    std::future::pending::<()>().await;
                    return;
                }
            };

            let mut f = Framed::new(stream, MessageCodec);

            let hello = ProtocolMessage::Hello {
                version: PROTOCOL_VERSION,
                screen_width: sw,
                screen_height: sh,
            };
            if let Err(e) = f.send(hello).await {
                if attempt < max_attempts {
                    let _ = output.send(NetworkEvent::Error(
                        format!("Send Hello failed: {e} — Retrying (attempt {attempt}/{max_attempts})...")
                    )).await;
                    time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                    continue;
                }
                let _ = output.send(NetworkEvent::Error(format!("Send Hello failed after {max_attempts} attempts: {e}"))).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }

            framed = Some(f);
            break;
        }

        let framed = match framed {
            Some(f) => f,
            None => {
                let _ = output.send(NetworkEvent::Error("Connection failed after all retries".to_string())).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }
        };

        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<ProtocolMessage>(100);
        let handle = ConnectionHandle::new(input_tx);
        let _ = output.send(NetworkEvent::Connected(handle)).await;

        let (mut sink, mut stream_reader) = framed.split();

        let mut heartbeat = time::interval(Duration::from_secs(5));
        heartbeat.tick().await;
        let mut last_pong = time::Instant::now();

        loop {
            tokio::select! {
                msg = stream_reader.next() => {
                    match msg {
                        Some(Ok(ProtocolMessage::Frame(frame_data))) => {
                            match crate::capture::encoder::decode_frame(&frame_data) {
                                Ok(pixels) => {
                                    let _ = output.send(NetworkEvent::Frame {
                                        width: frame_data.width,
                                        height: frame_data.height,
                                        pixels,
                                    }).await;
                                }
                                Err(e) => {
                                    tracing::warn!("Frame decode error: {e}");
                                }
                            }
                        }
                        Some(Ok(ProtocolMessage::Pong(ts))) => {
                            last_pong = time::Instant::now();
                            let rtt_ms = now_ms().saturating_sub(ts);
                            let _ = output.send(NetworkEvent::LatencyUpdate { rtt_ms }).await;
                        }
                        Some(Ok(ProtocolMessage::Disconnect)) | None => break,
                        Some(Err(e)) => {
                            let _ = output.send(NetworkEvent::Error(e.to_string())).await;
                            break;
                        }
                        _ => {}
                    }
                }
                input = input_rx.recv() => {
                    match input {
                        Some(msg) => {
                            if let Err(e) = sink.send(msg).await {
                                let _ = output.send(NetworkEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                        None => break,
                    }
                }
                _ = heartbeat.tick() => {
                    if last_pong.elapsed() > Duration::from_secs(15) {
                        let _ = output.send(NetworkEvent::Error("Server heartbeat timeout".to_string())).await;
                        break;
                    }
                    if let Err(e) = sink.send(ProtocolMessage::Ping(now_ms())).await {
                        let _ = output.send(NetworkEvent::Error(format!("Ping failed: {e}"))).await;
                        break;
                    }
                }
            }
        }

        let _ = output.send(NetworkEvent::Stopped).await;
        std::future::pending::<()>().await;
    }))
}
