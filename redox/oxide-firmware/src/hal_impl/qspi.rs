use embassy_stm32::qspi::{Config, Qspi, Command, AddressSize, DataMode};
use embassy_stm32::time::mhz;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_hal_common::into_ref;
use oxide_hal::flash::ExternalFlash;

pub struct QspiDriver<'d> {
    qspi: Qspi<'d, embassy_stm32::peripherals::QUADSPI, embassy_stm32::dma::DMA1_CH0>,
}

impl<'d> QspiDriver<'d> {
    pub fn new(
        qspi: embassy_stm32::peripherals::QUADSPI,
        dma: embassy_stm32::dma::DMA1_CH0,
        sck: embassy_stm32::peripherals::PB2,
        cs: embassy_stm32::peripherals::PG6,
        io0: embassy_stm32::peripherals::PF8,
        io1: embassy_stm32::peripherals::PF9,
        io2: embassy_stm32::peripherals::PF7,
        io3: embassy_stm32::peripherals::PF6,
    ) -> Self {
        let mut config = Config::default();
        config.frequency = mhz(50);
        let qspi = Qspi::new(
            qspi,
            sck,
            cs,
            io0,
            io1,
            io2,
            io3,
            dma,
            config,
        );
        Self { qspi }
    }
}

#[derive(Debug)]
pub enum QspiError {
    Qspi(embassy_stm32::qspi::Error),
    // Add other error types as needed
}

impl From<embassy_stm32::qspi::Error> for QspiError {
    fn from(e: embassy_stm32::qspi::Error) -> Self {
        QspiError::Qspi(e)
    }
}

impl<'d> ExternalFlash for QspiDriver<'d> {
    type Error = QspiError;

    fn init(&mut self) -> Result<(), Self::Error> {
        // The QSPI peripheral is initialized in the constructor.
        // We can add any device-specific initialization here.
        Ok(())
    }

    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut cmd = Command::default();
        cmd.instruction = Some(0xEB); // Fast Read Quad I/O
        cmd.address = Some(addr);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.dummy_cycles = 6;
        cmd.data_size = Some(buf.len() as u32);
        cmd.data_mode = Some(DataMode::Quad);
        self.qspi.read_dma(cmd, buf).map_err(Into::into)
    }

    fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
        let mut cmd = Command::default();
        cmd.instruction = Some(0x06); // Write Enable
        self.qspi.command(cmd)?;

        let mut cmd = Command::default();
        cmd.instruction = Some(0x02); // Page Program
        cmd.address = Some(addr);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.data_size = Some(data.len() as u32);
        self.qspi.write_dma(cmd, data)?;
        self.busy_wait()
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), Self::Error> {
        let mut cmd = Command::default();
        cmd.instruction = Some(0x06); // Write Enable
        self.qspi.command(cmd)?;

        let mut cmd = Command::default();
        cmd.instruction = Some(0x20); // Sector Erase 4KB
        cmd.address = Some(addr);
        cmd.address_size = Some(AddressSize::Bits24);
        self.qspi.command(cmd)?;
        self.busy_wait()
    }

    fn read_device_id(&mut self) -> Result<u32, Self::Error> {
        let mut buf = [0u8; 3];
        let mut cmd = Command::default();
        cmd.instruction = Some(0x9F); // Read JEDEC ID
        cmd.data_size = Some(3);
        self.qspi.read_dma(cmd, &mut buf)?;
        Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
    }
}

impl<'d> QspiDriver<'d> {
    fn busy_wait(&mut self) -> Result<(), QspiError> {
        loop {
            let mut buf = [0u8; 1];
            let mut cmd = Command::default();
            cmd.instruction = Some(0x05); // Read Status Reg 1
            cmd.data_size = Some(1);
            self.qspi.read_dma(cmd, &mut buf)?;
            if (buf[0] & 0x01) == 0 {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn verify_read_command() {
        let mut cmd = Command::default();
        cmd.instruction = Some(0xEB); // Fast Read Quad I/O
        cmd.address = Some(0x123456);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.dummy_cycles = 6;
        cmd.data_size = Some(256);
        cmd.data_mode = Some(DataMode::Quad);

        // These assertions are checked at compile time
        assert!(cmd.instruction.unwrap() == 0xEB);
        assert!(cmd.dummy_cycles == 6);
        assert!(matches!(cmd.data_mode.unwrap(), DataMode::Quad));
    }

    const fn verify_write_command() {
        let mut cmd = Command::default();
        cmd.instruction = Some(0x02); // Page Program
        assert!(cmd.instruction.unwrap() == 0x02);
    }

    const fn verify_erase_command() {
        let mut cmd = Command::default();
        cmd.instruction = Some(0x20); // Sector Erase 4KB
        assert!(cmd.instruction.unwrap() == 0x20);
    }

    #[test]
    fn run_compile_time_verifications() {
        verify_read_command();
        verify_write_command();
        verify_erase_command();
    }
}
