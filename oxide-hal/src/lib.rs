#![no_std]

pub mod mock; // Placeholder for mock implementation

#[cfg(feature = "stm32g4")]
pub mod stm32g4;

#[cfg(feature = "nxp_s32k")]
pub mod nxp_s32k;

pub trait HighResAdc {
    type Error;
    fn read_channel(&mut self, channel: u8) -> Result<u16, Self::Error>;
    fn read_all_dma(&mut self, buffer: &mut [u16]) -> Result<(), Self::Error>;
}

pub trait EngineTimer {
    type Error;
    fn set_frequency(&mut self, freq_hz: u32) -> Result<(), Self::Error>;
    fn set_compare(&mut self, channel: u8, ticks: u32) -> Result<(), Self::Error>;
    fn get_counter(&self) -> u32;
    fn enable_interrupt(&mut self) -> Result<(), Self::Error>;
}

pub trait TriggerCapture {
    type Error;
    fn capture_rising_edge(&mut self) -> Result<u32, Self::Error>;
    fn capture_falling_edge(&mut self) -> Result<u32, Self::Error>;
}

// Placeholder for CanFrame, will be defined in oxide-protocol
pub struct CanFrame;

pub trait CanBus {
    type Error;
    fn send_frame(&mut self, id: u32, data: &[u8]) -> Result<(), Self::Error>;
    fn receive_frame(&mut self, buffer: &mut CanFrame) -> Result<(), Self::Error>;
}