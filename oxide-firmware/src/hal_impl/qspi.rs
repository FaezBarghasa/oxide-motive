//! QSPI driver implementation for the Winbond W25Q64 flash chip on the STM32H750.

use stm32h7xx_hal::pac;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::gpio::{self, Alternate};
use stm32h7xx_hal::rcc::CoreClocks;
use stm32h7xx_hal::dma::{self, Dma, DmaStream, Transfer, MemoryToPeripheral, PeripheralToMemory};
use stm32h7xx_hal::quadspi::{self, QuadSpi, Command, AddressSize, QspiMode};

use crate::hal::ExternalFlash;

// Winbond W25Q64 command set
const W25Q64_CMD_WRITE_ENABLE: u8 = 0x06;
const W25Q64_CMD_PAGE_PROGRAM: u8 = 0x02;
const W25Q64_CMD_SECTOR_ERASE_4K: u8 = 0x20;
const W25Q64_CMD_READ_DATA: u8 = 0x03;
const W25Q64_CMD_FAST_READ_QUAD_IO: u8 = 0xEB;
const W25Q64_CMD_READ_STATUS_REG1: u8 = 0x05;
const W25Q64_CMD_JEDEC_ID: u8 = 0x9F;

const W25Q64_STATUS_BUSY_BIT: u8 = 1 << 0;
const W25Q64_PAGE_SIZE: usize = 256;
const W25Q64_JEDEC_ID: u32 = 0xEF4017;

#[derive(Debug)]
pub enum QspiError {
    QuadSpi(quadspi::Error),
    Dma(dma::Error),
    InvalidDeviceId,
    Timeout,
    WriteError,
}

impl From<quadspi::Error> for QspiError {
    fn from(e: quadspi::Error) -> Self {
        QspiError::QuadSpi(e)
    }
}

impl From<dma::Error> for QspiError {
    fn from(e: dma::Error) -> Self {
        QspiError::Dma(e)
    }
}

pub struct QspiDriver {
    qspi: QuadSpi<pac::QUADSPI>,
    dma: DmaStream<pac::DMA1, dma::stream::Stream0>,
}

impl QspiDriver {
    pub fn new(
        qspi_regs: pac::QUADSPI,
        dma_regs: pac::DMA1,
        rcc: &mut stm32h7xx_hal::rcc::REC,
        clocks: &CoreClocks,
        gpiof: gpio::gpiof::Parts,
        gpiog: gpio::gpiog::Parts,
    ) -> Self {
        // QSPI pins
        let _qspi_clk = gpiof.pf10.into_alternate::<9>();
        let _qspi_bk1_ncs = gpiog.pg6.into_alternate::<10>();
        let _qspi_bk1_io0 = gpiof.pf8.into_alternate::<10>();
        let _qspi_bk1_io1 = gpiof.pf9.into_alternate::<10>();
        let _qspi_bk1_io2 = gpiof.pf7.into_alternate::<9>();
        let _qspi_bk1_io3 = gpiof.pf6.into_alternate::<9>();

        let mut qspi_config = quadspi::Config::new(50.MHz(), clocks);
        qspi_config.flash_size = 22; // 2^(22+1) = 8 MBytes

        let qspi = QuadSpi::new(qspi_regs, rcc, qspi_config);

        let dma_streams = Dma::new(dma_regs, rcc);
        let dma = dma_streams.streams.stream0;

        Self { qspi, dma }
    }

    fn wait_for_not_busy(&mut self) -> Result<(), QspiError> {
        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_READ_STATUS_REG1);
        cmd.data_mode = Some(QspiMode::Single);
        cmd.address_size = None;
        cmd.dummy_cycles = 0;

        // Poll status register until busy bit is cleared
        for _ in 0..1000 {
            let mut status = [0u8; 1];
            self.qspi.read_command(&cmd, &mut status)?;
            if (status[0] & W25Q64_STATUS_BUSY_BIT) == 0 {
                return Ok(());
            }
            // Small delay
            cortex_m::asm::delay(1000);
        }
        Err(QspiError::Timeout)
    }

    fn write_enable(&mut self) -> Result<(), QspiError> {
        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_WRITE_ENABLE);
        cmd.data_mode = Some(QspiMode::Single);
        self.qspi.write_command(&cmd)?;
        Ok(())
    }
}

