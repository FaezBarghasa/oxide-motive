
use anyhow::Result;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time;
use oxide_protocol::{HostToMcu, McuToHost};
use oxide_protocol::framing::{encode_frame, decode_frame};
use oxide_protocol::clock_sync::ClockSync;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    println!("Virtual host connected to MCU");

    let mut clock_sync = ClockSync::new();
    let mut seq = 0;

    let mut heartbeat_interval = time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                let msg = HostToMcu::Heartbeat;
                let mut buf = [0u8; 128];
                let len = encode_frame(&msg, &mut buf, seq).unwrap();
                seq += 1;
                stream.write_all(&buf[..len]).await?;
            }
            result = read_from_mcu(&mut stream) => {
                match result {
                    Ok(Some(msg)) => {
                        if let McuToHost::TelemetryBatch { rpm, .. } = msg {
                            println!("RPM: {}", rpm);
                        }
                    },
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("Error reading from MCU: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn read_from_mcu(stream: &mut TcpStream) -> Result<Option<McuToHost>> {
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(None);
    }
    let (msg, _): (McuToHost, _) = decode_frame(&mut buf[..n])?;
    Ok(Some(msg))
}
