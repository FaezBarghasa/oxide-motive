
use embedded_hal::pwm::SetDutyCycle;

pub trait PeakAndHoldPwm: SetDutyCycle {
    fn configure_peak_hold(&mut self, peak_time_us: u16, hold_duty_percent: u8) -> Result<(), Self::Error>;
}
