use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

pub struct BleBridge {
    uart_port: String,
}

impl BleBridge {
    pub fn new(uart_port: &str) -> Self {
        Self {
            uart_port: uart_port.to_string(),
        }
    }

    pub async fn run(self) {
        let addr = "0.0.0.0:8080";
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(Self::handle_connection(stream));
        }
    }

    async fn handle_connection(stream: tokio::net::TcpStream) {
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

        while let Some(msg) = ws_stream.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() || msg.is_binary() {
                    // Process message and interact with MCU via UART
                    ws_stream.send(msg).await.unwrap();
                }
            }
        }
    }
}
