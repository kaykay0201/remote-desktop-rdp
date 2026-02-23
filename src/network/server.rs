use std::pin::Pin;
use std::time::Duration;
use futures::{Stream, StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio_util::codec::Framed;
use crate::protocol::ProtocolMessage;
use crate::protocol::codec::MessageCodec;
use crate::capture::{CaptureConfig, CaptureEvent, CaptureCommand};
use crate::capture::capturer::capture_loop;
use crate::input_handler::handler::InputHandler;
use super::NetworkEvent;

pub fn host_server_subscription(host: String, port: u16) -> iced::Subscription<NetworkEvent> {
    iced::Subscription::run_with((host.clone(), port), move |(host, port)| host_server_stream(host.clone(), *port))
}

fn host_server_stream(host: String, port: u16) -> Pin<Box<dyn Stream<Item = NetworkEvent> + Send>> {
    Box::pin(iced::stream::channel(100, move |mut output: futures::channel::mpsc::Sender<NetworkEvent>| async move {
        let addr = format!("{host}:{port}");
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                let _ = output.send(NetworkEvent::Error(format!("Bind failed: {e}"))).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }
        };

        let _ = output.send(NetworkEvent::Listening { port }).await;

        let (stream, client_addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                let _ = output.send(NetworkEvent::Error(format!("Accept failed: {e}"))).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }
        };

        let _ = output.send(NetworkEvent::ClientConnected).await;

        let mut framed = Framed::new(stream, MessageCodec);

        match framed.next().await {
            Some(Ok(ProtocolMessage::Hello { version, screen_width, screen_height })) => {
                tracing::info!(
                    "Client hello: version={version}, screen={screen_width}x{screen_height}, addr={client_addr}"
                );
            }
            Some(Ok(other)) => {
                tracing::warn!("Expected Hello, got: {other:?}");
            }
            Some(Err(e)) => {
                let _ = output.send(NetworkEvent::Error(format!("Read hello failed: {e}"))).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }
            None => {
                let _ = output.send(NetworkEvent::ClientDisconnected).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }
        }

        let _ = output.send(NetworkEvent::ClientInfo { addr: client_addr.to_string() }).await;

        let config = CaptureConfig::default();
        let (capture_tx, mut capture_rx) = tokio::sync::mpsc::channel::<CaptureEvent>(30);
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<CaptureCommand>(10);

        tokio::task::spawn_blocking(move || capture_loop(config, capture_tx, cmd_rx));

        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<ProtocolMessage>(100);

        tokio::task::spawn_blocking(move || {
            let mut handler = match InputHandler::new() {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("Failed to create InputHandler: {e}");
                    return;
                }
            };
            while let Some(msg) = input_rx.blocking_recv() {
                handler.apply(&msg);
            }
        });

        let mut heartbeat = tokio::time::interval(Duration::from_secs(5));
        let mut last_pong = tokio::time::Instant::now();

        let (mut sink, mut stream_reader) = framed.split();

        loop {
            tokio::select! {
                frame = capture_rx.recv() => {
                    match frame {
                        Some(CaptureEvent::Frame(data)) => {
                            if let Err(e) = sink.send(ProtocolMessage::Frame(data)).await {
                                tracing::warn!("Send frame error: {e}");
                                break;
                            }
                        }
                        Some(CaptureEvent::Started { width, height }) => {
                            tracing::info!("Capture started: {width}x{height}");
                        }
                        Some(CaptureEvent::Error(e)) => {
                            tracing::warn!("Capture error: {e}");
                        }
                        Some(CaptureEvent::Stopped) | None => break,
                    }
                }
                msg = stream_reader.next() => {
                    match msg {
                        Some(Ok(ProtocolMessage::Disconnect)) => break,
                        Some(Ok(ProtocolMessage::Ping(ts))) => {
                            let _ = sink.send(ProtocolMessage::Pong(ts)).await;
                        }
                        Some(Ok(ProtocolMessage::Pong(_))) => {
                            last_pong = tokio::time::Instant::now();
                        }
                        Some(Ok(input_msg)) => {
                            let _ = input_tx.send(input_msg).await;
                        }
                        Some(Err(e)) => {
                            tracing::warn!("Client read error: {e}");
                            break;
                        }
                        None => break,
                    }
                }
                _ = heartbeat.tick() => {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let _ = sink.send(ProtocolMessage::Ping(ts)).await;
                    if last_pong.elapsed() > Duration::from_secs(15) {
                        tracing::warn!("Client heartbeat timeout");
                        break;
                    }
                }
            }
        }

        let _ = cmd_tx.send(CaptureCommand::Stop).await;
        drop(input_tx);

        let _ = output.send(NetworkEvent::ClientDisconnected).await;
        let _ = output.send(NetworkEvent::Stopped).await;

        std::future::pending::<()>().await;
    }))
}
