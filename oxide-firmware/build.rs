use std::env;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use quote::quote;

#[derive(Deserialize)]
struct HardwareConfig {
    pins: Pins,
}

#[derive(Deserialize)]
struct Pins {
    crank_pin: String,
    cam_pin: String,
    injector_1_pin: String,
    injector_2_pin: String,
    injector_3_pin: String,
    injector_4_pin: String,
    coil_1_pin: String,
    coil_2_pin: String,
    coil_3_pin: String,
    coil_4_pin: String,
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hardware_config.rs");

    let config_str = fs::read_to_string("hardware.toml").unwrap();
    let config: HardwareConfig = toml::from_str(&config_str).unwrap();

    let crank_pin = format_pin(&config.pins.crank_pin);
    let cam_pin = format_pin(&config.pins.cam_pin);
    // ... and so on for all pins

    let generated_code = quote! {
        // This is a simplified example. A real implementation would be more robust.
        pub const CRANK_PIN_PORT: char = #crank_pin.0;
        pub const CRANK_PIN_NUM: u8 = #crank_pin.1;
    };

    fs::write(&dest_path, generated_code.to_string()).unwrap();
    println!("cargo:rerun-if-changed=hardware.toml");
}

fn format_pin(pin_str: &str) -> (char, u8) {
    let port = pin_str.chars().nth(1).unwrap();
    let num = pin_str[2..].parse::<u8>().unwrap();
    (port, num)
}
