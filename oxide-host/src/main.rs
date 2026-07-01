use anyhow::{Result, Context};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{broadcast, mpsc},
};
use log::{info, error, warn, debug};
use std::{path::Path, time::Duration};

use oxide_protocol::{
    framing, clock_sync::ClockSync, HostToMcu, McuToHost, TelemetryBatch, DtcEvent, EcuConfig,
};
use oxide_math::Table3D;

/// Trait for subscribing to telemetry data.
pub trait TelemetrySubscriber: Send + Sync + 'static {
    /// Called when a new telemetry batch is received.
    fn on_telemetry(&self, telemetry: TelemetryBatch);
    /// Called when a DTC event is received.
    fn on_dtc(&self, dtc: DtcEvent);
}

/// Represents a connection to the MCU.
pub struct McuConnection {
    port: tokio_serial::SerialStream,
    tx_sender: mpsc::Sender<(HostToMcu, u32)>, // (message, sequence_number)
    telemetry_broadcast_tx: broadcast::Sender<TelemetryBatch>,
    dtc_broadcast_tx: broadcast::Sender<DtcEvent>,
    config_update_receiver: mpsc::Receiver<EcuConfig>,
    table_update_receiver: mpsc::Receiver<HostToMcu>, // For TableUpdate messages
}

impl McuConnection {
    /// Establishes a connection to the MCU via a serial port.
    pub async fn connect(port_path: &str) -> Result<Self> {
        info!("Connecting to MCU on {}", port_path);
        let port = tokio_serial::SerialStream::open(
            port_path,
            &tokio_serial::SerialPortBuilder::new()
                .baud_rate(115_200)
                .data_bits(tokio_serial::DataBits::Eight)
                .parity(tokio_serial::Parity::None)
                .stop_bits(tokio_serial::StopBits::One)
                .flow_control(tokio_serial::FlowControl::None),
        )
        .context(format!("Failed to open serial port {}", port_path))?;

        let (tx_sender, tx_receiver) = mpsc::channel(100);
        let (telemetry_broadcast_tx, _) = broadcast::channel(100);
        let (dtc_broadcast_tx, _) = broadcast::channel(10);
        let (config_update_sender, config_update_receiver) = mpsc::channel(10);
        let (table_update_sender, table_update_receiver) = mpsc::channel(100);

        // Spawn the protocol handler task
        tokio::spawn(protocol_handler(
            port,
            tx_receiver,
            telemetry_broadcast_tx.clone(),
            dtc_broadcast_tx.clone(),
            config_update_sender,
            table_update_sender,
        ));

        Ok(Self {
            port,
            tx_sender,
            telemetry_broadcast_tx,
            dtc_broadcast_tx,
            config_update_receiver,
            table_update_receiver,
        })
    }

    /// Sends a message to the MCU.
    pub async fn send_message(&self, msg: HostToMcu, seq_num: u32) -> Result<()> {
        self.tx_sender
            .send((msg, seq_num))
            .await
            .context("Failed to send message to protocol handler")?;
        Ok(())
    }

    /// Subscribes to telemetry updates.
    pub fn subscribe_telemetry(&self) -> broadcast::Receiver<TelemetryBatch> {
        self.telemetry_broadcast_tx.subscribe()
    }

    /// Subscribes to DTC updates.
    pub fn subscribe_dtc(&self) -> broadcast::Receiver<DtcEvent> {
        self.dtc_broadcast_tx.subscribe()
    }

    /// Sends a configuration update to the MCU.
    pub async fn send_config_update(&self, config: EcuConfig, seq_num: u32) -> Result<()> {
        self.send_message(HostToMcu::ConfigUpdate { config }, seq_num)
            .await
    }

    /// Sends a table update to the MCU.
    pub async fn send_table_update(
        &self,
        table_id: u8,
        x_idx: u8,
        y_idx: u8,
        value: f32,
        seq_num: u32,
    ) -> Result<()> {
        self.send_message(
            HostToMcu::TableUpdate {
                table_id,
                x_idx,
                y_idx,
                value,
            },
            seq_num,
        )
        .await
    }
}

