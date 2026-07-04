//! Secure storage manager for loading calibration tables from external flash.

use crate::hal::ExternalFlash;
use crc32fast::Hasher;

const TABLE_HEADER_MAGIC: u32 = 0x4F584D54; // "OXMT"
const CALIBRATION_BANK_A_ADDR: u32 = 0x0001_0000;
const CALIBRATION_BANK_B_ADDR: u32 = 0x0002_0000;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TableHeader {
    pub magic: u32,
    pub version: u16,
    pub size: u16,
    pub crc32: u32,
    pub is_active: u8,
    pub padding: [u8; 3],
}

impl TableHeader {
    pub fn is_valid(&self, payload: &[u8]) -> bool {
        if self.magic != TABLE_HEADER_MAGIC {
            return false;
        }
        let mut hasher = Hasher::new();
        hasher.update(payload);
        let calculated_crc = hasher.finalize();
        self.crc32 == calculated_crc
    }
}

#[derive(Debug)]
pub enum StorageError {
    FlashError,
    InvalidBankA,
    InvalidBankB,
    BothBanksCorrupt,
}

pub struct TableManager<F: ExternalFlash> {
    flash: F,
}

impl<F: ExternalFlash> TableManager<F> {
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    pub fn load_active_tables(&mut self, dest: &mut [u8]) -> Result<(), StorageError> {
        match self.load_bank(CALIBRATION_BANK_A_ADDR, dest) {
            Ok(_) => Ok(()),
            Err(StorageError::InvalidBankA) => {
                // Bank A is corrupt, try Bank B
                self.load_bank(CALIBRATION_BANK_B_ADDR, dest)
                    .map_err(|_| StorageError::BothBanksCorrupt)
            }
            Err(e) => Err(e),
        }
    }

    fn load_bank(&mut self, base_addr: u32, dest: &mut [u8]) -> Result<(), StorageError> {
        let mut header_buf = [0u8; core::mem::size_of::<TableHeader>()];
        self.flash.read(base_addr, &mut header_buf).map_err(|_| StorageError::FlashError)?;

        let header: TableHeader = unsafe { core::ptr::read(header_buf.as_ptr() as *const _) };

        if header.size as usize > dest.len() {
            return Err(StorageError::InvalidBankA); // Or some other error
        }

        let payload_slice = &mut dest[..header.size as usize];
        self.flash.read(base_addr + header_buf.len() as u32, payload_slice).map_err(|_| StorageError::FlashError)?;

        if header.is_valid(payload_slice) {
            Ok(())
        } else {
            Err(StorageError::InvalidBankA) // Use a generic error for invalid bank
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::ExternalFlash;
    use heapless::Vec;

    #[derive(Debug)]
    struct MockFlash {
        memory: Vec<u8, 65536>,
        fail_read: bool,
    }

    impl MockFlash {
        fn new() -> Self {
            Self {
                memory: Vec::new(),
                fail_read: false,
            }
        }

        fn write_mem(&mut self, addr: u32, data: &[u8]) {
            let addr = addr as usize;
            if self.memory.len() < addr + data.len() {
                self.memory.resize(addr + data.len(), 0).unwrap();
            }
            self.memory[addr..addr + data.len()].copy_from_slice(data);
        }
    }

    impl ExternalFlash for MockFlash {
        type Error = ();

        fn init(&mut self) -> Result<(), Self::Error> { Ok(()) }

        fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            if self.fail_read {
                return Err(());
            }
            let addr = addr as usize;
            if addr + buf.len() > self.memory.len() {
                return Err(());
            }
            buf.copy_from_slice(&self.memory[addr..addr + buf.len()]);
            Ok(())
        }

        fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.write_mem(addr, data);
            Ok(())
        }

        fn erase_sector(&mut self, _addr: u32) -> Result<(), Self::Error> { Ok(()) }
        fn read_device_id(&mut self) -> Result<u32, Self::Error> { Ok(0) }
    }

