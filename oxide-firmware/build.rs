
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod memory_maps;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let layouts = memory_maps::get_memory_layouts();

    let feature = env::vars()
        .map(|(key, _)| key)
        .find(|key| key.starts_with("CARGO_FEATURE_"))
        .map(|key| key.trim_start_matches("CARGO_FEATURE_").to_lowercase())
        .expect("No MCU feature flag enabled. Please enable one, e.g., --features stm32h750");

    let mcu_feature = feature.split(',').find(|f| layouts.contains_key(f)).expect("No valid MCU feature found");

    let layout = layouts.get(mcu_feature).unwrap();

    let mut memory_x = String::new();
    memory_x.push_str("MEMORY\n");
    memory_x.push_str("{\n");
    memory_x.push_str(&format!(
        "  FLASH : ORIGIN = 0x{:08X}, LENGTH = {}\n",
        layout.flash_origin, layout.flash_len
    ));
    memory_x.push_str(&format!(
        "  RAM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
        layout.ram_origin, layout.ram_len
    ));
    if let (Some(origin), Some(len)) = (layout.itcm_origin, layout.itcm_len) {
        memory_x.push_str(&format!(
            "  ITCM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
    if let (Some(origin), Some(len)) = (layout.dtcm_origin, layout.dtcm_len) {
        memory_x.push_str(&format!(
            "  DTCM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
     if let (Some(origin), Some(len)) = (layout.axi_origin, layout.axi_len) {
        memory_x.push_str(&format!(
            "  AXI : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
    if let (Some(origin), Some(len)) = (layout.ccm_origin, layout.ccm_len) {
        memory_x.push_str(&format!(
            "  CCM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
    memory_x.push_str("}\n");

    let mut file = File::create(out_dir.join("memory.x")).unwrap();
    file.write_all(memory_x.as_bytes()).unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-arg=-Tmemory.x");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=memory_maps.rs");
}

#[test]
fn test_generate_stm32h750_memory_x() {
    let layouts = memory_maps::get_memory_layouts();
    let layout = layouts.get("stm32h750").unwrap();

    let mut memory_x = String::new();
    memory_x.push_str("MEMORY\n");
    memory_x.push_str("{\n");
    memory_x.push_str(&format!(
        "  FLASH : ORIGIN = 0x{:08X}, LENGTH = {}\n",
        layout.flash_origin, layout.flash_len
    ));
    memory_x.push_str(&format!(
        "  RAM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
        layout.ram_origin, layout.ram_len
    ));
    if let (Some(origin), Some(len)) = (layout.itcm_origin, layout.itcm_len) {
        memory_x.push_str(&format!(
            "  ITCM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
    if let (Some(origin), Some(len)) = (layout.dtcm_origin, layout.dtcm_len) {
        memory_x.push_str(&format!(
            "  DTCM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
     if let (Some(origin), Some(len)) = (layout.axi_origin, layout.axi_len) {
        memory_x.push_str(&format!(
            "  AXI : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
    if let (Some(origin), Some(len)) = (layout.ccm_origin, layout.ccm_len) {
        memory_x.push_str(&format!(
            "  CCM : ORIGIN = 0x{:08X}, LENGTH = {}\n",
            origin, len
        ));
    }
    memory_x.push_str("}\n");

    assert!(memory_x.contains("FLASH : ORIGIN = 0x08000000, LENGTH = 131072"));
    assert!(memory_x.contains("RAM : ORIGIN = 0x24000000, LENGTH = 524288"));
    assert!(memory_x.contains("DTCM : ORIGIN = 0x20000000, LENGTH = 131072"));
}
