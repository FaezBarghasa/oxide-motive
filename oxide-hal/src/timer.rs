
pub trait EngineTimer {
    type Error;
    fn counter(&self) -> u32;
    fn set_compare(&mut self, channel: u8, ticks: u32) -> Result<(), Self::Error>;
    fn enable_compare_interrupt(&mut self, channel: u8);
    fn clear_interrupt(&mut self, channel: u8);
    fn frequency(&self) -> u32; // Returns timer clock in Hz
}
