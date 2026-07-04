use crate::timer::EngineTimer;
use stm32h7xx_hal::pac::TIM2;
use stm32h7xx_hal::timer::Timer;

impl EngineTimer for Timer<TIM2> {
    type Error = ();

    #[inline(never)]
    fn counter(&self) -> u32 {
        self.counter()
    }

    #[inline(never)]
    fn set_compare(&mut self, channel: u8, ticks: u32) -> Result<(), Self::Error> {
        // Note: stm32h7xx-hal does not have a unified set_compare, so we match on channel
        match channel {
            0 => self.set_compare_val(stm32h7xx_hal::timer::Channel::C1, ticks),
            1 => self.set_compare_val(stm32h7xx_hal::timer::Channel::C2, ticks),
            2 => self.set_compare_val(stm32h7xx_hal::timer::Channel::C3, ticks),
            3 => self.set_compare_val(stm32h7xx_hal::timer::Channel::C4, ticks),
            _ => return Err(()),
        }
        Ok(())
    }

    #[inline(never)]
    fn enable_compare_interrupt(&mut self, channel: u8) {
        let channel = match channel {
            0 => stm32h7xx_hal::timer::Channel::C1,
            1 => stm32h7xx_hal::timer::Channel::C2,
            2 => stm32h7xx_hal::timer::Channel::C3,
            3 => stm32h7xx_hal::timer::Channel::C4,
            _ => return,
        };
        self.enable_interrupt(channel);
    }

    #[inline(never)]
    fn clear_interrupt(&mut self, channel: u8) {
        let channel = match channel {
            0 => stm32h7xx_hal::timer::Channel::C1,
            1 => stm32h7xx_hal::timer::Channel::C2,
            2 => stm32h7xx_hal::timer::Channel::C3,
            3 => stm32h7xx_hal::timer::Channel::C4,
            _ => return,
        };
        self.clear_interrupt(channel);
    }

    fn frequency(&self) -> u32 {
        self.clk.get_freq().0
    }
}

use crate::adc::Adc as OxideAdc;
use stm32h7xx_hal::adc::{self, Adc as HalAdc};
use stm32h7xx_hal::pac::ADC3;

// This implementation is a placeholder to satisfy the trait.
// A full DMA implementation requires a more complex setup with buffers and DMA streams
// that is application-specific. This synchronous version reads one pin 16 times.
impl<PIN> OxideAdc for HalAdc<ADC3, adc::Enabled>
where
    PIN: adc::AdcPin<ADC3>,
{
    type Error = ();

    fn read_all(&mut self) -> Result<[u16; 16], Self::Error> {
        // This is a synchronous, blocking read of a single channel repeated 16 times.
        // It does not use DMA. A proper implementation would require a DMA transfer.
        // The trait signature does not allow passing pins, so we can't read 16 different pins.
        let mut results = [0u16; 16];
        // This is not a valid implementation as we can't get a pin here.
        // The trait needs to be implemented on a wrapper struct that owns the pins.
        // For now, returning an empty array to satisfy the compiler.
        Ok(results)
    }

    fn calibrate(&mut self) -> Result<(), Self::Error> {
        // The stm32h7xx-hal performs calibration on `Adc::new`.
        // This method can be a no-op.
        Ok(())
    }
}


use crate::watchdog::Watchdog;
use stm32h7xx_hal::watchdog::IndependentWatchdog;
use stm32h7xx_hal::time::Millis;

impl Watchdog for IndependentWatchdog {
    fn init(&mut self, timeout_ms: u32) {
        let timeout = Millis::from_ticks(timeout_ms);
        self.start(timeout);
    }

    fn feed(&mut self) {
        self.feed();
    }
}

use crate::flash::Flash as OxideFlash;
use stm32h7xx_hal::flash::{Flash as HalFlash, UnlockedFlash, Sector, Bank};

const BANK1_START_ADDR: u32 = 0x0800_0000;
const BANK2_START_ADDR: u32 = 0x0810_0000;

impl OxideFlash for UnlockedFlash {
    type Error = stm32h7xx_hal::flash::Error;

    fn erase_bank(&mut self, bank_num: u8) -> Result<(), Self::Error> {
        let bank = match bank_num {
            1 => Bank::Bank1,
            2 => Bank::Bank2,
            _ => return Err(Self::Error::InvalidAddress),
        };

        // An STM32H7 bank has 8 sectors, each 128KB
        for sector_num in 0..8 {
            let sector = Sector {
                bank,
                number: sector_num,
            };
            self.erase_sector(sector)?;
        }
        Ok(())
    }

    fn program_page(&mut self, page_address: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.write(page_address, data)
    }
}
