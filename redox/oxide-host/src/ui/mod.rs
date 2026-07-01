use slint::{SharedString, ComponentHandle};

slint::include_modules!();

pub fn run_dashboard() {
    let ui = AppWindow::new().unwrap();

    let ui_handle = ui.as_weak();
    std::thread::spawn(move || {
        loop {
            // TODO: Receive telemetry from IPC
            let rpm = 5000.0; // mock data
            let boost = 12.5;
            let afr = 11.8;
            let coolant = 95.0;

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
