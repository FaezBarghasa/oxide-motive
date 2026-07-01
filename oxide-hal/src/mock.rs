use crate::{CanBus, CanFrame, EngineTimer, HighResAdc, TriggerCapture};

#[derive(Debug, PartialEq)]
pub enum MockError {
    NotImplemented,
    InvalidChannel,
    BufferTooSmall,
}

pub struct MockAdc;

impl HighResAdc for MockAdc {
    type Error = MockError;

    fn read_channel(&mut self, channel: u8) -> Result<u16, Self::Error> {
        if channel > 15 {
            return Err(MockError::InvalidChannel);
        }
        // Return a dummy value for testing
        Ok(channel as u16 * 100)
    }

    fn read_all_dma(&mut self, buffer: &mut [u16]) -> Result<(), Self::Error> {
        if buffer.is_empty() {
            return Err(MockError::BufferTooSmall);
        }
        for (i, val) in buffer.iter_mut().enumerate() {
            *val = (i as u16) * 100;
        }
        Ok(())
    }
}

pub struct MockEngineTimer {
    pub frequency: u32,
    pub compare_values: [u32; 8], // Assuming up to 8 channels
    pub counter: u32,
    pub interrupt_enabled: bool,
}

impl Default for MockEngineTimer {
    fn default() -> Self {
        Self {
            frequency: 0,
            compare_values: [0; 8],
            counter: 0,
            interrupt_enabled: false,
        }
    }
}

impl EngineTimer for MockEngineTimer {
    type Error = MockError;

    fn set_frequency(&mut self, freq_hz: u32) -> Result<(), Self::Error> {
        self.frequency = freq_hz;
        Ok(())
    }

    fn set_compare(&mut self, channel: u8, ticks: u32) -> Result<(), Self::Error> {
        if (channel as usize) >= self.compare_values.len() {
            return Err(MockError::InvalidChannel);
        }
        self.compare_values[channel as usize] = ticks;
        Ok(())
    }

    fn get_counter(&self) -> u32 {
        self.counter
    }

    fn enable_interrupt(&mut self) -> Result<(), Self::Error> {
        self.interrupt_enabled = true;
        Ok(())
    }
}

pub struct MockTriggerCapture {
    pub captured_value: u32,
}

impl Default for MockTriggerCapture {
    fn default() -> Self {
        Self {
            captured_value: 0,
        }
    }
}

impl TriggerCapture for MockTriggerCapture {
    type Error = MockError;

    fn capture_rising_edge(&mut self) -> Result<u32, Self::Error> {
        // Simulate a capture
        self.captured_value += 100;
        Ok(self.captured_value)
    }

    fn capture_falling_edge(&mut self) -> Result<u32, Self::Error> {
        // Simulate a capture
        self.captured_value += 50;
        Ok(self.captured_value)
    }
}

pub struct MockCanBus {
    pub sent_frames: heapless::Deque<(u32, heapless::Vec<u8, 8>)>,
    pub received_frames: heapless::Deque<CanFrame>,
}

impl Default for MockCanBus {
    fn default() -> Self {
        Self {
            sent_frames: heapless::Deque::new(),
            received_frames: heapless::Deque::new(),
        }
    }
}

impl CanBus for MockCanBus {
    type Error = MockError;

    fn send_frame(&mut self, id: u32, data: &[u8]) -> Result<(), Self::Error> {
        let mut vec_data = heapless::Vec::new();
        vec_data.extend_from_slice(data).map_err(|_| MockError::BufferTooSmall)?;
        self.sent_frames.push_back((id, vec_data)).map_err(|_| MockError::BufferTooSmall)?;
        Ok(())
    }

    fn receive_frame(&mut self, buffer: &mut CanFrame) -> Result<(), Self::Error> {
        if let Some(frame) = self.received_frames.pop_front() {
            *buffer = frame;
            Ok(())
        } else {
            Err(MockError::NotImplemented) // Or a more specific "no data" error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;

    #[test]
    fn test_mock_adc_read_channel() {
        let mut adc = MockAdc;
        assert_eq!(adc.read_channel(0), Ok(0));
        assert_eq!(adc.read_channel(5), Ok(500));
        assert_eq!(adc.read_channel(16), Err(MockError::InvalidChannel));
    }

    #[test]
    fn test_mock_adc_read_all_dma() {
        let mut adc = MockAdc;
        let mut buffer = [0u16; 4];
        assert_eq!(adc.read_all_dma(&mut buffer), Ok(()));
        assert_eq!(buffer, [0, 100, 200, 300]);

        let mut empty_buffer = [];
        assert_eq!(adc.read_all_dma(&mut empty_buffer), Err(MockError::BufferTooSmall));
    }

    #[test]
    fn test_mock_engine_timer() {
        let mut timer = MockEngineTimer::default();
        assert_eq!(timer.set_frequency(1000), Ok(()));
        assert_eq!(timer.frequency, 1000);

        assert_eq!(timer.set_compare(0, 500), Ok(()));
        assert_eq!(timer.compare_values[0], 500);
        assert_eq!(timer.set_compare(7, 100), Ok(()));
        assert_eq!(timer.compare_values[7], 100);
        assert_eq!(timer.set_compare(8, 100), Err(MockError::InvalidChannel));

        timer.counter = 123;
        assert_eq!(timer.get_counter(), 123);

        assert_eq!(timer.enable_interrupt(), Ok(()));
        assert!(timer.interrupt_enabled);
    }

    #[test]
    fn test_mock_trigger_capture() {
        let mut capture = MockTriggerCapture::default();
        assert_eq!(capture.capture_rising_edge(), Ok(100));
        assert_eq!(capture.capture_falling_edge(), Ok(150));
        assert_eq!(capture.captured_value, 150);
    }

    #[test]
    fn test_mock_can_bus() {
        let mut can = MockCanBus::default();
        let data1: Vec<u8, 8> = Vec::from_slice(&[1, 2, 3]).unwrap();
        let data2: Vec<u8, 8> = Vec::from_slice(&[4, 5]).unwrap();

        assert_eq!(can.send_frame(0x100, &data1), Ok(()));
        assert_eq!(can.send_frame(0x200, &data2), Ok(()));
        assert_eq!(can.sent_frames.len(), 2);
        assert_eq!(can.sent_frames.pop_front(), Some((0x100, data1)));
        assert_eq!(can.sent_frames.pop_front(), Some((0x200, data2)));

        let mut received_frame = CanFrame;
        can.received_frames.push_back(CanFrame);
        assert_eq!(can.receive_frame(&mut received_frame), Ok(()));
        assert_eq!(can.receive_frame(&mut received_frame), Err(MockError::NotImplemented));
    }
}