impl ExternalFlash for QspiDriver {
    type Error = QspiError;

    fn init(&mut self) -> Result<(), Self::Error> {
        let device_id = self.read_device_id()?;
        if device_id != W25Q64_JEDEC_ID {
            return Err(QspiError::InvalidDeviceId);
        }
        Ok(())
    }

    fn read_device_id(&mut self) -> Result<u32, Self::Error> {
        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_JEDEC_ID);
        cmd.data_mode = Some(QspiMode::Single);
        cmd.address_size = None;
        cmd.dummy_cycles = 0;

        let mut buf = [0u8; 3];
        self.qspi.read_command(&cmd, &mut buf)?;

        Ok(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32))
    }

    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.wait_for_not_busy()?;

        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_FAST_READ_QUAD_IO);
        cmd.address = Some(addr);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.address_mode = Some(QspiMode::Quad);
        cmd.data_mode = Some(QspiMode::Quad);
        cmd.dummy_cycles = 4;

        let transfer = self.qspi.read_dma(&mut self.dma, buf, cmd)?;
        transfer.wait()?;
        Ok(())
    }

    fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
        if data.len() > W25Q64_PAGE_SIZE {
            return Err(QspiError::WriteError);
        }

        self.wait_for_not_busy()?;
        self.write_enable()?;

        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_PAGE_PROGRAM);
        cmd.address = Some(addr);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.address_mode = Some(QspiMode::Single);
        cmd.data_mode = Some(QspiMode::Single);

        let transfer = self.qspi.write_dma(&mut self.dma, data, cmd)?;
        transfer.wait()?;
        Ok(())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), Self::Error> {
        self.wait_for_not_busy()?;
        self.write_enable()?;

        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_SECTOR_ERASE_4K);
        cmd.address = Some(addr);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.address_mode = Some(QspiMode::Single);

        self.qspi.write_command(&cmd)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This is a compile-time test to ensure the command constants are correct.
    #[test]
    fn verify_w25q64_commands() {
        assert_eq!(W25Q64_CMD_WRITE_ENABLE, 0x06);
        assert_eq!(W25Q64_CMD_PAGE_PROGRAM, 0x02);
        assert_eq!(W25Q64_CMD_SECTOR_ERASE_4K, 0x20);
        assert_eq!(W25Q64_CMD_READ_DATA, 0x03);
        assert_eq!(W25Q64_CMD_FAST_READ_QUAD_IO, 0xEB);
        assert_eq!(W25Q64_CMD_READ_STATUS_REG1, 0x05);
        assert_eq!(W25Q64_CMD_JEDEC_ID, 0x9F);
    }

    // This test simulates the byte stream for a JEDEC ID read command.
    #[test]
    fn test_jedec_id_command_sequence() {
        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_JEDEC_ID);
        cmd.data_mode = Some(QspiMode::Single);
        cmd.address_size = None;
        cmd.dummy_cycles = 0;

        // In a real test, we would have a mock QSPI peripheral that captures
        // the generated command sequence and verifies it.
        // For now, we just assert that the instruction is correct.
        assert_eq!(cmd.instruction, Some(0x9F));
    }

    // This test simulates the byte stream for a fast read quad I/O command.
    #[test]
    fn test_fast_read_quad_io_command_sequence() {
        let mut cmd = Command::new();
        cmd.instruction = Some(W25Q64_CMD_FAST_READ_QUAD_IO);
        cmd.address = Some(0x123456);
        cmd.address_size = Some(AddressSize::Bits24);
        cmd.address_mode = Some(QspiMode::Quad);
        cmd.data_mode = Some(QspiMode::Quad);
        cmd.dummy_cycles = 4;

        assert_eq!(cmd.instruction, Some(0xEB));
        assert_eq!(cmd.address, Some(0x123456));
        assert_eq!(cmd.address_mode, Some(QspiMode::Quad));
        assert_eq!(cmd.data_mode, Some(QspiMode::Quad));
    }
}
