//! Slint UI Initialization for Direct DRM/KMS Rendering
use slint::{ComponentHandle, ModelRc, SharedString};

slint::include_modules!();

pub fn launch_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    // Force Linux KMS backend for headless SBC (RPi5/OrangePi)
    std::env::set_var("SLINT_BACKEND", "linuxkms");
    std.env::set_var("SLINT_DRM_DEVICE", "/dev/dri/card1");

    let ui = Dashboard::new()?;

    // Bind initial state
    ui.set_rpm_text(SharedString::from("0 RPM"));
    ui.set_status_text(SharedString::from("Connecting to MCU..."));

    // Graceful shutdown handler
    ctrlc::set_handler({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(handle) = ui_handle.upgrade() {
                handle.window().hide().unwrap_or(());
            }
            std::process::exit(0);
        }
    }).expect("Error setting Ctrl-C handler");

    ui.run()?;
    Ok(())
}
