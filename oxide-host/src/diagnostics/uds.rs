
pub enum DiagnosticSession {
    Default,
    Programming,
    Extended,
}

pub struct UdsServer {
    session: DiagnosticSession,
    security_level: u8,
}

impl UdsServer {
    pub fn new() -> Self {
        Self {
            session: DiagnosticSession::Default,
            security_level: 0,
        }
    }

    pub fn handle_request(&mut self, request: &[u8]) -> Vec<u8> {
        if request.is_empty() {
            return vec![0x7F, request[0], 0x12]; // SubFunctionNotSupported
        }

        let service_id = request[0];
        match service_id {
            0x10 => self.handle_diagnostic_session_control(&request[1..]),
            0x22 => self.handle_read_data_by_identifier(&request[1..]),
            0x19 => self.handle_read_dtcs(&request[1..]),
            0x14 => self.handle_clear_dtcs(&request[1..]),
            _ => vec![0x7F, service_id, 0x11], // ServiceNotSupported
        }
    }

    fn handle_diagnostic_session_control(&mut self, payload: &[u8]) -> Vec<u8> {
        if payload.is_empty() {
            return vec![0x7F, 0x10, 0x12];
        }
        match payload[0] {
            0x01 => self.session = DiagnosticSession::Default,
            0x02 => self.session = DiagnosticSession::Programming,
            0x03 => self.session = DiagnosticSession::Extended,
            _ => return vec![0x7F, 0x10, 0x12],
        }
        vec![0x50, payload[0]]
    }

    fn handle_read_data_by_identifier(&self, payload: &[u8]) -> Vec<u8> {
        if payload.len() < 2 {
            return vec![0x7F, 0x22, 0x13]; // IncorrectMessageLengthOrInvalidFormat
        }
        let did = u16::from_be_bytes([payload[0], payload[1]]);
        match did {
            0xF190 => vec![0x62, 0xF1, 0x90, b'V', b'I', b'N', b'1', b'2', b'3'], // VIN
            _ => vec![0x7F, 0x22, 0x31], // RequestOutOfRange
        }
    }

    fn handle_read_dtcs(&self, _payload: &[u8]) -> Vec<u8> {
        // Dummy implementation
        vec![0x59, 0x02, 0xFF]
    }

    fn handle_clear_dtcs(&self, _payload: &[u8]) -> Vec<u8> {
        // Dummy implementation
        vec![0x54]
    }
}
