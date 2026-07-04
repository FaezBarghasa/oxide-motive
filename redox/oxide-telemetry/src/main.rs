use std::os::unix::net::UnixListener;
use std::io::Write;
use std::time::Duration;

fn main() {
    println!("oxide-telemetry started");

    let socket_path = "/tmp/oxide_telemetry.sock";
    let _ = std::fs::remove_file(socket_path); // Clean up socket file if it previously crashed
    let listener = UnixListener::bind(socket_path).expect("Failed to bind to socket");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    let mut rpm: f32 = 1000.0;
                    let mut boost: f32 = 0.0;
                    let mut afr: f32 = 14.7;
                    let mut coolant: f32 = 80.0;

                    loop {
                        rpm = (rpm + 50.0).rem_euclid(7000.0);
                        boost = (boost + 0.1).rem_euclid(20.0);
                        afr = 14.7 + (rpm / 7000.0) * 2.0 - 1.0;
                        coolant = (coolant + 0.01).rem_euclid(120.0);

                        let mut buffer = [0u8; 16];
                        buffer[0..4].copy_from_slice(&rpm.to_le_bytes());
                        buffer[4..8].copy_from_slice(&boost.to_le_bytes());
                        buffer[8..12].copy_from_slice(&afr.to_le_bytes());
                        buffer[12..16].copy_from_slice(&coolant.to_le_bytes());

                        if stream.write_all(&buffer).is_err() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(33)); // ~30 fps update rate
                    }
                });
            }
            Err(err) => {
                eprintln!("Failed to accept connection: {}", err);
            }
        }
    }
}
