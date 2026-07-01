use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::sleep,
};

use oxide_protocol::{
    framing::{decode_frame, encode_frame},
    HostToMcu, McuToHost, SensorData,
};
use heapless::Vec;

async fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0u8; 256];

    loop {
        match stream.read(&mut buf).await {
            Ok(n) if n > 0 => {
                if let Ok(msg) = decode_frame::<HostToMcu>(&buf[1..n]) {
                    println!("MCU received: {:?}", msg);
                    match msg {
                        HostToMcu::SyncRequest => {
                            let mut response_buf = [0u8; 256];
                            let response = McuToHost::SyncResponse;
                            let len = encode_frame(&response, &mut response_buf).unwrap();
                            stream.write_all(&response_buf[..len]).await.unwrap();
                        }
                        HostToMcu::ScheduleEvent { .. } => {
                            // In a real sim, we'd schedule this. For now, just Ack.
                            let mut response_buf = [0u8; 256];
                            let response = McuToHost::Ack;
                            let len = encode_frame(&response, &mut response_buf).unwrap();
                            stream.write_all(&response_buf[..len]).await.unwrap();
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                println!("MCU connection closed");
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Virtual MCU listening on 127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream));
    }
}
