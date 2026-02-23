use std::pin::Pin;
use futures::Stream;
use tokio::net::TcpListener;
use tokio_util::codec::Framed;
use futures::StreamExt;
use futures::SinkExt;
use crate::protocol::ProtocolMessage;
use crate::protocol::codec::MessageCodec;
use super::NetworkEvent;

pub fn host_server_subscription(port: u16) -> iced::Subscription<NetworkEvent> {
    iced::Subscription::run_with(port, move |port| host_server_stream(*port))
}

fn host_server_stream(port: u16) -> Pin<Box<dyn Stream<Item = NetworkEvent> + Send>> {
    Box::pin(iced::stream::channel(100, move |mut output: futures::channel::mpsc::Sender<NetworkEvent>| async move {
        let addr = format!("0.0.0.0:{port}");
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

        let (stream, _addr) = match listener.accept().await {
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

        while let Some(result) = framed.next().await {
            match result {
                Ok(msg) => {
                    match msg {
                        ProtocolMessage::Disconnect => break,
                        ProtocolMessage::Ping(ts) => {
                            let _ = framed.send(ProtocolMessage::Pong(ts)).await;
                        }
                        _ => {}
                    }
                }
                Err(_) => break,
            }
        }

        let _ = output.send(NetworkEvent::ClientDisconnected).await;
        let _ = output.send(NetworkEvent::Stopped).await;

        std::future::pending::<()>().await;
    }))
}
