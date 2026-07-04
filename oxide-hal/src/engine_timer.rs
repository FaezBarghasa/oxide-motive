
use embedded_hal::digital::ErrorType;

pub trait EngineTimer {
    type Error: ErrorType;
    fn counter_us(&mut self) -> Result<u32, Self::Error>;
    fn set_compare_us(&mut self, channel: u8, ticks_us: u32) -> Result<(), Self::Error>;
    fn enable_compare_interrupt(&mut self, channel: u8);
    fn clear_interrupt(&mut self, channel: u8);
    fn ticks_per_us(&self) -> u32;
}
