#![cfg_attr(not(feature = "std"), no_std)]

use heapless::Vec;
use embedded_io_async::{Read, Write};
use embassy_time::{Duration, Timer, Instant};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub enum MqttError {
    Network,
    Encode,
    Decode,
    BufferTooSmall,
    NotConnected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PacketState {
    Start,
    WaitPubrec,
    SendPubrel,
    WaitPubcomp,
    Complete,
}

struct QoS2State {
    packet_id: u16,
    state: PacketState,
    retries: u8,
}

pub struct MqttClient<'a, R, W>
where
    R: Read,
    W: Write,
{
    reader: R,
    writer: W,
    qos2_states: Mutex<CriticalSectionRawMutex, Vec<QoS2State, 16>>,
    will: Option<Will<'a>>,
    last_activity: Signal<CriticalSectionRawMutex, Instant>,
}

#[derive(Debug, Clone)]
pub struct Will<'a> {
    pub topic: &'a str,
    pub message: &'a [u8],
    pub qos: QoS,
    pub retain: bool,
}

impl<'a, R, W> MqttClient<'a, R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(reader: R, writer: W, will: Option<Will<'a>>) -> Self {
        Self {
            reader,
            writer,
            qos2_states: Mutex::new(Vec::new()),
            will,
            last_activity: Signal::new(),
        }
    }

    pub async fn connect(&mut self, client_id: &str) -> Result<(), MqttError> {
        let mut packet = Vec::<u8, 256>::new();
        packet.extend_from_slice(&[0x10]).unwrap();
        let mut remaining_length = 10 + client_id.len() as u32;

        let mut connect_flags = 0x02; // Clean session
        if let Some(will) = &self.will {
            connect_flags |= 0x04; // Will flag
            connect_flags |= (will.qos as u8) << 3;
            if will.retain {
                connect_flags |= 0x20;
            }
            remaining_length += 2 + will.topic.len() as u32 + 2 + will.message.len() as u32;
        }

        let mut remaining_length_bytes = Vec::<u8, 4>::new();
        let mut x = remaining_length;
        loop {
            let mut encoded_byte = (x % 128) as u8;
            x /= 128;
            if x > 0 {
                encoded_byte |= 128;
            }
            remaining_length_bytes.push(encoded_byte).unwrap();
            if x == 0 {
                break;
            }
        }

        packet.extend_from_slice(&remaining_length_bytes).unwrap();
        packet.extend_from_slice(&[0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04]).unwrap();
        packet.push(connect_flags).unwrap();
        packet.extend_from_slice(&[0x00, 0x3C]).unwrap(); // Keep alive 60s
        packet.extend_from_slice(&(client_id.len() as u16).to_be_bytes()).unwrap();
        packet.extend_from_slice(client_id.as_bytes()).unwrap();

        if let Some(will) = &self.will {
            packet.extend_from_slice(&(will.topic.len() as u16).to_be_bytes()).unwrap();
            packet.extend_from_slice(will.topic.as_bytes()).unwrap();
            packet.extend_from_slice(&(will.message.len() as u16).to_be_bytes()).unwrap();
            packet.extend_from_slice(will.message).unwrap();
        }

        self.writer.write_all(&packet).await.map_err(|_| MqttError::Network)?;
        self.last_activity.signal(Instant::now());
        Ok(())
    }

    async fn send_packet(&mut self, packet: &[u8]) -> Result<(), MqttError> {
        self.writer.write_all(packet).await.map_err(|_| MqttError::Network)?;
        self.last_activity.signal(Instant::now());
        Ok(())
    }

    pub async fn publish(&mut self, topic: &str, payload: &[u8], qos: QoS) -> Result<(), MqttError> {
        let packet_id = 123;
        if qos == QoS::ExactlyOnce {
            let mut states = self.qos2_states.lock().await;
            states.push(QoS2State {
                packet_id,
                state: PacketState::Start,
                retries: 0,
            }).map_err(|_| MqttError::BufferTooSmall)?;
        }

        self.send_publish(topic, payload, qos, packet_id).await?;

        if qos == QoS::ExactlyOnce {
            self.run_qos2_sm(packet_id).await?;
        }

        Ok(())
    }

    async fn send_publish(&mut self, topic: &str, payload: &[u8], qos: QoS, packet_id: u16) -> Result<(), MqttError> {
        let mut packet = Vec::<u8, 1024>::new();
        let mut remaining_length = 2 + topic.len() as u32 + payload.len() as u32;
        if qos as u8 > 0 {
            remaining_length += 2;
        }
        let mut packet_type = 0x30;
        packet_type |= (qos as u8) << 1;
        packet.push(packet_type).unwrap();
        let mut remaining_length_bytes = Vec::<u8, 4>::new();
        let mut x = remaining_length;
        loop {
            let mut encoded_byte = (x % 128) as u8;
            x /= 128;
            if x > 0 {
                encoded_byte |= 128;
            }
            remaining_length_bytes.push(encoded_byte).unwrap();
            if x == 0 {
                break;
            }
        }
        packet.extend_from_slice(&remaining_length_bytes).unwrap();
        packet.extend_from_slice(&(topic.len() as u16).to_be_bytes()).unwrap();
        packet.extend_from_slice(topic.as_bytes()).unwrap();
        if qos as u8 > 0 {
            packet.extend_from_slice(&packet_id.to_be_bytes()).unwrap();
        }
        packet.extend_from_slice(payload).unwrap();

        self.send_packet(&packet).await
    }

    async fn run_qos2_sm(&mut self, packet_id: u16) -> Result<(), MqttError> {
        loop {
            let mut states = self.qos2_states.lock().await;
            let state = states.iter_mut().find(|s| s.packet_id == packet_id).unwrap();

            match state.state {
                PacketState::Start => {
                    state.state = PacketState::WaitPubrec;
                    drop(states);
                    self.wait_for_pubrec(packet_id).await?;
                }
                PacketState::WaitPubrec => {}
                PacketState::SendPubrel => {
                    self.send_pubrel(packet_id).await?;
                    state.state = PacketState::WaitPubcomp;
                    drop(states);
                    self.wait_for_pubcomp(packet_id).await?;
                }
                PacketState::WaitPubcomp => {}
                PacketState::Complete => {
                    states.retain(|s| s.packet_id != packet_id);
                    return Ok(());
                }
            }
        }
    }

    async fn wait_for_pubrec(&mut self, packet_id: u16) -> Result<(), MqttError> {
        let timeout = Duration::from_secs(5);
        let mut buf = [0u8; 4];

        loop {
            match embassy_time::with_timeout(timeout, self.reader.read_exact(&mut buf)).await {
                Ok(Ok(_)) => {
                    self.last_activity.signal(Instant::now());
                    if buf[0] == 0x50 && u16::from_be_bytes([buf[2], buf[3]]) == packet_id {
                        let mut states = self.qos2_states.lock().await;
                        let state = states.iter_mut().find(|s| s.packet_id == packet_id).unwrap();
                        state.state = PacketState::SendPubrel;
                        return Ok(());
                    }
                }
                _ => {
                    let mut states = self.qos2_states.lock().await;
                    let state = states.iter_mut().find(|s| s.packet_id == packet_id).unwrap();
                    if state.retries < 3 {
                        state.retries += 1;
                        return Err(MqttError::Network);
                    } else {
                        return Err(MqttError::Network);
                    }
                }
            }
        }
    }

    async fn send_pubrel(&mut self, packet_id: u16) -> Result<(), MqttError> {
        let packet: [u8; 4] = [0x62, 0x02, (packet_id >> 8) as u8, packet_id as u8];
        self.send_packet(&packet).await
    }

    async fn wait_for_pubcomp(&mut self, packet_id: u16) -> Result<(), MqttError> {
        let timeout = Duration::from_secs(5);
        let mut buf = [0u8; 4];

        loop {
            match embassy_time::with_timeout(timeout, self.reader.read_exact(&mut buf)).await {
                Ok(Ok(_)) => {
                    self.last_activity.signal(Instant::now());
                    if buf[0] == 0x70 && u16::from_be_bytes([buf[2], buf[3]]) == packet_id {
                        let mut states = self.qos2_states.lock().await;
                        let state = states.iter_mut().find(|s| s.packet_id == packet_id).unwrap();
                        state.state = PacketState::Complete;
                        return Ok(());
                    }
                }
                _ => {
                    let mut states = self.qos2_states.lock().await;
                    let state = states.iter_mut().find(|s| s.packet_id == packet_id).unwrap();
                    if state.retries < 3 {
                        state.retries += 1;
                        self.send_pubrel(packet_id).await?;
                    } else {
                        return Err(MqttError::Network);
                    }
                }
            }
        }
    }

    pub async fn keep_alive_task(&mut self) {
        let keep_alive_interval = Duration::from_secs(60);
        let mut backoff = 1;
        loop {
            let last_activity = self.last_activity.wait().await;
            let next_ping = last_activity + keep_alive_interval;
            Timer::at(next_ping).await;

            let pingreq: [u8; 2] = [0xC0, 0x00];
            if self.send_packet(&pingreq).await.is_err() {
                Timer::after(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(60);
            } else {
                backoff = 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_io_async::Read;
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;

    struct MockReader {
        data: Vec<u8, 1024>,
        pos: usize,
    }

    impl Read for MockReader {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, embedded_io_async::ReadExactError<()>> {
            let bytes_to_read = core::cmp::min(buf.len(), self.data.len() - self.pos);
            buf[..bytes_to_read].copy_from_slice(&self.data[self.pos..self.pos + bytes_to_read]);
            self.pos += bytes_to_read;
            Ok(bytes_to_read)
        }
    }

    struct MockWriter {
        data: Vec<u8, 1024>,
    }

    impl Write for MockWriter {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
            self.data.extend_from_slice(buf).unwrap();
            Ok(buf.len())
        }
    }

    #[test]
    fn test_connect_with_will() {
        let will = Will {
            topic: "status",
            message: b"offline",
            qos: QoS::AtLeastOnce,
            retain: true,
        };
        let mut client = MqttClient::new(MockReader{data: Vec::new(), pos: 0}, MockWriter{data: Vec::new()}, Some(will));

        // This requires an async test runner
    }
}