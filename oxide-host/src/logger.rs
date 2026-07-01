use std::fs::File;
use std::io::Write;
use std::path::Path;
use oxide_protocol::{TelemetryBatch, DtcEvent};
use crate::TelemetrySubscriber;

pub struct DataLogger {
    log_file: Option<File>,
    buffer: Vec<u8>,
    max_buffer_size: usize,
}

impl DataLogger {
    pub fn new(path: &Path) -> Self {
        Self {
            log_file: File::create(path).ok(),
            buffer: Vec::new(),
            max_buffer_size: 1024 * 1024, // 1MB
        }
    }

    fn flush_buffer(&mut self) {
        if let Some(file) = &mut self.log_file {
            file.write_all(&self.buffer).ok();
            self.buffer.clear();
        }
    }
}

impl TelemetrySubscriber for DataLogger {
    fn on_telemetry(&self, telemetry: TelemetryBatch) {
        let mut data = postcard::to_allocvec(&telemetry).unwrap();
        self.buffer.append(&mut data);
        if self.buffer.len() > self.max_buffer_size {
            self.flush_buffer();
        }
    }

    fn on_dtc(&self, dtc: DtcEvent) {
        let mut data = postcard::to_allocvec(&dtc).unwrap();
        self.buffer.append(&mut data);
    }
}

impl Drop for DataLogger {
    fn drop(&mut self) {
        self.flush_buffer();
    }
}

pub struct LogPlayback {
    data: Vec<u8>,
    cursor: usize,
}

impl LogPlayback {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        Ok(Self { data, cursor: 0 })
    }
}

impl Iterator for LogPlayback {
    type Item = (TelemetryBatch);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.data.len() {
            return None;
        }
        let (telemetry, used) = postcard::from_bytes_with_flavor::<TelemetryBatch, postcard::flavors::Slice>(&self.data[self.cursor..]).ok()?;
        self.cursor += used;
        Some(telemetry)
    }
}
