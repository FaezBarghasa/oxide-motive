//! Non-blocking USB CDC transport implementation.

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use stm32h7xx_hal::pac;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::otg_fs::{UsbBus, UsbPins};

use crate::hal::Transport;

pub enum UsbError {
    Usb(UsbError),
    BufferFull,
}

pub struct UsbCdcTransport<'a> {
    serial: SerialPort<'a, UsbBus<pac::OTG_FS>>,
    usb_dev: UsbDevice<'a, UsbBus<pac::OTG_FS>>,
}

impl<'a> UsbCdcTransport<'a> {
    pub fn new(
        usb: pac::OTG_FS,
        clocks: &stm32h7xx_hal::rcc::CoreClocks,
        pins: impl UsbPins<pac::OTG_FS>,
        usb_bus: &'static mut Option<usb_device::bus::UsbBusAllocator<UsbBus<pac::OTG_FS>>>,
    ) -> Self {
        let usb = stm32h7xx_hal::otg_fs::USB::new(usb, clocks, pins);
        *usb_bus = Some(UsbBus::new(usb));
        let serial = SerialPort::new(usb_bus.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(usb_bus.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Oxide Motive")
            .product("Oxide ECU")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        Self { serial, usb_dev }
    }

    // This would be called in the USB interrupt handler
    pub fn handle_interrupt(&mut self) {
        self.usb_dev.poll(&mut [&mut self.serial]);
    }
}

impl<'a> Transport for UsbCdcTransport<'a> {
    type Error = UsbError;

    fn send_non_blocking(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        match self.serial.write(data) {
            Ok(count) => Ok(count),
            Err(UsbError::WouldBlock) => Ok(0),
            Err(e) => Err(UsbError::Usb(e)),
        }
    }

    fn receive_non_blocking(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self.serial.read(buf) {
            Ok(count) => Ok(count),
            Err(UsbError::WouldBlock) => Ok(0),
            Err(e) => Err(UsbError::Usb(e)),
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.serial.flush().map_err(UsbError::Usb)
    }
}

#[cfg(test)]
mod tests {
    // USB device testing is complex on host and typically requires a real device
    // or a sophisticated hardware-in-the-loop setup.
    // We will rely on compile-time checks for this mock.
    #[test]
    fn test_usb_transport_compiles() {
        // This test ensures the structure compiles, but cannot test functionality.
        assert!(true);
    }
}
