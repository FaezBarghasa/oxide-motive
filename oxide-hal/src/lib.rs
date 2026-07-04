#![no_std]

use embedded_hal_async::{
    can::Can,
    digital::Wait,
    pwm::Pwm,
    serial::{Read, Write},
    spi::SpiBus,
};

pub trait EngineTimer {
    fn set_compare(&mut self, value: u32);
    fn get_counter(&self) -> u32;
}

pub trait Adc<WORD, PIN> {
    type Error;
    fn read(&mut self, pin: &mut PIN) -> nb::Result<WORD, Self::Error>;
}

pub trait ExternalFlash {
    async fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), ()>;
    async fn write(&mut self, address: u32, buffer: &[u8]) -> Result<(), ()>;
    async fn erase_sector(&mut self, sector: u32) -> Result<(), ()>;
}

pub trait Transport: Read + Write {}
impl<T: Read + Write> Transport for T {}
