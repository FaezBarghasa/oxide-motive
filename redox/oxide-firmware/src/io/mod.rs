use embassy_stm32::adc::Adc;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use heapless::Vec;

pub static SENSOR_VALUES: Channel<ThreadModeRawMutex, Vec<u16, 16>, 1> = Channel::new();

#[embassy_executor::task]
pub async fn adc_task(mut adc: Adc<'static, embassy_stm32::peripherals::ADC1>) {
    // TODO: Implement hardware oversampling and DMA reading
}

pub fn setup_pwm(
    timer: embassy_stm32::peripherals::TIM1,
    pin: embassy_stm32::peripherals::PA8,
) -> SimplePwm<'static, embassy_stm32::peripherals::TIM1> {
    let ch1 = PwmPin::new_ch1(pin);
    let mut pwm = SimplePwm::new(timer, Some(ch1), None, None, None, khz(20), Default::default());
    pwm.enable(embassy_stm32::timer::Channel::Ch1);
    pwm
}