    fn create_test_bank(payload: &[u8], version: u16, is_active: bool) -> (TableHeader, Vec<u8, 1024>) {
        let mut hasher = Hasher::new();
        hasher.update(payload);
        let crc32 = hasher.finalize();

        let header = TableHeader {
            magic: TABLE_HEADER_MAGIC,
            version,
            size: payload.len() as u16,
            crc32,
            is_active: if is_active { 1 } else { 0 },
            padding: [0; 3],
        };

        let mut bank_data: Vec<u8, 1024> = Vec::new();
        let header_bytes: [u8; core::mem::size_of::<TableHeader>()] = unsafe { core::mem::transmute(header) };
        bank_data.extend_from_slice(&header_bytes).unwrap();
        bank_data.extend_from_slice(payload).unwrap();

        (header, bank_data)
    }

    #[test]
    fn test_load_bank_a_success() {
        let payload = b"This is a valid payload for bank A";
        let (_header, bank_data) = create_test_bank(payload, 1, true);

        let mut flash = MockFlash::new();
        flash.write_mem(CALIBRATION_BANK_A_ADDR, &bank_data);

        let mut manager = TableManager::new(flash);
        let mut dest = [0u8; 1024];

        assert!(manager.load_active_tables(&mut dest).is_ok());
        assert_eq!(&dest[..payload.len()], payload);
    }

    #[test]
    fn test_fallback_to_bank_b() {
        // Bank A is corrupt
        let payload_a = b"This is a corrupt payload for bank A";
        let (mut header_a, bank_data_a) = create_test_bank(payload_a, 1, true);
        header_a.crc32 = 0; // Corrupt the CRC
        let header_a_bytes: [u8; core::mem::size_of::<TableHeader>()] = unsafe { core::mem::transmute(header_a) };

        let mut flash = MockFlash::new();
        flash.write_mem(CALIBRATION_BANK_A_ADDR, &header_a_bytes);
        flash.write_mem(CALIBRATION_BANK_A_ADDR + header_a_bytes.len() as u32, payload_a);

        // Bank B is valid
        let payload_b = b"This is a valid payload for bank B";
        let (_header_b, bank_data_b) = create_test_bank(payload_b, 1, true);
        flash.write_mem(CALIBRATION_BANK_B_ADDR, &bank_data_b);

        let mut manager = TableManager::new(flash);
        let mut dest = [0u8; 1024];

        assert!(manager.load_active_tables(&mut dest).is_ok());
        assert_eq!(&dest[..payload_b.len()], payload_b);
    }

    #[test]
    fn test_both_banks_corrupt() {
        // Bank A is corrupt
        let payload_a = b"This is a corrupt payload for bank A";
        let (mut header_a, bank_data_a) = create_test_bank(payload_a, 1, true);
        header_a.crc32 = 0; // Corrupt the CRC
        let header_a_bytes: [u8; core::mem::size_of::<TableHeader>()] = unsafe { core::mem::transmute(header_a) };

        let mut flash = MockFlash::new();
        flash.write_mem(CALIBRATION_BANK_A_ADDR, &header_a_bytes);
        flash.write_mem(CALIBRATION_BANK_A_ADDR + header_a_bytes.len() as u32, payload_a);

        // Bank B is also corrupt
        let payload_b = b"This is a corrupt payload for bank B";
        let (mut header_b, bank_data_b) = create_test_bank(payload_b, 1, true);
        header_b.magic = 0; // Corrupt the magic
        let header_b_bytes: [u8; core::mem::size_of::<TableHeader>()] = unsafe { core::mem::transmute(header_b) };
        flash.write_mem(CALIBRATION_BANK_B_ADDR, &header_b_bytes);
        flash.write_mem(CALIBRATION_BANK_B_ADDR + header_b_bytes.len() as u32, payload_b);

        let mut manager = TableManager::new(flash);
        let mut dest = [0u8; 1024];

        match manager.load_active_tables(&mut dest) {
            Err(StorageError::BothBanksCorrupt) => assert!(true),
            _ => assert!(false, "Expected BothBanksCorrupt error"),
        }
    }
}
