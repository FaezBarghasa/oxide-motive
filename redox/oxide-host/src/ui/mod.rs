use slint::{SharedString, ComponentHandle};

slint::include_modules!();

pub fn run_dashboard() {
    let ui = AppWindow::new().unwrap();

    let ui_handle = ui.as_weak();
    std::thread::spawn(move || {
        loop {
            let mut rpm = 5000.0; // default mock data fallback
            let mut boost = 12.5;
            let mut afr = 11.8;
            let mut coolant = 95.0;

            // Receive telemetry from IPC via Unix Socket
            if let Ok(mut stream) = std::os::unix::net::UnixStream::connect("/tmp/oxide_telemetry.sock") {
                let mut buffer = [0u8; 16];
                if std::io::Read::read_exact(&mut stream, &mut buffer).is_ok() {
                    rpm = f32::from_le_bytes(buffer[0..4].try_into().unwrap());
                    boost = f32::from_le_bytes(buffer[4..8].try_into().unwrap());
                    afr = f32::from_le_bytes(buffer[8..12].try_into().unwrap());
                    coolant = f32::from_le_bytes(buffer[12..16].try_into().unwrap());
                }
            }

            ui_handle.upgrade_in_event_loop(move |ui| {
                ui.set_rpm(rpm as i32);
                ui.set_boost(boost as i32);
                ui.set_afr(afr as i32);
                ui.set_coolant(coolant as i32);
            }).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(33)); // ~30fps
        }
    });

    ui.run().unwrap();
}
