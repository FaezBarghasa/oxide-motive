use crc32fast::Hasher;
use oxide_hal::flash::ExternalFlash;

const LIMP_MODE_MAP: [f32; 256] = [0.0; 256];

#[repr(C, packed)]
pub struct TableHeader {
    pub magic: u32,       // 0x4F584D54 ("OXMT")
    pub version: u16,
    pub size: u16,
    pub crc32: u32,
    pub is_active: u8,
    pub padding: [u8; 3],
}

pub struct TableManager<F: ExternalFlash> {
    flash: F,
}

#[derive(Debug)]
pub enum StorageError {
    Flash(F::Error),
    InvalidHeader,
    CrcMismatch,
}

impl<F: ExternalFlash> TableManager<F> {
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    pub fn load_active_tables(&mut self, dest: &mut [f32]) -> Result<(), StorageError> {
        const BANK_A_OFFSET: u32 = 0x00010000;
        const BANK_B_OFFSET: u32 = 0x00020000;

        if self.load_bank(BANK_A_OFFSET, dest).is_err() {
            if self.load_bank(BANK_B_OFFSET, dest).is_err() {
                dest.copy_from_slice(&LIMP_MODE_MAP[..dest.len()]);
            }
        }
        Ok(())
    }

    fn load_bank(&mut self, offset: u32, dest: &mut [f32]) -> Result<(), StorageError> {
        let mut header_buf = [0u8; core::mem::size_of::<TableHeader>()];
        self.flash.read(offset, &mut header_buf).map_err(StorageError::Flash)?;

        let header: TableHeader = unsafe { core::mem::transmute(header_buf) };

        if header.magic != 0x4F584D54 {
            return Err(StorageError::InvalidHeader);
        }

        let mut table_buf = [0u8; 1024]; // Max table size
        let table_slice = &mut table_buf[..header.size as usize];
        self.flash.read(offset + core::mem::size_of::<TableHeader>() as u32, table_slice).map_err(StorageError::Flash)?;

        let mut hasher = Hasher::new();
        hasher.update(table_slice);
        let crc = hasher.finalize();

        if crc != header.crc32 {
            return Err(StorageError::CrcMismatch);
        }

        // This is unsafe because we are reinterpreting a byte slice as a float slice.
        // This is only safe if the data is correctly aligned and represents valid floats.
        let float_slice = unsafe {
            core::slice::from_raw_parts(table_slice.as_ptr() as *const f32, table_slice.len() / 4)
        };

        dest.copy_from_slice(&float_slice[..dest.len()]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_hal::flash::ExternalFlash;

    struct MockFlash {
        data: [u8; 0x40000],
    }

    impl MockFlash {
        fn new() -> Self {
            Self { data: [0; 0x40000] }
        }

        fn write_header(&mut self, offset: u32, header: &TableHeader) {
            let header_bytes = unsafe {
                core::slice::from_raw_parts(header as *const _ as *const u8, core::mem::size_of::<TableHeader>())
            };
            self.data[offset as usize..offset as usize + header_bytes.len()].copy_from_slice(header_bytes);
        }

        fn write_table(&mut self, offset: u32, table: &[f32]) {
            let table_bytes = unsafe {
                core::slice::from_raw_parts(table.as_ptr() as *const u8, table.len() * 4)
            };
            self.data[offset as usize..offset as usize + table_bytes.len()].copy_from_slice(table_bytes);
        }
    }

    impl ExternalFlash for MockFlash {
        type Error = ();

        fn init(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            buf.copy_from_slice(&self.data[addr as usize..addr as usize + buf.len()]);
            Ok(())
        }

        fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.data[addr as usize..addr as usize + data.len()].copy_from_slice(data);
            Ok(())
        }

        fn erase_sector(&mut self, addr: u32) -> Result<(), Self::Error> {
            for byte in &mut self.data[addr as usize..addr as usize + 4096] {
                *byte = 0xFF;
            }
            Ok(())
        }

        fn read_device_id(&mut self) -> Result<u32, Self::Error> {
            Ok(0)
        }
    }

    #[test]
    fn test_load_valid_bank_a() {
        let mut flash = MockFlash::new();
        let table: [f32; 16] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];

        let mut hasher = Hasher::new();
        let table_bytes = unsafe {
            core::slice::from_raw_parts(table.as_ptr() as *const u8, table.len() * 4)
        };
        hasher.update(table_bytes);
        let crc = hasher.finalize();

        let header = TableHeader {
            magic: 0x4F584D54,
            version: 1,
            size: (table.len() * 4) as u16,
            crc32: crc,
            is_active: 1,
            padding: [0; 3],
        };

        flash.write_header(0x10000, &header);
        flash.write_table(0x10000 + core::mem::size_of::<TableHeader>() as u32, &table);

        let mut manager = TableManager::new(flash);
        let mut dest = [0.0f32; 16];
        manager.load_active_tables(&mut dest).unwrap();

        assert_eq!(dest, table);
    }

