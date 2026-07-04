use slint::{Model, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn main() {
    std::env::set_var("SLINT_BACKEND", "linuxkms");
    std::env::set_var("SLINT_DRM_DEVICE", "/dev/dri/card1");

    let main_window = MainWindow::new().unwrap();
    let main_window_weak = main_window.as_weak();

    ctrlc::set_handler(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            main_window.hide().unwrap();
        }
    })
    .expect("Error setting Ctrl-C handler");

    let sensors = Rc::new(VecModel::from(vec![
        Sensor{ name: "RPM".into(), value: 0.0 },
        Sensor{ name: "Boost".into(), value: 0.0 },
        Sensor{ name: "Lambda".into(), value: 0.0 },
    ]));
    main_window.set_sensors(sensors.clone().into());

    main_window.run().unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ui_init() {
        std::env::set_var("SLINT_BACKEND", "linuxkms");
        std::env::set_var("SLINT_DRM_DEVICE", "/dev/dri/card1");
        // This test only checks that the environment variables can be set.
        // It does not attempt to initialize the UI.
    }
}
