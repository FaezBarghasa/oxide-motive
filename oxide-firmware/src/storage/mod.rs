use crc::{Crc, CRC_32_ISO_HDLC};
use oxide_hal::ExternalFlash;
use postcard::from_bytes;
use crate::app::Table3D;

#[repr(C, packed)]
pub struct TableHeader {
    magic: u32,
    version: u32,
    size: u32,
    crc32: u32,
    is_active: bool,
}

pub struct TableManager<'a, F: ExternalFlash> {
    flash: &'a mut F,
}

impl<'a, F: ExternalFlash> TableManager<'a, F> {
    pub fn new(flash: &'a mut F) -> Self {
        Self { flash }
    }

    pub async fn load_active_tables(&mut self) -> (Table3D<f32, 16, 16>, Table3D<f32, 16, 16>) {
        if let Ok(tables) = self.load_bank(0).await {
            tables
        } else if let Ok(tables) = self.load_bank(1).await {
            tables
        } else {
            (
                Table3D::new_from_data([[0.0; 16]; 16]),
                Table3D::new_from_data([[0.0; 16]; 16]),
            )
        }
    }

    async fn load_bank(&mut self, bank: u32) -> Result<(Table3D<f32, 16, 16>, Table3D<f32, 16, 16>), ()> {
        let mut header_buf = [0u8; core::mem::size_of::<TableHeader>()];
        self.flash.read(bank * 0x100000, &mut header_buf).await?;
        let header: TableHeader = from_bytes(&header_buf).map_err(|_| ())?;

        if header.magic != 0xDEADBEEF || !header.is_active {
            return Err(());
        }

        let mut table_buf = [0u8; 1024];
        self.flash.read(bank * 0x100000 + core::mem::size_of::<TableHeader>() as u32, &mut table_buf).await?;

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        if crc.checksum(&table_buf[..header.size as usize]) != header.crc32 {
            return Err(());
        }

        let tables: (Table3D<f32, 16, 16>, Table3D<f32, 16, 16>) = from_bytes(&table_buf).map_err(|_| ())?;
        Ok(tables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Table3D;
    use postcard::to_vec;
    use heapless::Vec;

    struct MockFlash {
        data: Vec<u8, 8192>,
    }

    impl ExternalFlash for MockFlash {
        async fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), ()> {
            let address = address as usize;
            if address + buffer.len() > self.data.len() {
                return Err(());
            }
            buffer.copy_from_slice(&self.data[address..address + buffer.len()]);
            Ok(())
        }

        async fn write(&mut self, address: u32, buffer: &[u8]) -> Result<(), ()> {
            let address = address as usize;
            if address + buffer.len() > self.data.len() {
                return Err(());
            }
            self.data[address..address + buffer.len()].copy_from_slice(buffer);
            Ok(())
        }

        async fn erase_sector(&mut self, sector: u32) -> Result<(), ()> {
            let start = sector as usize * 4096;
            if start >= self.data.len() {
                return Err(());
            }
            let end = (start + 4096).min(self.data.len());
            for i in start..end {
                self.data[i] = 0xFF;
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_table_manager() {
        let mut flash_data = Vec::new();
        flash_data.resize_default(8192).unwrap();
        let mut flash = MockFlash { data: flash_data };

        let tables = (
            Table3D::new_from_data([[1.0; 16]; 16]),
            Table3D::new_from_data([[2.0; 16]; 16]),
        );
        let table_data: Vec<u8, 1024> = to_vec(&tables).unwrap();

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let crc32 = crc.checksum(&table_data);

        let header = TableHeader {
            magic: 0xDEADBEEF,
            version: 1,
            size: table_data.len() as u32,
            crc32,
            is_active: true,
        };
        let header_data: Vec<u8, 32> = to_vec(&header).unwrap();

        flash.write(0, &header_data).await.unwrap();
        flash.write(header_data.len() as u32, &table_data).await.unwrap();

        let mut manager = TableManager::new(&mut flash);
        let loaded_tables = manager.load_active_tables().await;

        assert_eq!(loaded_tables.0.data[0][0], 1.0);
        assert_eq!(loaded_tables.1.data[0][0], 2.0);
    }
}
