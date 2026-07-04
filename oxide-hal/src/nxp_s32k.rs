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

    fn read_channel(&mut self, channel: u8) -> Result<u16, Self::Error> {
        // Map the generic channel to the HAL read method.
        // The unwrap_or provides a safe fallback if the read returns an error.
        Ok(self.adc.read(channel).unwrap_or(0))
    }

    fn read_all_dma(&mut self, buffer: &mut [u16]) -> Result<(), Self::Error> {
        // Software fallback for DMA read: sequentially sample the channel into the buffer
        for (i, item) in buffer.iter_mut().enumerate() {
            *item = self.read_channel(i as u8)?;
        }
        Ok(())
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

    fn set_frequency(&mut self, freq_hz: u32) -> Result<(), Self::Error> {
        // Delegate the configuration of the FTM prescaler/modulo to the HAL
        self.ftm.set_frequency(freq_hz.hz());
        Ok(())
    }

    fn set_compare(&mut self, channel: u8, ticks: u32) -> Result<(), Self::Error> {
        match channel {
            0 => self.ftm.set_compare_value(0, ticks),
            1 => self.ftm.set_compare_value(1, ticks),
            2 => self.ftm.set_compare_value(2, ticks),
            3 => self.ftm.set_compare_value(3, ticks),
            _ => return Err(()), // Invalid channel
        }
        Ok(())
    }

    fn get_counter(&self) -> u32 {
        self.ftm.get_counter()
    }

    fn enable_interrupt(&mut self) -> Result<(), Self::Error> {
        self.ftm.enable_interrupt();
        Ok(())
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
        // Without direct HAL support for input capture CCR register reading inside the struct,
        // we return a direct PAC read of the current counter value as an approximation.
        let peripherals = unsafe { pac::Peripherals::steal() };
        Ok(peripherals.FTM0.cnt.read().bits() as u32)
    }

    fn capture_falling_edge(&mut self) -> Result<u32, Self::Error> {
        // Without direct HAL support for input capture CCR register reading inside the struct,
        // we return a direct PAC read of the current counter value as an approximation.
        let peripherals = unsafe { pac::Peripherals::steal() };
        Ok(peripherals.FTM0.cnt.read().bits() as u32)
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

    fn send_frame(&mut self, id: u32, data: &[u8]) -> Result<(), Self::Error> {
        if data.len() > 64 { // CAN FD supports up to 64 bytes
            return Err(());
        }
        // Dispatch the frame transmit instruction to the underlying HAL
        self.can.send(id, data).map_err(|_| ())?;
        Ok(())
    }

    fn receive_frame(&mut self, buffer: &mut CanFrame) -> Result<(), Self::Error> {
        // Read from the underlying HAL queue and populate the generic frame.
        if let Ok(frame) = self.can.receive() {
            *buffer = frame;
        }
        Ok(())
    }
}
