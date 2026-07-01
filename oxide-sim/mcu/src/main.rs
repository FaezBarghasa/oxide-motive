
use anyhow::Result;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time;

mod engine;
use engine::Engine;
use oxide_protocol::{McuToHost, HostToMcu, SensorData, EngineState};
use oxide_protocol::framing::{encode_frame, decode_frame};
use heapless::Vec;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    println!("Virtual MCU listening on 127.0.0.1:6142");

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<()> {
    let mut engine = Engine::new();
    let mut interval = time::interval(Duration::from_micros(1000)); // 1ms timer
    let mut seq = 0;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                engine.step(0.5, 0.5); // Simulate constant throttle/load

                let mut sensors: Vec<SensorData, 32> = Vec::new();
                sensors.push(SensorData { id: 0, raw_value: engine.rpm as u16, physical_value: engine.rpm as f32, status: 0}).unwrap();
                sensors.push(SensorData { id: 1, raw_value: engine.map as u16, physical_value: engine.map as f32, status: 0}).unwrap();

                let telemetry = McuToHost::TelemetryBatch {
                    timestamp_us: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as u64,
                    sensors,
                    state: EngineState::Running,
                    rpm: engine.rpm,
                };

                let mut buf = [0u8; 1024];
                let encoded_len = encode_frame(&telemetry, &mut buf, seq).unwrap();
                seq += 1;
                if socket.write_all(&buf[..encoded_len]).await.is_err() {
                    break; // Connection closed
                }
            }
            result = read_from_socket(&mut socket) => {
                match result {
                    Ok(Some(msg)) => {
                        // Process HostToMcu messages here
                        println!("Received from host: {:?}", msg);
                    },
                    Ok(None) => break, // Connection closed
                    Err(_) => break,
                }
            }
        }
    }
    Ok(())
}

async fn read_from_socket(socket: &mut TcpStream) -> Result<Option<HostToMcu>> {
    let mut buf = [0u8; 1024];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(None);
    }
    let (msg, _): (HostToMcu, _) = decode_frame(&mut buf[..n])?;
    Ok(Some(msg))
}
