
use std::fs;
use std::path::Path;

pub struct DeviceTreeParser;

impl DeviceTreeParser {
    pub fn get_model() -> Option<String> {
        let model_path = Path::new("/sys/firmware/devicetree/base/model");
        if model_path.exists() {
            fs::read_to_string(model_path).ok()
        } else {
            None
        }
    }
}
