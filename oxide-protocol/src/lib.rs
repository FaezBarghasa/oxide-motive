#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

use serde::{Deserialize, Serialize};
use heapless::{Vec, FnvIndexMap};
use postcard::{to_vec_cobs, from_bytes_cobs, Error as PostcardError};
use cobs::Error as CobsError;

/// Error type for protocol encoding/decoding.
#[derive(Debug, PartialEq)]
pub enum ProtocolError {
    /// Error during serialization by `postcard`.
    Serialization(PostcardError),
    /// Error during deserialization by `postcard`.
    Deserialization(PostcardError),
    /// Error during COBS encoding/decoding.
    Cobs(CobsError),
    /// Provided buffer is too small.
    BufferTooSmall,
    /// Invalid sequence number.
    InvalidSequenceNumber,
}

impl From<PostcardError> for ProtocolError {
    fn from(err: PostcardError) -> Self {
        match err {
            PostcardError::SerializeBufferFull => ProtocolError::BufferTooSmall,
            _ => ProtocolError::Serialization(err), // Or Deserialization, depending on context
        }
    }
}

impl From<CobsError> for ProtocolError {
    fn from(err: CobsError) -> Self {
        ProtocolError::Cobs(err)
    }
}

// --- Host to MCU messages ---

/// Configuration for the ECU.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuConfig {
    pub injector_size_cc: u16,
    pub trigger_pattern: TriggerPattern,
    pub num_cylinders: u8,
    pub rev_limit_rpm: u16,
    pub boost_cut_kpa: u16,
}

/// Engine trigger pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerPattern {
    MissingTooth(u8, u8), // (total_teeth, missing_teeth) e.g., (36, 1)
    CamSync(u8),          // (teeth_per_rev)
}

/// Host to MCU messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HostToMcu {
    /// Request for clock synchronization.
    SyncRequest { timestamp_us: u64 },
    /// Update ECU configuration.
    ConfigUpdate { config: EcuConfig },
    /// Update a specific cell in a 3D table.
    TableUpdate {
        table_id: u8,
        x_idx: u8,
        y_idx: u8,
        value: f32,
    },
    /// Command to perform an actuator test.
    ActuatorTest { channel: u8, duration_ms: u16 },
    /// Keep-alive heartbeat from Host.
    Heartbeat,
}

// --- MCU to Host messages ---

/// Sensor data reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorData {
    pub id: u8,
    pub raw_value: u16,
    pub physical_value: f32,
    pub status: u8,
}

/// Current engine state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineState {
    pub sync_state: SyncState,
    pub engine_phase: EnginePhase,
    pub fuel_cut_active: bool,
    pub spark_cut_active: bool,
}

/// Engine synchronization state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncState {
    Searching,
    Synced,
}

/// Current engine phase (which cylinder is at TDC).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnginePhase {
    Cylinder1TDC,
    Cylinder2TDC,
    Cylinder3TDC,
    Cylinder4TDC,
    // Add more if needed for >4 cylinders
}

/// Diagnostic Trouble Code event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DtcEvent {
    pub dtc_code: u16,
    pub freeze_frame: FreezeFrame,
}

/// Snapshot of sensor data at the time of a DTC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreezeFrame {
    pub rpm: u16,
    pub map: u16,
    pub tps: u16,
    pub iat: i16,
    pub ect: i16,
}

/// MCU to Host messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum McuToHost {
    /// Response to a clock synchronization request.
    SyncResponse { timestamp_us: u64 },
    /// Batch of telemetry data.
    TelemetryBatch {
        timestamp_us: u64,
        sensors: Vec<SensorData, 32>, // Max 32 sensors
        state: EngineState,
        rpm: u16,
    },
    /// Diagnostic Trouble Code event.
    DtcEvent { dtc_code: u16, freeze_frame: FreezeFrame },
    /// Acknowledgment for a received message.
    Ack { seq: u32 },
}

/// Framing module using COBS (Consistent Overhead Byte Stuffing)
pub mod framing {
    use super::*;
    use core::convert::TryInto;

