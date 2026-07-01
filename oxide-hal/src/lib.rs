#![no_std]

// Hardware Abstraction Layer traits will go here
pub trait Adc {
    fn read(&mut self, channel: u8) -> u16;
}
