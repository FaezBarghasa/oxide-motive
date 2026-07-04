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

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::adc::{Mock as AdcMock, Transaction as AdcTransaction};
    use embedded_hal_mock::timer::{Mock as TimerMock, Transaction as TimerTransaction};
    use embedded_hal_mock::pwm::{Mock as PwmMock, Transaction as PwmTransaction};

    struct MockDriver<T: EngineTimer, P: PeakAndHoldPwm> {
        timer: T,
        pwm: P,
    }

    #[test]
    fn test_generic_driver_compiles() {
        let timer_expectations = [
            TimerTransaction::get_counter_us(100),
            TimerTransaction::set_compare_us(0, 500),
            TimerTransaction::enable_compare_interrupt(0),
            TimerTransaction::clear_interrupt(0),
        ];
        let pwm_expectations = [
            PwmTransaction::set_duty_cycle(128),
            PwmTransaction::configure_peak_hold(100, 50),
        ];

        let mut timer = TimerMock::new(&timer_expectations);
        let mut pwm = PwmMock::new(&pwm_expectations);

        let mut driver = MockDriver {
            timer,
            pwm,
        };

        assert_eq!(driver.timer.counter_us().unwrap(), 100);
        driver.timer.set_compare_us(0, 500).unwrap();
        driver.timer.enable_compare_interrupt(0);
        driver.timer.clear_interrupt(0);

        driver.pwm.set_duty_cycle(128).unwrap();
        driver.pwm.configure_peak_hold(100, 50).unwrap();

        driver.timer.done();
        driver.pwm.done();
    }
}
