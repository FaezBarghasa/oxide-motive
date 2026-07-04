
#![allow(dead_code)]
use oxide_hal::{EngineTimer, PeakAndHoldPwm};

#[cfg(feature = "stm32h7")]
use stm32h7xx_hal::{pac, prelude::*, timer::{Timer, Event}};

#[cfg(feature = "stm32h7")]
pub struct Stm32EngineTimer {
    timer: Timer<pac::TIM2>,
}

#[cfg(feature = "stm32h7")]
impl Stm32EngineTimer {
    pub fn new(tim: pac::TIM2, rcc: &mut stm32h7xx_hal::rcc::CoreClocks) -> Self {
        let mut timer = tim.timer(1.MHz(), rcc);
        timer.listen(Event::Update);
        Stm32EngineTimer { timer }
    }
}

#[cfg(feature = "stm32h7")]
impl EngineTimer for Stm32EngineTimer {
    type Error = stm32h7xx_hal::timer::Error;

    fn counter_us(&mut self) -> Result<u32, Self::Error> {
        Ok(self.timer.counter().to_micros())
    }

    fn set_compare_us(&mut self, channel: u8, ticks_us: u32) -> Result<(), Self::Error> {
        let ch = stm32h7xx_hal::timer::Channel::from(channel);
        self.timer.set_compare(ch, stm32h7xx_hal::time::duration::Microseconds(ticks_us));
        Ok(())
    }

    fn enable_compare_interrupt(&mut self, channel: u8) {
        let ch = stm32h7xx_hal::timer::Channel::from(channel);
        self.timer.enable_interrupt(stm32h7xx_hal::timer::Event::Compare(ch));
    }

    fn clear_interrupt(&mut self, channel: u8) {
        let ch = stm32h7xx_hal::timer::Channel::from(channel);
        self.timer.clear_interrupt(stm32h7xx_hal::timer::Event::Compare(ch));
    }

    fn ticks_per_us(&self) -> u32 {
        1
    }
}

// Mock implementation for other STM32 families for compilation
#[cfg(not(feature = "stm32h7"))]
pub struct Stm32EngineTimer;

#[cfg(not(feature = "stm32h7"))]
impl EngineTimer for Stm32EngineTimer {
    type Error = core::convert::Infallible;
    fn counter_us(&mut self) -> Result<u32, Self::Error> { Ok(0) }
    fn set_compare_us(&mut self, _channel: u8, _ticks_us: u32) -> Result<(), Self::Error> { Ok(()) }
    fn enable_compare_interrupt(&mut self, _channel: u8) {}
    fn clear_interrupt(&mut self, _channel: u8) {}
    fn ticks_per_us(&self) -> u32 { 1 }
}

pub struct Stm32PeakAndHoldPwm;
impl embedded_hal::pwm::ErrorType for Stm32PeakAndHoldPwm {
    type Error = core::convert::Infallible;
}
impl embedded_hal::pwm::SetDutyCycle for Stm32PeakAndHoldPwm {
    fn max_duty_cycle(&self) -> u16 { 255 }
    fn set_duty_cycle(&mut self, _duty: u16) -> Result<(), Self::Error> { Ok(()) }
}
impl PeakAndHoldPwm for Stm32PeakAndHoldPwm {
    fn configure_peak_hold(&mut self, _peak_time_us: u16, _hold_duty_percent: u8) -> Result<(), Self::Error> {
        Ok(())
    }
}
