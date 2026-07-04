use stm32h7xx_hal::qspi::{self, Qspi};
use stm32h7xx_hal::gpio::{self, GpioExt};
use stm32h7xx_hal::rcc::{self, RccExt};
use stm32h7xx_hal::pac;
use oxide_hal::ExternalFlash;

pub struct QspiFlash {
    qspi: Qspi<pac::QUADSPI>,
}

impl QspiFlash {
    pub fn new(
        qspi: pac::QUADSPI,
        gpiod: pac::GPIOD,
        gpioe: pac::GPIOE,
        gpiof: pac::GPIOF,
        rcc: &mut rcc::CoreClocks,
    ) -> Self {
        let gpiod = gpiod.split(rcc.ahb4);
        let gpioe = gpioe.split(rcc.ahb4);
        let gpiof = gpiof.split(rcc.ahb4);

        let sck = gpiod.pd3.into_alternate();
        let cs = gpioe.pe11.into_alternate();
        let io0 = gpiof.pf8.into_alternate();
        let io1 = gpiof.pf9.into_alternate();
        let io2 = gpioe.pe2.into_alternate();
        let io3 = gpiof.pf7.into_alternate();

        let qspi_config = qspi::Config::new(50.MHz()).with_dma();
        let qspi = Qspi::new(qspi, (sck, cs, io0, io1, io2, io3), qspi_config, rcc);

        Self { qspi }
    }
}

impl ExternalFlash for QspiFlash {
    async fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), ()> {
        self.qspi.read(0x6B, address, buffer).await
    }

    async fn write(&mut self, address: u32, buffer: &[u8]) -> Result<(), ()> {
        self.qspi.write_enable().await?;
        self.qspi.write(0x32, address, buffer).await
    }

    async fn erase_sector(&mut self, sector: u32) -> Result<(), ()> {
        self.qspi.write_enable().await?;
        self.qspi.erase(0x20, sector * 4096).await
    }
}
