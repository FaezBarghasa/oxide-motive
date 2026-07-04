use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub origin: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub flash: MemoryRegion,
    pub ram: MemoryRegion,
    pub itcm: Option<MemoryRegion>,
    pub dtcm: Option<MemoryRegion>,
    pub axi_sram: Option<MemoryRegion>,
    pub ccm: Option<MemoryRegion>,
}

pub fn get_memory_maps() -> HashMap<&'static str, MemoryLayout> {
    let mut maps = HashMap::new();

    maps.insert(
        "stm32h750",
        MemoryLayout {
            flash: MemoryRegion { origin: 0x08000000, size: 128 * 1024 },
            ram: MemoryRegion { origin: 0x24000000, size: 512 * 1024 }, // AXI SRAM
            itcm: None,
            dtcm: Some(MemoryRegion { origin: 0x20000000, size: 128 * 1024 }),
            axi_sram: Some(MemoryRegion { origin: 0x24000000, size: 512 * 1024 }),
            ccm: None,
        },
    );

    maps.insert(
        "stm32f407",
        MemoryLayout {
            flash: MemoryRegion { origin: 0x08000000, size: 1024 * 1024 },
            ram: MemoryRegion { origin: 0x20000000, size: 192 * 1024 },
            itcm: None,
            dtcm: None,
            axi_sram: None,
            ccm: Some(MemoryRegion { origin: 0x10000000, size: 64 * 1024 }),
        },
    );

    maps.insert(
        "nrf9160",
        MemoryLayout {
            flash: MemoryRegion { origin: 0x00000000, size: 1024 * 1024 },
            ram: MemoryRegion { origin: 0x20000000, size: 256 * 1024 },
            itcm: None,
            dtcm: None,
            axi_sram: None,
            ccm: None,
        },
    );

    maps.insert(
        "lpc55s69",
        MemoryLayout {
            flash: MemoryRegion { origin: 0x00000000, size: 640 * 1024 },
            ram: MemoryRegion { origin: 0x20000000, size: 320 * 1024 },
            itcm: None,
            dtcm: None,
            axi_sram: None,
            ccm: None,
        },
    );

    maps.insert(
        "renesas_ra8m1",
        MemoryLayout {
            flash: MemoryRegion { origin: 0x00000000, size: 2 * 1024 * 1024 },
            ram: MemoryRegion { origin: 0x20000000, size: 1024 * 1024 },
            itcm: None,
            dtcm: None,
            axi_sram: None,
            ccm: None,
        },
    );

    maps
}
