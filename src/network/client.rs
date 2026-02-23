use std::pin::Pin;
use futures::Stream;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::StreamExt;
use futures::SinkExt;
use crate::protocol::{ProtocolMessage, PROTOCOL_VERSION};
use crate::protocol::codec::MessageCodec;
use super::{NetworkEvent, ConnectionHandle};

pub fn access_client_subscription(host: String, port: u16) -> iced::Subscription<NetworkEvent> {
    iced::Subscription::run_with(
        (host.clone(), port),
        move |(host, port)| access_client_stream(host.clone(), *port),
    )
}

fn access_client_stream(host: String, port: u16) -> Pin<Box<dyn Stream<Item = NetworkEvent> + Send>> {
    Box::pin(iced::stream::channel(100, move |mut output: futures::channel::mpsc::Sender<NetworkEvent>| async move {
        let addr = format!("{host}:{port}");
        let stream = match TcpStream::connect(&addr).await {
            Ok(s) => s,
            Err(e) => {
                let _ = output.send(NetworkEvent::Error(format!("Connect failed: {e}"))).await;
                let _ = output.send(NetworkEvent::Stopped).await;
                std::future::pending::<()>().await;
                return;
            }
        };

        let mut framed = Framed::new(stream, MessageCodec);

        let hello = ProtocolMessage::Hello {
            version: PROTOCOL_VERSION,
            screen_width: 0,
            screen_height: 0,
        };
        if let Err(e) = framed.send(hello).await {
            let _ = output.send(NetworkEvent::Error(format!("Send Hello failed: {e}"))).await;
            let _ = output.send(NetworkEvent::Stopped).await;
            std::future::pending::<()>().await;
            return;
        }

        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<ProtocolMessage>(100);
        let handle = ConnectionHandle::new(input_tx);
        let _ = output.send(NetworkEvent::Connected(handle)).await;

        let (mut sink, mut stream_reader) = framed.split();

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
                        Some(Ok(ProtocolMessage::Pong(_))) => {}
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
            }
        }

        let _ = output.send(NetworkEvent::Stopped).await;
        std::future::pending::<()>().await;
    }))
}
