
pub enum UdsService {
    DiagnosticSessionControl = 0x10,
    ReadDataByIdentifier = 0x22,
    WriteDataByIdentifier = 0x2E,
    ReadDTCs = 0x19,
    ClearDTCs = 0x14,
    RoutineControl = 0x31,
}

pub enum Nrc {
    SubFunctionNotSupported = 0x12,
    ConditionsNotCorrect = 0x22,
}

pub fn handle_uds_request(request: &[u8]) -> Vec<u8> {
    let service_id = request[0];
    let sub_function = if request.len() > 1 { Some(request[1]) } else { None };

    match service_id {
        s if s == UdsService::DiagnosticSessionControl as u8 => {
            vec![0x50, sub_function.unwrap_or(0)]
        }
        s if s == UdsService::ReadDataByIdentifier as u8 => {
            if request.len() >= 3 {
                let did = u16::from_be_bytes([request[1], request[2]]);
                let data = match did {
                    0xF190 => vec![0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37], // VIN
                    _ => vec![0xDE, 0xAD, 0xBE, 0xEF], // Mock data for other DIDs
                };
                let mut response = vec![0x62, request[1], request[2]];
                response.extend_from_slice(&data);
                response
            } else {
                vec![0x7F, service_id, Nrc::ConditionsNotCorrect as u8]
            }
        }
        _ => {
            vec![0x7F, service_id, Nrc::SubFunctionNotSupported as u8]
        }
    }
}
use heapless::Vec;
