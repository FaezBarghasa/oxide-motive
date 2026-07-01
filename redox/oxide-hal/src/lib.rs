#![no_std]

use embedded_hal_async::{
    adc::Channel,
    can::Can,
    pwm::Pwm,
    spi::SpiDevice,
};
use embedded_io_async::{Read, Write};
use futures::future::Future;

pub trait Timer {
    fn now(&self) -> u64;
    fn sleep(&self, duration_us: u64) -> impl Future<Output = ()>;
}

pub trait Adc<ADC, P>: Channel<ADC, Word = u16> {}

pub trait Uart: Read + Write {}

pub trait Gpio {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

pub trait EngineTimer {
    fn set_compare(&mut self, channel: u8, ticks: u32);
    fn get_counter(&self) -> u32;
}
