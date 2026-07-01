use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::sleep,
};

use oxide_protocol::{
    framing::{decode_frame, encode_frame},
    HostToMcu, McuToHost,
};

#[tokio::main]
async fn main() {
    sleep(Duration::from_secs(1)).await; // Wait for MCU to start
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    println!("Virtual Host connected to MCU");

    let mut buf = [0u8; 256];

    // Send a sync request
    let sync_request = HostToMcu::SyncRequest;
    let len = encode_frame(&sync_request, &mut buf).unwrap();
    stream.write_all(&buf[..len]).await.unwrap();

    // Wait for sync response
    match stream.read(&mut buf).await {
        Ok(n) if n > 0 => {
            if let Ok(msg) = decode_frame::<McuToHost>(&buf[1..n]) {
                println!("Host received: {:?}", msg);
            }
        }
        _ => {}
    }

    // Schedule an event
    let schedule_event = HostToMcu::ScheduleEvent {
        channel: 1,
        timestamp_us: 12345,
        duration_us: 100,
    };
    let len = encode_frame(&schedule_event, &mut buf).unwrap();
    stream.write_all(&buf[..len]).await.unwrap();

    // Wait for ack
    match stream.read(&mut buf).await {
        Ok(n) if n > 0 => {
            if let Ok(msg) = decode_frame::<McuToHost>(&buf[1..n]) {
                println!("Host received: {:?}", msg);
            }
        }
        _ => {}
    }

    println!("Host finished");
}
