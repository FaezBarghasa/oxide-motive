use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use toml::Value;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hardware_config.rs");
    let mut f = File::create(&dest_path).unwrap();

    let hardware_toml_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("hardware.toml");
    let hardware: Value = toml::from_str(&std::fs::read_to_string(hardware_toml_path).unwrap()).unwrap();

    let crank_pin = hardware["pins"]["crank_pin"].as_str().unwrap();
    let injector_1_pin = hardware["pins"]["injector_1_pin"].as_str().unwrap();

    write!(f, "pub const CRANK_PIN: &str = \"{}\";\n", crank_pin).unwrap();
    write!(f, "pub const INJECTOR_1_PIN: &str = \"{}\";\n", injector_1_pin).unwrap();
}
