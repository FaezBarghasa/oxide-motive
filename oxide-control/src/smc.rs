
pub struct SlidingModeController {
    lambda: f32,
    phi: f32,
    k: f32,
}

impl SlidingModeController {
    pub fn new(lambda: f32, phi: f32, k: f32) -> Self {
        Self { lambda, phi, k }
    }

    pub fn update(&mut self, error: f32, error_dot: f32) -> f32 {
        let s = error_dot + self.lambda * error;
        let sat_s = s.max(-self.phi).min(self.phi) / self.phi;
        -self.k * sat_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smc() {
        let mut smc = SlidingModeController::new(1.0, 0.1, 10.0);
        let output = smc.update(1.0, 0.5);
        assert_ne!(output, 0.0);
    }
}
