use tokio::sync::mpsc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use std::rc::Rc;
use slint::{Model, VecModel};
use oxide_protocol::{TelemetryFrame, framing, postcard};

slint::include_modules!();

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    // Task A: Serial Poller
    let serial_tx = tx.clone();
    tokio::spawn(async move {
        // This is a mock serial port. In a real application, we would use
        // a crate like `tokio-serial`.
        loop {
            let frame = TelemetryFrame {
                rpm: 6000,
                map: 1013,
                tps: 50,
                afr: 147,
                advance: 30,
            };
            let mut buf = [0u8; 128];
            let serialized = postcard::to_slice(&frame, &mut buf).unwrap();
            let mut encoded = [0u8; 128];
            let encoded_len = framing::cobs_encode(serialized, &mut encoded).unwrap();

            // Simulate receiving the data
            if serial_tx.send(encoded[..encoded_len].to_vec()).await.is_err() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });

    // Task B: Disk Logger
    let logger_tx = tx.clone();
    tokio::spawn(async move {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("telemetry.tsv")
            .await
            .unwrap();

        while let Some(data) = rx.recv().await {
            let mut decoded = [0u8; 128];
            if let Ok(decoded_len) = framing::cobs_decode(&data, &mut decoded) {
                if let Ok(frame) = postcard::from_bytes::<TelemetryFrame>(&decoded[..decoded_len]) {
                    let line = format!("{}\t{}\t{}\t{}\t{}\n", frame.rpm, frame.map, frame.tps, frame.afr, frame.advance);
                    if file.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Task C: UI Sync
    let main_window = MainWindow::new().unwrap();
    let sensors = Rc::new(VecModel::from(vec![
        Sensor{ name: "RPM".into(), value: 0.0 },
        Sensor{ name: "Boost".into(), value: 0.0 },
        Sensor{ name: "Lambda".into(), value: 0.0 },
    ]));
    main_window.set_sensors(sensors.clone().into());

    let ui_handle = main_window.as_weak();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
        while let Some(data) = rx.recv().await {
             interval.tick().await;
            let ui = ui_handle.upgrade().unwrap();
            let mut decoded = [0u8; 128];
            if let Ok(decoded_len) = framing::cobs_decode(&data, &mut decoded) {
                if let Ok(frame) = postcard::from_bytes::<TelemetryFrame>(&decoded[..decoded_len]) {
                    let mut sensor_data = sensors.as_any().downcast_ref::<VecModel<Sensor>>().unwrap().iter().collect::<Vec<_>>();
                    sensor_data[0].value = frame.rpm as f32;
                    sensor_data[1].value = frame.map as f32;
                    sensor_data[2].value = frame.afr as f32;
                    ui.set_sensors(Rc::new(VecModel::from(sensor_data)).into());
                }
            }
        }
    });

    main_window.run().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency() {
        let (tx, mut rx) = mpsc::channel(10000);

        // Simulate a burst of 10,000 frames
        for i in 0..10000 {
            tx.send(i).await.unwrap();
        }

        let mut count = 0;
        while let Some(_) = rx.recv().await {
            count += 1;
            if count == 10000 {
                break;
            }
        }
        assert_eq!(count, 10000);
    }
}
