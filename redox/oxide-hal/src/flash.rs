
pub trait ExternalFlash {
    type Error: core::fmt::Debug;
    fn init(&mut self) -> Result<(), Self::Error>;
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error>;
    fn erase_sector(&mut self, addr: u32) -> Result<(), Self::Error>;
    fn read_device_id(&mut self) -> Result<u32, Self::Error>;
}
