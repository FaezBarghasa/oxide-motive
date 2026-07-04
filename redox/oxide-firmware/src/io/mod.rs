use embassy_stm32::adc::Adc;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use heapless::Vec;
use embassy_time::Timer;

pub static SENSOR_VALUES: Channel<ThreadModeRawMutex, Vec<u16, 16>, 1> = Channel::new();

#[embassy_executor::task]
pub async fn adc_task(mut adc: Adc<'static, embassy_stm32::peripherals::ADC1>) {
    let mut vrefint = adc.enable_vrefint();
    loop {
        let mut samples = Vec::new();
        for _ in 0..16 {
            let sample = adc.read(&mut vrefint);
            let _ = samples.push(sample);
        }
        
        let _ = SENSOR_VALUES.send(samples).await;
        Timer::after_millis(10).await;
    }
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
