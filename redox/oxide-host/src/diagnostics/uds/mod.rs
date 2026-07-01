
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
            // TODO: Implement DID reading
            vec![0x62, request[1], request[2], 0xDE, 0xAD, 0xBE, 0xEF]
        }
        _ => {
            vec![0x7F, service_id, Nrc::SubFunctionNotSupported as u8]
        }
    }
}
use heapless::Vec;
