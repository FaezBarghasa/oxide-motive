
use anyhow::Result;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::fs::File;

mod ui;
mod platform;

use platform::display::{DisplayProbe, RendererTier};

// Re-exporting the protocol types for convenience
pub use oxide_protocol::{TelemetryFrame, McuToHost, HostToMcu, framing};

#[tokio::main]
async fn main() -> Result<()> {
    let renderer_tier = DisplayProbe::probe();
    DisplayProbe::set_renderer_env(renderer_tier);

    let (ui_tx, ui_rx) = mpsc::channel(100);
    let (log_tx, log_rx) = mpsc::channel(100);
    let (mcu_tx, mcu_rx) = mpsc::channel(100);

    // Task A: Serial/USB Polling
    let serial_log_tx = log_tx.clone();
    tokio::spawn(serial_task(ui_tx, serial_log_tx, mcu_rx));

    // Task B: Disk Logger
    tokio::spawn(logger_task(log_rx));

    // Task C: UI Task (runs on main thread)
    ui::ui_task(ui_rx).await;

    Ok(())
}

async fn serial_task(
    ui_tx: mpsc::Sender<TelemetryFrame>,
    log_tx: mpsc::Sender<String>,
    mut mcu_rx: mpsc::Receiver<HostToMcu>,
) {
    // This would be a real serial port
    let port_name = "/dev/ttyACM0";
    let builder = serialport::new(port_name, 115_200)
        .timeout(Duration::from_millis(10));

    match builder.open() {
        Ok(mut port) => {
            log_tx.send(format!("Opened serial port {}", port_name)).await.ok();
            let mut serial_buf = [0u8; 1024];
            loop {
                match port.read(&mut serial_buf) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            // Here we would decode COBS and deserialize the frame
                            // For now, we'll just log the raw data.
                            log_tx.send(format!("Read {} bytes", bytes_read)).await.ok();
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        log_tx.send(format!("Serial port error: {}", e)).await.ok();
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        Err(e) => {
            log_tx.send(format!("Failed to open serial port {}: {}", port_name, e)).await.ok();
        }
    }
}

async fn logger_task(mut rx: mpsc::Receiver<String>) {
    let file = File::create("host.log").await.unwrap();
    let mut writer = BufWriter::new(file);

    while let Some(log_entry) = rx.recv().await {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let formatted_entry = format!("[{}] {}\n", timestamp, log_entry);
        if writer.write_all(formatted_entry.as_bytes()).await.is_err() {
            eprintln!("Failed to write to log file.");
        }
        writer.flush().await.ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_channel_communication() {
        let (tx, mut rx) = mpsc::channel(10);

        tokio::spawn(async move {
            tx.send("test".to_string()).await.unwrap();
        });

        let received = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
        assert_eq!(received, "test");
    }
}
