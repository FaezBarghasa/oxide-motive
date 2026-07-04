use std::env;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use quote::quote;

#[derive(Deserialize)]
struct HardwareConfig {
    board: Board,
    pins: Pins,
    peripherals: Peripherals,
}

#[derive(Deserialize)]
struct Board {
    name: String,
    mcu: String,
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

#[derive(Deserialize)]
struct Peripherals {
    uart_protocol: String,
    can_bus: String,
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hardware_config.rs");

    let config_str = fs::read_to_string("hardware.toml").unwrap();
    let config: HardwareConfig = toml::from_str(&config_str).unwrap();

    let mcu_feature = format!("feature = \"{}\"", config.board.mcu.to_lowercase());

    let crank_pin_ident = format_ident!("{}", config.pins.crank_pin.to_lowercase());
    let cam_pin_ident = format_ident!("{}", config.pins.cam_pin.to_lowercase());
    let injector_1_pin_ident = format_ident!("{}", config.pins.injector_1_pin.to_lowercase());
    let injector_2_pin_ident = format_ident!("{}", config.pins.injector_2_pin.to_lowercase());
    let injector_3_pin_ident = format_ident!("{}", config.pins.injector_3_pin.to_lowercase());
    let injector_4_pin_ident = format_ident!("{}", config.pins.injector_4_pin.to_lowercase());
    let coil_1_pin_ident = format_ident!("{}", config.pins.coil_1_pin.to_lowercase());
    let coil_2_pin_ident = format_ident!("{}", config.pins.coil_2_pin.to_lowercase());
    let coil_3_pin_ident = format_ident!("{}", config.pins.coil_3_pin.to_lowercase());
    let coil_4_pin_ident = format_ident!("{}", config.pins.coil_4_pin.to_lowercase());

    let uart_protocol_ident = format_ident!("{}", config.peripherals.uart_protocol);
    let can_bus_ident = format_ident!("{}", config.peripherals.can_bus);

    let generated_code = quote! {
        #[cfg(#mcu_feature)]
        pub mod pins {
            use stm32h7xx_hal::gpio::{self, Analog};
            pub type CrankPin = gpio::#crank_pin_ident<Analog>;
            pub type CamPin = gpio::#cam_pin_ident<Analog>;
            pub type Injector1Pin = gpio::#injector_1_pin_ident<Analog>;
            pub type Injector2Pin = gpio::#injector_2_pin_ident<Analog>;
            pub type Injector3Pin = gpio::#injector_3_pin_ident<Analog>;
            pub type Injector4Pin = gpio::#injector_4_pin_ident<Analog>;
            pub type Coil1Pin = gpio::#coil_1_pin_ident<Analog>;
            pub type Coil2Pin = gpio::#coil_2_pin_ident<Analog>;
            pub type Coil3Pin = gpio::#coil_3_pin_ident<Analog>;
            pub type Coil4Pin = gpio::#coil_4_pin_ident<Analog>;
        }

        #[cfg(#mcu_feature)]
        pub mod peripherals {
             use stm32h7xx_hal::pac::{#uart_protocol_ident as UartProtocol, #can_bus_ident as CanBus};
        }
    };

    fs::write(&dest_path, generated_code.to_string()).unwrap();
    println!("cargo:rerun-if-changed=hardware.toml");
    println!("cargo:rerun-if-changed=build.rs");
}

fn format_ident(s: &str) -> proc_macro2::Ident {
    quote::format_ident!("{}", s)
}
