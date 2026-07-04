
use std::fs;
use anyhow::Result;

pub struct DiscoveryManager;

impl DiscoveryManager {
    pub fn discover_uart() -> Result<String> {
        for entry in fs::read_dir("/sys/class/tty")? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("ttyACM") || name.starts_with("ttyUSB") {
                return Ok(format!("/dev/{}", name));
            }
        }
        Err(anyhow::anyhow!("No suitable UART port found"))
    }

    pub fn discover_network_interface() -> Result<String> {
        for entry in fs::read_dir("/sys/class/net")? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("eth") || name.starts_with("wlan") {
                return Ok(name);
            }
        }
        Err(anyhow::anyhow!("No suitable network interface found"))
    }
}
