use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

async fn handle_websocket_connection(stream: TcpStream) {
    let mut ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.expect("Failed to get message");
        if msg.is_text() || msg.is_binary() {
            // Process API requests from the WebSocket and respond with acknowledgement
            let response = match msg {
                tokio_tungstenite::tungstenite::Message::Text(ref text) => tokio_tungstenite::tungstenite::Message::Text(format!("Ack: {}", text)),
                _ => msg.clone(),
            };
            ws_stream.send(response).await.expect("Failed to send message");
        }
    }
}

#[tokio::main]
pub async fn run_ble_bridge() {
    let addr = "127.0.0.1:9002".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_websocket_connection(stream));
    }
}
