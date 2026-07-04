use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("oxide-logger started");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("telemetry.log")
        .expect("Failed to open log file");

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let log_entry = format!("{}: Oxide-logger recorded active heartbeat\n", timestamp);
        if let Err(e) = file.write_all(log_entry.as_bytes()) {
            eprintln!("Failed to write to log file: {}", e);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
