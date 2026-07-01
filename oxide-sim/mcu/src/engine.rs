
// A simple 4-cylinder engine physics model
pub struct Engine {
    pub rpm: u16,
    pub map: u16, // Manifold Absolute Pressure in kPa
    pub tps: u16, // Throttle Position Sensor in %
    pub iat: i16, // Intake Air Temperature in C
    pub ect: i16, // Engine Coolant Temperature in C
}

impl Engine {
    pub fn new() -> Self {
        Self {
            rpm: 0,
            map: 100,
            tps: 0,
            iat: 25,
            ect: 25,
        }
    }

    pub fn step(&mut self, throttle_position: f32, load: f32) {
        // Very basic simulation
        self.tps = (throttle_position * 100.0) as u16;
        let air_mass = self.tps as f32 * 0.8 + load * 0.2;
        self.rpm = (air_mass * 80.0) as u16;
        self.map = 100 + (self.rpm / 200) as u16;

        if self.ect < 90 {
            self.ect += 1;
        }
    }
}
