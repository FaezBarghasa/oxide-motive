#![no_std]

pub mod engine_timer;
pub mod peak_and_hold_pwm;

// Re-export standard embedded-hal traits for universal access
pub use embedded_hal::digital::{InputPin, OutputPin};
pub use embedded_hal::pwm::SetDutyCycle;
pub use embedded_hal::spi::{SpiBus, SpiDevice};
pub use embedded_hal_async::adc::Adc;
pub use embedded_hal_async::can::Can;
pub use embedded_hal_async::i2c::I2c;
pub use embedded_hal_async::spi::SpiBus as SpiBusAsync;
pub use embedded_hal_async::uart::Uart;

// Re-export our custom automotive traits
pub use engine_timer::EngineTimer;
pub use peak_and_hold_pwm::PeakAndHoldPwm;
