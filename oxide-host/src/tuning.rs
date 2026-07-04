use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use crate::{HostToMcu, McuToHost, framing};
use crate::oxide_protocol::TuningCommand;

pub enum CommError {
    SendError,
    AckTimeout,
    NakReceived,
}

pub struct TuningManager {
    mcu_tx: mpsc::Sender<HostToMcu>,
    // This would be a channel to receive acks from the serial task
    // ack_rx: mpsc::Receiver<McuToHost>,
}

impl TuningManager {
    pub fn new(mcu_tx: mpsc::Sender<HostToMcu>) -> Self {
        Self { mcu_tx }
    }

    pub async fn update_map_cell(&mut self, table_id: u8, row: u8, col: u8, new_val: f32) -> Result<(), CommError> {
        let command = TuningCommand::WriteTableValue {
            table_id,
            row,
            col,
            value: new_val,
        };
        let message = HostToMcu::Tuning(command);

        self.mcu_tx.send(message).await.map_err(|_| CommError::SendError)?;

        // In a real implementation, we would wait for an ack here.
        // For this mock, we'll just assume it works.
        // let ack = tokio::time::timeout(Duration::from_millis(100), self.ack_rx.recv()).await;
        // match ack {
        //     Ok(Some(McuToHost::TuningAck)) => Ok(()),
        //     Ok(Some(McuToHost::TuningNak)) => Err(CommError::NakReceived),
        //     _ => Err(CommError::AckTimeout),
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_update_map_cell() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut manager = TuningManager::new(tx);

        let table_id = 0;
        let row = 1;
        let col = 2;
        let value = 123.45;

        manager.update_map_cell(table_id, row, col, value).await.unwrap();

        let received_msg = rx.recv().await.unwrap();
        match received_msg {
            HostToMcu::Tuning(TuningCommand::WriteTableValue { table_id: tid, row: r, col: c, value: v }) => {
                assert_eq!(tid, table_id);
                assert_eq!(r, row);
                assert_eq!(c, col);
                assert_eq!(v, value);
            }
            _ => panic!("Incorrect message type received"),
        }
    }
}
