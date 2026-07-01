//! NXP S32K HAL implementation for oxide-hal traits

use crate::{CanBus, CanFrame, EngineTimer, HighResAdc, TriggerCapture};
use nxp_s32k1xx_hal::{
    adc::{Adc, AdcConfig},
    can::Can,
    ftm::{Ftm, FtmConfig},
    gpio::{GpioExt, Port},
    pac::{self, ADC0, CAN0, FTM0},
    prelude::*,
};

pub struct NxpS32kAdc {
    adc: Adc<ADC0>,
}

impl NxpS32kAdc {
    pub fn new(adc: ADC0) -> Self {
        let mut peripherals = unsafe { pac::Peripherals::steal() };
        let mut rcc = peripherals.RCC.constrain();
        let mut pcc = peripherals.PCC.constrain();

        let adc_config = AdcConfig::default();
        let adc = Adc::new(adc, &mut pcc.adc0, adc_config);
        Self { adc }
    }
}

impl HighResAdc for NxpS32kAdc {
    type Error = (); // Placeholder for actual error type

    fn read_channel(&mut self, _channel: u8) -> Result<u16, Self::Error> {
        todo!("Implement ADC read for NXP S32K");
    }

    fn read_all_dma(&mut self, _buffer: &mut [u16]) -> Result<(), Self::Error> {
        todo!("Implement DMA ADC read for NXP S32K");
    }
}

pub struct NxpS32kEngineTimer {
    ftm: Ftm<FTM0>,
}

impl NxpS32kEngineTimer {
    pub fn new(ftm: FTM0) -> Self {
        let mut peripherals = unsafe { pac::Peripherals::steal() };
        let mut rcc = peripherals.RCC.constrain();
        let mut pcc = peripherals.PCC.constrain();

        let ftm_config = FtmConfig::default();
        let ftm = Ftm::new(ftm, &mut pcc.ftm0, ftm_config);
        Self { ftm }
    }
}

impl EngineTimer for NxpS32kEngineTimer {
    type Error = (); // Placeholder for actual error type

    fn set_frequency(&mut self, _freq_hz: u32) -> Result<(), Self::Error> {
        todo!("Implement set_frequency for NXP S32K FTM");
    }

    fn set_compare(&mut self, _channel: u8, _ticks: u32) -> Result<(), Self::Error> {
        todo!("Implement set_compare for NXP S32K FTM");
    }

    fn get_counter(&self) -> u32 {
        todo!("Implement get_counter for NXP S32K FTM");
    }

    fn enable_interrupt(&mut self) -> Result<(), Self::Error> {
        todo!("Implement enable_interrupt for NXP S32K FTM");
    }
}

pub struct NxpS32kTriggerCapture {
    // Placeholder for FTM input capture
}

impl NxpS32kTriggerCapture {
    pub fn new() -> Self {
        Self {}
    }
}

impl TriggerCapture for NxpS32kTriggerCapture {
    type Error = (); // Placeholder for actual error type

    fn capture_rising_edge(&mut self) -> Result<u32, Self::Error> {
        todo!("Implement rising edge capture for NXP S32K");
    }

    fn capture_falling_edge(&mut self) -> Result<u32, Self::Error> {
        todo!("Implement falling edge capture for NXP S32K");
    }
}

pub struct NxpS32kCanBus {
    can: Can<CAN0>,
}

impl NxpS32kCanBus {
    pub fn new(can: CAN0) -> Self {
        let mut peripherals = unsafe { pac::Peripherals::steal() };
        let mut rcc = peripherals.RCC.constrain();
        let mut pcc = peripherals.PCC.constrain();

        let can = Can::new(can, &mut pcc.can0);
        Self { can }
    }
}

impl CanBus for NxpS32kCanBus {
    type Error = (); // Placeholder for actual error type

    fn send_frame(&mut self, _id: u32, _data: &[u8]) -> Result<(), Self::Error> {
        todo!("Implement CAN send for NXP S32K");
    }

    fn receive_frame(&mut self, _buffer: &mut CanFrame) -> Result<(), Self::Error> {
        todo!("Implement CAN receive for NXP S32K");
    }
}