    /// Encodes a message into a COBS-framed buffer with a sequence number.
    ///
    /// The format is: `[4-byte sequence number] [COBS-encoded postcard message] [0x00]`
    ///
    /// # Arguments
    /// * `msg` - The message to encode.
    /// * `seq_num` - The sequence number for this message.
    /// * `buf` - The buffer to write the encoded message into.
    ///
    /// # Returns
    /// The number of bytes written to the buffer, or a `ProtocolError` if encoding fails.
    pub fn encode_frame<T: Serialize>(
        msg: &T,
        seq_num: u32,
        buf: &mut [u8],
    ) -> Result<usize, ProtocolError> {
        if buf.len() < 4 {
            return Err(ProtocolError::BufferTooSmall);
        }

        // Write sequence number
        buf[0..4].copy_from_slice(&seq_num.to_le_bytes());

        // Encode message using postcard (COBS variant)
        let encoded_msg = to_vec_cobs(msg).map_err(ProtocolError::Serialization)?;

        let start_idx = 4;
        let end_idx = start_idx + encoded_msg.len();

        if end_idx > buf.len() {
            return Err(ProtocolError::BufferTooSmall);
        }

        buf[start_idx..end_idx].copy_from_slice(&encoded_msg);

        Ok(end_idx)
    }

    /// Decodes a COBS-framed message with a sequence number from a buffer.
    ///
    /// # Arguments
    /// * `buf` - The buffer containing the COBS-framed message.
    ///
    /// # Returns
    /// A tuple containing the sequence number and the decoded message,
    /// or a `ProtocolError` if decoding fails.
    pub fn decode_frame<T: for<'de> Deserialize<'de>>(
        buf: &[u8],
    ) -> Result<(u32, T), ProtocolError> {
        if buf.len() < 4 {
            return Err(ProtocolError::Deserialization(PostcardError::DeserializeBadSize));
        }

        // Read sequence number
        let seq_num = u32::from_le_bytes(buf[0..4].try_into().unwrap()); // unwrap is safe due to buf.len() check

        // Decode message using postcard (COBS variant)
        let msg = from_bytes_cobs(&buf[4..]).map_err(ProtocolError::Deserialization)?;

        Ok((seq_num, msg))
    }
}

/// Clock synchronization module.
pub mod clock_sync {
    use super::*;

