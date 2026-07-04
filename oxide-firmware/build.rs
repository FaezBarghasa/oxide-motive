//! Automated Linker Script Generator for Universal MCU Support
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

struct MemoryLayout {
    flash_origin: u32,
    flash_len: u32,
    ram_origin: u32,
    ram_len: u32,
    itcm_origin: Option<u32>,
    itcm_len: Option<u32>,
}

fn get_layout() -> MemoryLayout {
    // STM32H750 (Default)
    let mut layout = MemoryLayout {
        flash_origin: 0x08008000, flash_len: 0x18000, // 96K (32K reserved for bootloader)
        ram_origin: 0x20000000, ram_len: 0x20000,     // 128K DTCM
        itcm_origin: Some(0x00000000), itcm_len: Some(0x10000), // 64K ITCM
    };

    if cfg!(feature = "stm32f407") {
        layout = MemoryLayout {
            flash_origin: 0x08008000, flash_len: 0x100000, // 1M
            ram_origin: 0x20000000, ram_len: 0x20000,      // 128K SRAM
            itcm_origin: None, itcm_len: None,             // No ITCM on M4
        };
    } else if cfg!(feature = "nrf9160") {
        layout = MemoryLayout {
            flash_origin: 0x00008000, flash_len: 0xF8000,  // 1M (32K bootloader)
            ram_origin: 0x20000000, ram_len: 0x40000,      // 256K RAM
            itcm_origin: None, itcm_len: None,
        };
    }
    // Add NXP LPC55, Renesas RA8, etc. here...

    layout
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let layout = get_layout();

    let mut memory_x = String::new();
    memory_x.push_str("MEMORY\n{\n");
    memory_x.push_str(&format!("  FLASH (rx)  : ORIGIN = 0x{:08X}, LENGTH = {}K\n", layout.flash_origin, layout.flash_len / 1024));
    memory_x.push_str(&format!("  RAM (rwx)   : ORIGIN = 0x{:08X}, LENGTH = {}K\n", layout.ram_origin, layout.ram_len / 1024));

    if let (Some(itcm_o), Some(itcm_l)) = (layout.itcm_origin, layout.itcm_len) {
        memory_x.push_str(&format!("  ITCM (rwx)  : ORIGIN = 0x{:08X}, LENGTH = {}K\n", itcm_o, itcm_l / 1024));
    }
    memory_x.push_str("}\n\n");

    memory_x.push_str("SECTIONS\n{\n");
    memory_x.push_str("  .text : { *(.text*) } > FLASH\n");
    memory_x.push_str("  .rodata : { *(.rodata*) } > FLASH\n");

    if layout.itcm_origin.is_some() {
        memory_x.push_str("  .itcm_code : { *(.itcm*) } > ITCM AT > FLASH\n");
    }

    memory_x.push_str("  .data : { *(.data*) } > RAM AT > FLASH\n");
    memory_x.push_str("  .bss : { *(.bss*) } > RAM\n");
    memory_x.push_str("  _stack_start = ORIGIN(RAM) + LENGTH(RAM);\n");
    memory_x.push_str("}\n");

    let dest_path = out_dir.join("memory.x");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(memory_x.as_bytes()).unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
}