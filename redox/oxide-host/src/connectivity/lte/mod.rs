use atcommands::at;
use ed25519_dalek::{Signature, VerifyingKey};
use std::io;

pub struct LteModem<T: io::Read + io::Write> {
    transport: T,
}

impl<T: io::Read + io.Write> LteModem<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn send_at_command(&mut self, cmd: &str) -> Result<String, atcommands::Error> {
        let command = at::Command::new(at::AT::new(cmd));
        at::send(&mut self.transport, &command)?;
        let response: at::Response = at::recv(&mut self.transport)?;
        Ok(response.to_string())
    }
}

pub struct TrimUpdate {
    pub table_id: u8,
    pub x: f32,
    pub y: f32,
    pub delta: f32,
    pub signature: Signature,
}

pub struct RemoteTuning {
    public_key: VerifyingKey,
}

impl RemoteTuning {
    pub fn new(public_key: VerifyingKey) -> Self {
        Self { public_key }
    }

    pub fn apply_trim(&self, update: &TrimUpdate) -> Result<(), &'static str> {
        let mut message = Vec::new();
        message.extend_from_slice(&update.table_id.to_le_bytes());
        message.extend_from_slice(&update.x.to_le_bytes());
        message.extend_from_slice(&update.y.to_le_bytes());
        message.extend_from_slice(&update.delta.to_le_bytes());

        if self.public_key.verify_strict(&message, &update.signature).is_ok() {
            // Apply the trim to the live 3D table via IPC Unix stream
            if let Ok(mut stream) = std::os::unix::net::UnixStream::connect("/tmp/oxide_tuning.sock") {
                let _ = std::io::Write::write_all(&mut stream, &message);
            }

            Ok(())
        } else {
            Err("Invalid signature")
        }
    }
}
