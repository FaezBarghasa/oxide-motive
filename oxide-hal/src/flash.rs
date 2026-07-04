
pub trait Flash {
    type Error;
    fn erase_bank(&mut self, bank: u8) -> Result<(), Self::Error>;
    fn program_page(&mut self, page_address: u32, data: &[u8]) -> Result<(), Self::Error>;
}