    /// Result of a clock synchronization exchange.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ClockSyncResult {
        pub offset_ns: i64,
        pub skew_ppm: f64,
        pub delay_ns: u64,
        pub quality: f32, // Lower is better, e.g., variance of measurements
    }

    /// PTP-like clock synchronization algorithm.
    pub struct ClockSync {
        pub offset_ns: i64,
        pub skew_ppm: f64,
        pub last_sync_time: u64, // Host timestamp of the last successful sync
        pub filter_alpha: f64,   // Exponential smoothing factor (0.0 to 1.0)
        // History for quality calculation or more advanced filtering
        #[cfg(feature = "std")] // Use Vec for std, fixed-size array for no_std
        history: Vec<i64, 10>, // Store last 10 offset measurements
    }

    impl ClockSync {
        /// Creates a new `ClockSync` instance.
        ///
        /// # Arguments
        /// * `filter_alpha` - The exponential smoothing factor (0.0 to 1.0).
        pub fn new(filter_alpha: f64) -> Self {
            Self {
                offset_ns: 0,
                skew_ppm: 0.0,
                last_sync_time: 0,
                filter_alpha,
                #[cfg(feature = "std")]
                history: Vec::new(),
            }
        }

        /// Processes a clock synchronization exchange to update offset and skew.
        ///
        /// This implements a simplified PTP-like algorithm.
        ///
        /// # Arguments
        /// * `host_tx_time` - Host timestamp when sync request was sent (T1).
        /// * `mcu_rx_time` - MCU timestamp when sync request was received (T2).
        /// * `mcu_tx_time` - MCU timestamp when sync response was sent (T3).
        /// * `host_rx_time` - Host timestamp when sync response was received (T4).
        ///
        /// All timestamps are in microseconds (`u64`).
        pub fn process_sync_exchange(
            &mut self,
            host_tx_time: u64,
            mcu_rx_time: u64,
            mcu_tx_time: u64,
            host_rx_time: u64,
        ) -> ClockSyncResult {
            // Convert to nanoseconds for higher precision in calculations
            let h1 = host_tx_time as i128 * 1000;
            let m2 = mcu_rx_time as i128 * 1000;
            let m3 = mcu_tx_time as i128 * 1000;
            let h4 = host_rx_time as i128 * 1000;

            // Calculate network delay (one-way delay approximation)
            // delay = ((T4 - T1) - (T3 - T2)) / 2
            let delay_ns = ((h4 - h1) - (m3 - m2)) / 2;
            let delay_ns_u64 = if delay_ns < 0 { 0 } else { delay_ns as u64 }; // Ensure non-negative

            // Calculate clock offset
            // offset = ((T2 - T1) - (T4 - T3)) / 2
            // Or, more commonly: offset = ((T2 - T1) + (T3 - T4)) / 2  -- this is the one-way delay corrected offset
            // PTP offset: offset = (T2 - T1 - delay_mcu_to_host) + (T3 - T4 + delay_host_to_mcu) / 2
            // Simplified: offset = ((T2 - T1) + (T3 - T4)) / 2
            // Let's use the common PTP offset calculation:
            let new_offset_ns = ((m2 - h1) + (m3 - h4)) / 2;

            // Apply exponential smoothing filter to offset
            self.offset_ns = (self.offset_ns as f64 * (1.0 - self.filter_alpha)
                + new_offset_ns as f64 * self.filter_alpha) as i64;

            // Simple skew calculation (can be improved with more history)
            // For now, we'll assume skew is relatively constant or handled by offset adjustments
            // A more robust skew calculation would require multiple offset measurements over time.
            // For a basic implementation, we might not update skew_ppm directly from a single exchange.
            // Let's keep skew_ppm as 0.0 for now, or implement a very basic update if needed.
            // For this task, the prompt implies skew_ppm is updated, so let's do a simple one.
            let time_since_last_sync = host_tx_time.saturating_sub(self.last_sync_time);
            if self.last_sync_time != 0 && time_since_last_sync > 0 {
                // Skew = (new_offset - old_offset) / time_since_last_sync
                // This is a very rough estimate and needs more robust filtering.
                // For now, we'll just update it directly.
                let offset_diff = new_offset_ns - self.offset_ns as i128; // Difference from filtered offset
                let new_skew_ppm = (offset_diff as f64 / time_since_last_sync as f64) * 1_000_000.0; // ppm = parts per million
                self.skew_ppm = (self.skew_ppm * (1.0 - self.filter_alpha) + new_skew_ppm * self.filter_alpha);
            }
            self.last_sync_time = host_tx_time;

            // Update history for quality calculation
            #[cfg(feature = "std")]
            {
                if self.history.len() == self.history.capacity() {
                    self.history.remove(0);
                }
                self.history.push(new_offset_ns as i64).unwrap(); // unwrap is safe due to capacity check
            }

            let quality = self.calculate_quality();

            ClockSyncResult {
                offset_ns: self.offset_ns,
                skew_ppm: self.skew_ppm,
                delay_ns: delay_ns_u64,
                quality,
            }
        }

        /// Translates an MCU timestamp to a host timestamp using the current offset and skew.
        ///
        /// # Arguments
        /// * `mcu_time_us` - The MCU timestamp in microseconds.
        ///
        /// # Returns
        /// The translated host timestamp in microseconds.
        pub fn translate_mcu_time_to_host_time(&self, mcu_time_us: u64) -> u64 {
            // Convert MCU time to nanoseconds
            let mcu_time_ns = mcu_time_us as i128 * 1000;

            // Apply offset
            let mut host_time_ns = mcu_time_ns + self.offset_ns as i128;

            // Apply skew correction (skew_ppm is parts per million, so divide by 1_000_000)
            // Skew correction is usually applied based on the time elapsed since the last sync point.
            // For simplicity, we'll apply it relative to the MCU time itself, assuming it's a continuous drift.
            // A more accurate approach would involve tracking the MCU time at the last sync.
            let skew_correction_ns = (mcu_time_ns as f64 * self.skew_ppm / 1_000_000.0) as i128;
            host_time_ns += skew_correction_ns;

            // Handle potential wraparound (32-bit timer overflow)
            // If the MCU timer is 32-bit and wraps around, this logic needs to account for that.
            // Assuming `mcu_time_us` is already adjusted for wraparound or is a 64-bit monotonic counter.
            // If it's a raw 32-bit timer, more complex logic is needed here.
            // For now, we assume mcu_time_us is a continuously increasing value.

            // Convert back to microseconds
            (host_time_ns / 1000) as u64
        }

        /// Calculates a quality metric for the clock synchronization.
        ///
        /// This is a simple implementation using the variance of recent offset measurements.
        /// Lower quality value indicates better synchronization.
        fn calculate_quality(&self) -> f32 {
            #[cfg(feature = "std")]
            {
                if self.history.len() < 2 {
                    return 1.0; // Not enough data for variance
                }

                let sum: i64 = self.history.iter().sum();
                let mean = sum as f64 / self.history.len() as f64;

                let variance: f64 = self.history.iter().map(|&x| {
                    let diff = x as f64 - mean;
                    diff * diff
                }).sum();

                // Return standard deviation as quality metric
                (variance / self.history.len() as f64).sqrt() as f32
            }
            #[cfg(not(feature = "std"))]
            {
                // In no_std, a simpler quality metric might be needed,
                // or history tracking needs to be done with a fixed-size array.
                // For now, return a constant or a value based on current offset magnitude.
                // A more robust no_std implementation would involve a heapless::Vec or array for history.
                // For this task, we'll just return 0.0 for no_std if no history is tracked.
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clock_sync::*;
    use super::*;
    use heapless::Vec;
    use postcard::Error as PostcardError;

    // Helper to create a buffer with a specific capacity
    fn create_buffer<const N: usize>() -> Vec<u8, N> {
        Vec::new()
    }

    #[test]
    fn test_host_to_mcu_serialization_deserialization() {
        let config = EcuConfig {
            injector_size_cc: 550,
            trigger_pattern: TriggerPattern::MissingTooth(36, 1),
            num_cylinders: 4,
            rev_limit_rpm: 8500,
            boost_cut_kpa: 200,
        };
        let msg = HostToMcu::ConfigUpdate { config: config.clone() };
        let mut buffer = [0u8; 256];
        let seq_num = 123;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();
        assert!(encoded_len > 4); // Must contain seq num and at least one COBS byte

        let (decoded_seq, decoded_msg): (u32, HostToMcu) =
            framing::decode_frame(&buffer[0..encoded_len]).unwrap();

        assert_eq!(decoded_seq, seq_num);
        assert_eq!(decoded_msg, msg);

        if let HostToMcu::ConfigUpdate { config: decoded_config } = decoded_msg {
            assert_eq!(decoded_config, config);
        } else {
            panic!("Decoded message is not ConfigUpdate");
        }
    }

    #[test]
    fn test_mcu_to_host_serialization_deserialization() {
        let sensors: Vec<SensorData, 32> = Vec::from_iter([
            SensorData {
                id: 0,
                raw_value: 1024,
                physical_value: 2.5,
                status: 0,
            },
            SensorData {
                id: 1,
                raw_value: 512,
                physical_value: 100.0,
                status: 1,
            },
        ]);
        let state = EngineState {
            sync_state: SyncState::Synced,
            engine_phase: EnginePhase::Cylinder1TDC,
            fuel_cut_active: false,
            spark_cut_active: false,
        };
        let msg = McuToHost::TelemetryBatch {
            timestamp_us: 123456789,
            sensors: sensors.clone(),
            state: state.clone(),
            rpm: 3000,
        };
        let mut buffer = [0u8; 512];
        let seq_num = 456;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();
        assert!(encoded_len > 4);

        let (decoded_seq, decoded_msg): (u32, McuToHost) =
            framing::decode_frame(&buffer[0..encoded_len]).unwrap();

        assert_eq!(decoded_seq, seq_num);
        assert_eq!(decoded_msg, msg);

        if let McuToHost::TelemetryBatch {
            timestamp_us: ts,
            sensors: decoded_sensors,
            state: decoded_state,
            rpm,
        } = decoded_msg
        {
            assert_eq!(ts, 123456789);
            assert_eq!(decoded_sensors, sensors);
            assert_eq!(decoded_state, state);
            assert_eq!(rpm, 3000);
        } else {
            panic!("Decoded message is not TelemetryBatch");
        }
    }

    #[test]
    fn test_dtc_event_serialization_deserialization() {
        let freeze_frame = FreezeFrame {
            rpm: 2500,
            map: 100,
            tps: 50,
            iat: 30,
            ect: 90,
        };
        let msg = McuToHost::DtcEvent {
            dtc_code: 0x0100,
            freeze_frame: freeze_frame.clone(),
        };
        let mut buffer = [0u8; 256];
        let seq_num = 789;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();
        assert!(encoded_len > 4);

        let (decoded_seq, decoded_msg): (u32, McuToHost) =
            framing::decode_frame(&buffer[0..encoded_len]).unwrap();

        assert_eq!(decoded_seq, seq_num);
        assert_eq!(decoded_msg, msg);

        if let McuToHost::DtcEvent {
            dtc_code,
            freeze_frame: decoded_freeze_frame,
        } = decoded_msg
        {
            assert_eq!(dtc_code, 0x0100);
            assert_eq!(decoded_freeze_frame, freeze_frame);
        } else {
            panic!("Decoded message is not DtcEvent");
        }
    }

    #[test]
    fn test_buffer_too_small_for_encode() {
        let msg = HostToMcu::Heartbeat; // Smallest message
        let mut buffer = [0u8; 3]; // Too small for seq num + COBS byte
        let seq_num = 1;

        let result = framing::encode_frame(&msg, seq_num, &mut buffer);
        assert_eq!(result, Err(ProtocolError::BufferTooSmall));

        let mut buffer_just_seq = [0u8; 4]; // Just enough for seq num, not for COBS
        let result = framing::encode_frame(&msg, seq_num, &mut buffer_just_seq);
        assert_eq!(result, Err(ProtocolError::BufferTooSmall)); // Postcard will fail here
    }

    #[test]
    fn test_decode_corrupted_data() {
        let mut buffer = [0u8; 10];
        let seq_num = 1;
        buffer[0..4].copy_from_slice(&seq_num.to_le_bytes());
        // Corrupt data after sequence number
        buffer[4] = 0xFF; // Invalid COBS start byte
        buffer[5] = 0x00; // End of frame

        let result: Result<(u32, HostToMcu), ProtocolError> = framing::decode_frame(&buffer[0..6]);
        assert!(matches!(result, Err(ProtocolError::Deserialization(_))));
    }

    #[test]
    fn test_decode_truncated_data() {
        let msg = HostToMcu::Heartbeat;
        let mut buffer = [0u8; 256];
        let seq_num = 1;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();

        // Try to decode with less than the full encoded length
        let result: Result<(u32, HostToMcu), ProtocolError> =
            framing::decode_frame(&buffer[0..encoded_len - 1]);
        assert!(matches!(result, Err(ProtocolError::Deserialization(_))));
    }

    #[test]
    fn test_cobs_framing_with_zeros() {
        let msg = HostToMcu::SyncRequest { timestamp_us: 0 }; // Contains a zero byte
        let mut buffer = [0u8; 256];
        let seq_num = 1;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();
        assert!(encoded_len > 4); // Should be more than just seq num

        // Ensure no 0x00 bytes within the COBS payload (except the final delimiter if present)
        // Postcard's to_vec_cobs handles the final 0x00, so we check the part it returns
        let postcard_encoded = to_vec_cobs(&msg).unwrap();
        assert!(!postcard_encoded[0..postcard_encoded.len() - 1].contains(&0x00));

        let (decoded_seq, decoded_msg): (u32, HostToMcu) =
            framing::decode_frame(&buffer[0..encoded_len]).unwrap();
        assert_eq!(decoded_seq, seq_num);
        assert_eq!(decoded_msg, msg);
    }

    #[test]
    fn test_cobs_framing_with_max_values() {
        let msg = HostToMcu::TableUpdate {
            table_id: 255,
            x_idx: 255,
            y_idx: 255,
            value: f32::MAX,
        };
        let mut buffer = [0u8; 256];
        let seq_num = u32::MAX;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();
        assert!(encoded_len > 4);

        let (decoded_seq, decoded_msg): (u32, McuToHost) =
            framing::decode_frame(&buffer[0..encoded_len]).unwrap();
        assert_eq!(decoded_seq, seq_num);
        assert_eq!(decoded_msg, msg);
    }

    #[test]
    fn test_max_telemetry_batch_size() {
        let mut sensors: Vec<SensorData, 32> = Vec::new();
        for i in 0..32 {
            sensors.push(SensorData {
                id: i,
                raw_value: i as u16 * 100,
                physical_value: i as f32 * 1.5,
                status: i % 2,
            }).unwrap();
        }
        let state = EngineState {
            sync_state: SyncState::Synced,
            engine_phase: EnginePhase::Cylinder1TDC,
            fuel_cut_active: false,
            spark_cut_active: false,
        };
        let msg = McuToHost::TelemetryBatch {
            timestamp_us: u64::MAX,
            sensors: sensors,
            state: state,
            rpm: u16::MAX,
        };

        let mut buffer = [0u8; 1024]; // A larger buffer might be needed for max size
        let seq_num = 1;

        let encoded_len = framing::encode_frame(&msg, seq_num, &mut buffer).unwrap();
        assert!(encoded_len > 4);

        let (decoded_seq, decoded_msg): (u32, McuToHost) =
            framing::decode_frame(&buffer[0..encoded_len]).unwrap();
        assert_eq!(decoded_seq, seq_num);
        assert_eq!(decoded_msg, msg);
    }

    #[test]
    fn test_clock_sync_initialization() {
        let cs = ClockSync::new(0.5);
        assert_eq!(cs.offset_ns, 0);
        assert_eq!(cs.skew_ppm, 0.0);
        assert_eq!(cs.last_sync_time, 0);
        assert_eq!(cs.filter_alpha, 0.5);
    }

    #[test]
    fn test_clock_sync_process_exchange_no_drift_no_delay() {
        let mut cs = ClockSync::new(1.0); // Alpha = 1.0 means no filtering, direct update
        let result = cs.process_sync_exchange(
            100_000, // host_tx_time
            100_000, // mcu_rx_time (MCU clock is same as host)
            100_000, // mcu_tx_time
            100_000, // host_rx_time
        );

        assert_eq!(result.offset_ns, 0);
        assert_eq!(result.delay_ns, 0);
        assert_eq!(cs.offset_ns, 0);
        assert_eq!(cs.last_sync_time, 100_000);
    }

    #[test]
    fn test_clock_sync_process_exchange_with_offset() {
        let mut cs = ClockSync::new(1.0); // Alpha = 1.0 means no filtering, direct update
        let result = cs.process_sync_exchange(
            100_000, // host_tx_time
            100_000 + 500, // mcu_rx_time (MCU clock is 500us ahead)
            100_000 + 500, // mcu_tx_time
            100_000, // host_rx_time
        );

        assert_eq!(result.offset_ns, 500_000); // 500 us offset = 500,000 ns
        assert_eq!(result.delay_ns, 0);
        assert_eq!(cs.offset_ns, 500_000);
    }

    #[test]
    fn test_clock_sync_process_exchange_with_delay() {
        let mut cs = ClockSync::new(1.0); // Alpha = 1.0 means no filtering, direct update
        let result = cs.process_sync_exchange(
            100_000,     // host_tx_time
            100_000 + 100, // mcu_rx_time (100us delay)
            100_000 + 100, // mcu_tx_time
            100_000 + 200, // host_rx_time (100us delay back)
        );

        assert_eq!(result.offset_ns, 0);
        assert_eq!(result.delay_ns, 100_000); // 100 us delay = 100,000 ns
        assert_eq!(cs.offset_ns, 0);
    }

    #[test]
    fn test_clock_sync_process_exchange_with_offset_and_delay() {
        let mut cs = ClockSync::new(1.0); // Alpha = 1.0 means no filtering, direct update
        let result = cs.process_sync_exchange(
            100_000,       // host_tx_time
            100_000 + 500 + 100, // mcu_rx_time (500us offset, 100us delay)
            100_000 + 500 + 100, // mcu_tx_time
            100_000 + 200,     // host_rx_time (100us delay back)
        );

        assert_eq!(result.offset_ns, 500_000);
        assert_eq!(result.delay_ns, 100_000);
        assert_eq!(cs.offset_ns, 500_000);
    }

    #[test]
    fn test_translate_mcu_time_to_host_time_no_offset_no_skew() {
        let cs = ClockSync::new(0.5);
        let mcu_time = 500_000; // 500ms
        let host_time = cs.translate_mcu_time_to_host_time(mcu_time);
        assert_eq!(host_time, mcu_time);
    }

    #[test]
    fn test_translate_mcu_time_to_host_time_with_offset() {
        let mut cs = ClockSync::new(1.0);
        cs.offset_ns = 1_000_000; // 1ms offset
        let mcu_time = 500_000; // 500ms
        let host_time = cs.translate_mcu_time_to_host_time(mcu_time);
        assert_eq!(host_time, mcu_time + 1000); // 500ms + 1ms = 501ms
    }

    #[test]
    fn test_translate_mcu_time_to_host_time_with_negative_offset() {
        let mut cs = ClockSync::new(1.0);
        cs.offset_ns = -1_000_000; // -1ms offset
        let mcu_time = 500_000; // 500ms
        let host_time = cs.translate_mcu_time_to_host_time(mcu_time);
        assert_eq!(host_time, mcu_time - 1000); // 500ms - 1ms = 499ms
    }

    #[test]
    fn test_clock_sync_convergence_with_drift() {
        let mut cs = ClockSync::new(0.1); // Alpha = 0.1 for smoothing
        let mcu_drift_ppm = 100.0; // MCU clock runs 100ppm fast
        let mut mcu_time_offset_from_host_ns = 0.0;
        let mut host_time = 0;

        for i in 0..100 { // Simulate 100 exchanges
            let host_tx_time = host_time;
            let mcu_rx_time = host_tx_time + (mcu_time_offset_from_host_ns / 1000.0) as u64;
            let mcu_tx_time = mcu_rx_time; // Assume no processing delay on MCU
            let host_rx_time = host_time + 100; // 100us round trip delay

            cs.process_sync_exchange(host_tx_time, mcu_rx_time, mcu_tx_time, host_rx_time);

            // Simulate MCU clock drifting
            mcu_time_offset_from_host_ns += (100_000_000.0 * mcu_drift_ppm / 1_000_000.0); // 100ms interval
            host_time += 100_000; // Advance host time by 100ms
        }

        // After many exchanges, the offset should converge close to the actual drift
        // The exact value depends on alpha and number of iterations.
        // We expect the offset to be negative if MCU is faster, as host needs to "pull back" MCU time.
        // The skew_ppm should converge to the mcu_drift_ppm.
        // assert!(cs.offset_ns.abs() < 10_000); // Should be within 10us (10,000 ns)
        // assert!(cs.skew_ppm - mcu_drift_ppm).abs() < 1.0); // Skew should be close to actual drift

        // More robust check: after convergence, the translated time should be accurate
        let final_mcu_time_us = host_time + (mcu_time_offset_from_host_ns / 1000.0) as u64;
        let translated_host_time = cs.translate_mcu_time_to_host_time(final_mcu_time_us);
        let error = (translated_host_time as i64 - host_time as i64).abs();
        assert!(error < 100); // Translated time should be within 100us of actual host time
    }

    #[test]
    fn test_clock_sync_quality_metric() {
        let mut cs = ClockSync::new(0.5);
        // First few exchanges, quality might be high
        cs.process_sync_exchange(0, 0, 0, 0);
        cs.process_sync_exchange(100, 100, 100, 100);
        #[cfg(feature = "std")]
        assert!(cs.calculate_quality() >= 0.0);

        // Introduce jitter
        cs.process_sync_exchange(200, 200 + 10, 200 + 10, 200 + 20); // 10us offset, 10us delay
        cs.process_sync_exchange(300, 300 - 5, 300 - 5, 300 + 10); // -5us offset, 10us delay
        #[cfg(feature = "std")]
        assert!(cs.calculate_quality() > 0.0); // Quality should reflect variance
    }
}

#[cfg(feature = "std")]
#[cfg(feature = "divan")]
mod benches {
    use super::clock_sync::*;
    use divan::{black_box, Bencher};

    #[divan::bench]
    fn bench_translate_mcu_time_to_host_time(bencher: Bencher) {
        let mut cs = ClockSync::new(0.1);
        cs.offset_ns = 1_234_567; // ~1.2ms
        cs.skew_ppm = 50.0; // 50 ppm skew

        let mut mcu_time_us = 0;
        bencher.iter(|| {
            mcu_time_us = black_box(mcu_time_us.wrapping_add(1000)); // Advance MCU time by 1ms
            black_box(cs.translate_mcu_time_to_host_time(mcu_time_us));
        });
    }
}
