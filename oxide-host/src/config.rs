use std::collections::HashSet;
use std::path::Path;
use oxide_protocol::{HostToMcu, McuConnection};
use oxide_math::Table3D;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TableEditor {
    table_id: u8,
    table: Table3D<16, 16>,
    modified_cells: HashSet<(u8, u8)>,
}

impl TableEditor {
    pub fn new(table_id: u8) -> Self {
        Self {
            table_id,
            table: Table3D::new(),
            modified_cells: HashSet::new(),
        }
    }

    pub async fn load_from_mcu(&mut self, _connection: &McuConnection) -> Result<(), ()> {
        // In a real scenario, we'd request the table from the MCU
        Ok(())
    }

    pub fn update_cell(&mut self, x: u8, y: u8, value: f32) {
        self.table.set(x, y, value);
        self.modified_cells.insert((x, y));
    }

    pub async fn save_to_mcu(&self, connection: &mut McuConnection) -> Result<(), ()> {
        for (x, y) in &self.modified_cells {
            let value = self.table.get(*x as f32, *y as f32);
            let msg = HostToMcu::TableUpdate {
                table_id: self.table_id,
                x_idx: *x,
                y_idx: *y,
                value,
            };
            let mut buf = [0u8; 64];
            let len = oxide_protocol::framing::encode_frame(&msg, &mut buf, 0).unwrap();
            connection.stream.write_all(&buf[..len]).await.map_err(|_| ())?;
        }
        Ok(())
    }

    pub fn export_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)
    }

    pub fn import_from_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        let editor: TableEditor = serde_json::from_str(&data)?;
        self.table = editor.table;
        self.modified_cells = editor.modified_cells;
        Ok(())
    }
}

pub struct ConfigManager {
    // ECU configuration fields
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send_config(&self, _connection: &McuConnection) -> Result<(), ()> {
        // Send config update to MCU
        Ok(())
    }
}
