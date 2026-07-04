
use std::collections::HashMap;

pub struct MemoryLayout {
    pub flash_origin: u32,
    pub flash_len: u32,
    pub ram_origin: u32,
    pub ram_len: u32,
    pub itcm_origin: Option<u32>,
    pub itcm_len: Option<u32>,
    pub dtcm_origin: Option<u32>,
    pub dtcm_len: Option<u32>,
    pub axi_origin: Option<u32>,
    pub axi_len: Option<u32>,
    pub ccm_origin: Option<u32>,
    pub ccm_len: Option<u32>,
}

pub fn get_memory_maps() -> HashMap<&'static str, MemoryLayout> {
    let mut maps = HashMap::new();
    maps.insert("stm32h750", MemoryLayout {
        flash_origin: 0x08000000,
        flash_len: 128 * 1024,
        ram_origin: 0x24000000, // AXI SRAM
        ram_len: 512 * 1024,
        itcm_origin: None,
        itcm_len: None,
        dtcm_origin: Some(0x20000000),
        dtcm_len: Some(128 * 1024),
        axi_origin: Some(0x24000000),
        axi_len: Some(512 * 1024),
        ccm_origin: None,
        ccm_len: None,
    });
    maps.insert("stm32f407", MemoryLayout {
        flash_origin: 0x08000000,
        flash_len: 1024 * 1024,
        ram_origin: 0x20000000,
        ram_len: 192 * 1024,
        itcm_origin: None,
        itcm_len: None,
        dtcm_origin: None,
        dtcm_len: None,
        axi_origin: None,
        axi_len: None,
        ccm_origin: Some(0x10000000),
        ccm_len: Some(64 * 1024),
    });
    maps.insert("nrf9160", MemoryLayout {
        flash_origin: 0x00000000,
        flash_len: 1024 * 1024,
        ram_origin: 0x20000000,
        ram_len: 256 * 1024,
        itcm_origin: None,
        itcm_len: None,
        dtcm_origin: None,
        dtcm_len: None,
        axi_origin: None,
        axi_len: None,
        ccm_origin: None,
        ccm_len: None,
    });
    maps.insert("lpc55s69", MemoryLayout {
        flash_origin: 0x00000000,
        flash_len: 640 * 1024,
        ram_origin: 0x20000000,
        ram_len: 320 * 1024,
        itcm_origin: None,
        itcm_len: None,
        dtcm_origin: None,
        dtcm_len: None,
        axi_origin: None,
        axi_len: None,
        ccm_origin: None,
        ccm_len: None,
    });
    maps.insert("renesas_ra8m1", MemoryLayout {
        flash_origin: 0x00000000,
        flash_len: 2 * 1024 * 1024,
        ram_origin: 0x20000000,
        ram_len: 1024 * 1024,
        itcm_origin: None,
        itcm_len: None,
        dtcm_origin: None,
        dtcm_len: None,
        axi_origin: None,
        axi_len: None,
        ccm_origin: None,
        ccm_len: None,
    });
    maps
}