/// The main protocol handling task.
async fn protocol_handler(
    mut port: tokio_serial::SerialStream,
    mut tx_receiver: mpsc::Receiver<(HostToMcu, u32)>,
    telemetry_broadcast_tx: broadcast::Sender<TelemetryBatch>,
    dtc_broadcast_tx: broadcast::Sender<DtcEvent>,
    _config_update_sender: mpsc::Sender<EcuConfig>, // Not used directly here, but passed through
    _table_update_sender: mpsc::Sender<HostToMcu>, // Not used directly here, but passed through
) -> Result<()> {
    let mut read_buffer = vec![0u8; 512]; // Max frame size
    let mut write_buffer = vec![0u8; 512];
    let mut current_seq_num_tx = 0;
    let mut current_seq_num_rx = 0;

    let mut clock_sync = ClockSync::new(0.1); // Smoothing factor

    info!("Protocol handler started.");

    loop {
        tokio::select! {
            // Handle outgoing messages
            Some((msg, seq_num)) = tx_receiver.recv() => {
                debug!("Sending message: {:?}", msg);
                current_seq_num_tx = seq_num; // Use provided seq_num for specific messages
                let encoded_len = framing::encode_frame(&msg, current_seq_num_tx, &mut write_buffer)
                    .context("Failed to encode message")?;
                port.write_all(&write_buffer[0..encoded_len]).await
                    .context("Failed to write to serial port")?;
            },
            // Handle incoming messages
            read_result = port.read(&mut read_buffer) => {
                let bytes_read = read_result.context("Failed to read from serial port")?;
                if bytes_read == 0 {
                    warn!("Serial port disconnected.");
                    break;
                }
                debug!("Received {} bytes: {:?}", bytes_read, &read_buffer[0..bytes_read]);

                match framing::decode_frame(&read_buffer[0..bytes_read]) {
                    Ok((seq_num, mcu_msg)) => {
                        current_seq_num_rx = seq_num;
                        debug!("Received MCU message (seq {}): {:?}", seq_num, mcu_msg);

                        match mcu_msg {
                            McuToHost::TelemetryBatch { .. } => {
                                telemetry_broadcast_tx.send(mcu_msg.try_into().unwrap()).ok(); // Unwrap is safe here
                            },
                            McuToHost::DtcEvent { .. } => {
                                dtc_broadcast_tx.send(mcu_msg.try_into().unwrap()).ok(); // Unwrap is safe here
                            },
                            McuToHost::SyncResponse { timestamp_us: mcu_timestamp } => {
                                let host_rx_time = tokio::time::Instant::now().elapsed().as_micros() as u64;
                                // Assuming host_tx_time was captured when SyncRequest was sent
                                // For a full implementation, we'd need to store the host_tx_time for each SyncRequest
                                // For now, let's use a dummy host_tx_time for demonstration.
                                let dummy_host_tx_time = host_rx_time.saturating_sub(1000); // Assume 1ms round trip
                                let sync_result = clock_sync.process_sync_exchange(
                                    dummy_host_tx_time,
                                    mcu_timestamp, // MCU RX time of request
                                    mcu_timestamp, // MCU TX time of response (assuming immediate)
                                    host_rx_time,
                                );
                                info!("Clock sync updated: {:?}", sync_result);
                            },
                            McuToHost::Ack { seq } => {
                                debug!("Received ACK for sequence number {}", seq);
                                // Here, implement logic to confirm message delivery
                            },
                        }
                    },
                    Err(e) => {
                        error!("Failed to decode MCU message: {:?}", e);
                    }
                }
            },
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Send heartbeat periodically
                current_seq_num_tx = current_seq_num_tx.wrapping_add(1);
                let heartbeat_msg = HostToMcu::Heartbeat;
                let encoded_len = framing::encode_frame(&heartbeat_msg, current_seq_num_tx, &mut write_buffer)
                    .context("Failed to encode heartbeat")?;
                port.write_all(&write_buffer[0..encoded_len]).await
                    .context("Failed to write heartbeat to serial port")?;
            }
        }
    }
    Ok(())
}

/// Dummy UI task.
async fn ui_task() -> Result<()> {
    info!("UI task started.");
    // In a real application, this would be a GUI framework like egui, iced, or web UI.
    // For now, it just logs.
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        info!("UI is alive.");
    }
}

/// Dummy logger task.
async fn logger_task() -> Result<()> {
    info!("Logger task started.");
    // In a real application, this would write to a file, database, or cloud.
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        info!("Logger is alive.");
    }
}

/// Dummy connectivity task.
async fn connectivity_task() -> Result<()> {
    info!("Connectivity task started.");
    // This would handle BLE, Wi-Fi, LTE, etc.
    loop {
        tokio::time::sleep(Duration::from_secs(15)).await;
        info!("Connectivity is alive.");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init(); // Initialize logger

    info!("Oxide-Host application starting...");

    // Example: Connect to a dummy serial port or a real one
    let mcu_connection = McuConnection::connect("/dev/ttyUSB0").await?;

    // Example: Subscribe to telemetry
    let mut telemetry_rx = mcu_connection.subscribe_telemetry();
    tokio::spawn(async move {
        while let Ok(telemetry) = telemetry_rx.recv().await {
            debug!("Telemetry: RPM={}, State={:?}", telemetry.rpm, telemetry.state);
        }
    });

    // Example: Send a config update
    let initial_config = EcuConfig {
        injector_size_cc: 550,
        trigger_pattern: oxide_protocol::TriggerPattern::MissingTooth(36, 1),
        num_cylinders: 4,
        rev_limit_rpm: 8500,
        boost_cut_kpa: 200,
    };
    mcu_connection.send_config_update(initial_config, 1).await?;

    // Spawn other tasks
    let ui_handle = tokio::spawn(ui_task());
    let logger_handle = tokio::spawn(logger_task());
    let connectivity_handle = tokio::spawn(connectivity_task());

    // Wait for all tasks to complete (or for an error)
    tokio::try_join!(ui_handle, logger_handle, connectivity_handle)?;

    info!("Oxide-Host application finished.");
    Ok(())
}
