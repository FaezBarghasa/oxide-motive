#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use heapless::Vec;

// This is a marker trait for the build script
pub trait OxideSlint {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, OxideSlint)]
pub struct VehicleTelemetry {
    pub speed: f32,
    pub rpm: u32,
    pub soc: u8,
    pub temp: i16,
}

impl VehicleTelemetry {
    pub fn validate(&self) -> bool {
        self.soc <= 100 && self.rpm <= 16000 // Example logical bound for RPM
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, OxideSlint)]
pub struct DiagnosticDtc {
    pub code: u32,
    pub severity: u8,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, OxideSlint)]
pub struct HmiCommand {
    pub target: u8,
    pub action: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UdsRequest {
    DiagnosticSessionControl(u8),
    ReadDataByIdentifier(u16),
    // Add other UDS services as needed
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UdsResponse {
    DiagnosticSessionControl(u8),
    ReadDataByIdentifier(u16, Vec<u8, 64>),
    NegativeResponse(u8, u8), // Service ID, NRC
}


slint::include_modules!();

#[cfg(test)]
mod tests {
    use super::*;
    use postcard::{from_bytes, to_vec};
    use heapless::Vec;

    #[test]
    fn test_telemetry_serialization_deserialization() {
        let telemetry = VehicleTelemetry {
            speed: 120.5,
            rpm: 3500,
            soc: 85,
            temp: 95,
        };

        let mut buf = [0u8; 32];
        let serialized = to_vec::<_, Vec<u8, 32>>(&telemetry, &mut buf).unwrap();
        let deserialized: VehicleTelemetry = from_bytes(&serialized).unwrap();

        assert_eq!(telemetry, deserialized);
    }

    #[test]
    fn test_telemetry_validation() {
        let valid_telemetry = VehicleTelemetry {
            speed: 120.5,
            rpm: 3500,
            soc: 85,
            temp: 95,
        };
        assert!(valid_telemetry.validate());

        let invalid_soc = VehicleTelemetry {
            speed: 120.5,
            rpm: 3500,
            soc: 101,
            temp: 95,
        };
        assert!(!invalid_soc.validate());

        let invalid_rpm = VehicleTelemetry {
            speed: 120.5,
            rpm: 20000,
            soc: 85,
            temp: 95,
        };
        assert!(!invalid_rpm.validate());
    }
}

#[cfg(feature = "std")]
#[cfg(feature = "divan")]
mod benches {
    use super::*;
    use divan::{black_box, Bencher};
    use postcard::to_vec;
    use heapless::Vec;

    #[divan::bench]
    fn bench_serialize_telemetry(bencher: Bencher) {
        let telemetry = VehicleTelemetry {
            speed: 120.5,
            rpm: 3500,
            soc: 85,
            temp: 95,
        };
        let mut buf = [0u8; 32];
        bencher.iter(|| {
            let serialized = to_vec::<_, Vec<u8, 32>>(&black_box(telemetry.clone()), &mut buf).unwrap();
            black_box(serialized);
        });
    }
}