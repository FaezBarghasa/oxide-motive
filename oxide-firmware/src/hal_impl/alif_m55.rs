
#![allow(dead_code)]
use oxide_hal::{EngineTimer, PeakAndHoldPwm};

pub struct AlifEngineTimer;

impl EngineTimer for AlifEngineTimer {
    type Error = core::convert::Infallible;
    fn counter_us(&mut self) -> Result<u32, Self::Error> { Ok(0) }
    fn set_compare_us(&mut self, _channel: u8, _ticks_us: u32) -> Result<(), Self::Error> { Ok(()) }
    fn enable_compare_interrupt(&mut self, _channel: u8) {}
    fn clear_interrupt(&mut self, _channel: u8) {}
    fn ticks_per_us(&self) -> u32 { 1 }
}

pub struct AlifPeakAndHoldPwm;
impl embedded_hal::pwm::ErrorType for AlifPeakAndHoldPwm {
    type Error = core::convert::Infallible;
}
impl embedded_hal::pwm::SetDutyCycle for AlifPeakAndHoldPwm {
    fn max_duty_cycle(&self) -> u16 { 255 }
    fn set_duty_cycle(&mut self, _duty: u16) -> Result<(), Self::Error> { Ok(()) }
}
impl PeakAndHoldPwm for AlifPeakAndHoldPwm {
    fn configure_peak_hold(&mut self, _peak_time_us: u16, _hold_duty_percent: u8) -> Result<(), Self::Error> {
        Ok(())
    }
}