    #[test]
    fn test_fallback_to_bank_b() {
        let mut flash = MockFlash::new();
        let table_a: [f32; 16] = [1.0; 16];
        let table_b: [f32; 16] = [2.0; 16];

        let mut hasher_a = Hasher::new();
        let table_a_bytes = unsafe {
            core::slice::from_raw_parts(table_a.as_ptr() as *const u8, table_a.len() * 4)
        };
        hasher_a.update(table_a_bytes);
        let crc_a = hasher_a.finalize();

        let mut hasher_b = Hasher::new();
        let table_b_bytes = unsafe {
            core::slice::from_raw_parts(table_b.as_ptr() as *const u8, table_b.len() * 4)
        };
        hasher_b.update(table_b_bytes);
        let crc_b = hasher_b.finalize();

        let header_a = TableHeader {
            magic: 0x4F584D54,
            version: 1,
            size: (table_a.len() * 4) as u16,
            crc32: crc_a + 1, // Corrupt CRC
            is_active: 1,
            padding: [0; 3],
        };

        let header_b = TableHeader {
            magic: 0x4F584D54,
            version: 1,
            size: (table_b.len() * 4) as u16,
            crc32: crc_b,
            is_active: 1,
            padding: [0; 3],
        };

        flash.write_header(0x10000, &header_a);
        flash.write_table(0x10000 + core::mem::size_of::<TableHeader>() as u32, &table_a);

        flash.write_header(0x20000, &header_b);
        flash.write_table(0x20000 + core::mem::size_of::<TableHeader>() as u32, &table_b);

        let mut manager = TableManager::new(flash);
        let mut dest = [0.0f32; 16];
        manager.load_active_tables(&mut dest).unwrap();

        assert_eq!(dest, table_b);
    }

    #[test]
    fn test_fallback_to_limp_mode() {
        let mut flash = MockFlash::new();
        let table: [f32; 16] = [1.0; 16];

        let mut hasher = Hasher::new();
        let table_bytes = unsafe {
            core::slice::from_raw_parts(table.as_ptr() as *const u8, table.len() * 4)
        };
        hasher.update(table_bytes);
        let crc = hasher.finalize();

        let header = TableHeader {
            magic: 0x4F584D54,
            version: 1,
            size: (table.len() * 4) as u16,
            crc32: crc + 1, // Corrupt CRC
            is_active: 1,
            padding: [0; 3],
        };

        flash.write_header(0x10000, &header);
        flash.write_table(0x10000 + core::mem::size_of::<TableHeader>() as u32, &table);
        flash.write_header(0x20000, &header);
        flash.write_table(0x20000 + core::mem::size_of::<TableHeader>() as u32, &table);

        let mut manager = TableManager::new(flash);
        let mut dest = [0.0f32; 16];
        manager.load_active_tables(&mut dest).unwrap();

        assert_eq!(dest, [0.0; 16]);
    }
}
