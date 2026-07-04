use embedded_hal::pwm::SetDutyCycle;

/// A trait for PWM peripherals that support a "peak and hold" mode.
///
/// This is commonly used for driving low-impedance fuel injectors, where a high
/// initial current ("peak") is needed to open the injector quickly, followed by a
/// lower current ("hold") to keep it open, reducing heat and power consumption.
pub trait PeakAndHoldPwm: SetDutyCycle {
    /// Configures the peak and hold parameters for the PWM channel.
    ///
    /// # Arguments
    /// * `peak_time_us` - The duration of the initial high-current "peak" phase in microseconds.
    /// * `hold_duty_percent` - The duty cycle (as a percentage) for the subsequent "hold" phase.
    fn configure_peak_hold(
        &mut self,
        peak_time_us: u16,
        hold_duty_percent: u8,
    ) -> Result<(), Self::Error>;
}
