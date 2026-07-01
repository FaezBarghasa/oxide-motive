#![no_std]

pub mod timer;
pub mod adc;
pub mod can;
pub mod pwm;
pub mod watchdog;
pub mod flash;

// Example: EngineTimer trait (critical for ignition scheduling)
pub trait EngineTimer {
    type Error;
    fn counter(&self) -> u32;
    fn set_compare(&mut self, channel: u8, ticks: u32) -> Result<(), Self::Error>;
    fn enable_compare_interrupt(&mut self, channel: u8);
    fn clear_interrupt(&mut self, channel: u8);
    fn frequency(&self) -> u32; // Returns timer clock in Hz
}

// Example: Adc trait
pub trait Adc {
    type Error;
    fn read_all(&mut self) -> Result<heapless::Vec<u16, 16>, Self::Error>;
    fn calibrate(&mut self) -> Result<(), Self::Error>;
}

// Example: Watchdog trait
pub trait Watchdog {
    fn init(&mut self, timeout_ms: u32);
    fn feed(&mut self);
}

// Example: Flash trait for A/B partition management
pub trait Flash {
    type Error;
    fn erase_bank(&mut self, bank: u8) -> Result<(), Self::Error>;
    fn program_page(&mut self, page_address: u32, data: &[u8]) -> Result<(), Self::Error>;
}
