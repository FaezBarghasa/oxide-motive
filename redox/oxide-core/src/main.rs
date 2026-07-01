use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EngineState {
    Off,
    Cranking,
    Running,
    Limp,
    OvertempProtection,
    OverboostProtection,
    Shutdown,
}

pub struct StateMachine {
    pub state: EngineState,
    last_transition_time: Instant,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: EngineState::Off,
            last_transition_time: Instant::now(),
        }
    }

    pub fn update(
        &mut self,
        rpm: u32,
        tps: f32,
        coolant_temp: f32,
        _oil_pressure: f32,
        host_link_alive: bool,
    ) {
        let new_state = match self.state {
            EngineState::Off => {
                if rpm > 150 && host_link_alive {
                    EngineState::Cranking
                } else {
                    self.state
                }
            }
            EngineState::Cranking => {
                if rpm > 400 && tps > 2.0 {
                    EngineState::Running
                } else if !host_link_alive {
                    EngineState::Limp
                } else {
                    self.state
                }
            }
            EngineState::Running => {
                if coolant_temp > 115.0 {
                    EngineState::OvertempProtection
                } else if !host_link_alive {
                    EngineState::Limp
                } else if rpm < 50 && tps == 0.0 && self.last_transition_time.elapsed() > Duration::from_secs(5) {
                    EngineState::Shutdown
                } else {
                    self.state
                }
            }
            EngineState::OvertempProtection => {
                if coolant_temp < 110.0 {
                    EngineState::Running
                } else {
                    self.state
                }
            }
            EngineState::Limp => {
                if host_link_alive {
                    EngineState::Running
                } else {
                    self.state
                }
            }
            _ => self.state,
        };

        if new_state != self.state {
            self.state = new_state;
            self.last_transition_time = Instant::now();
        }
    }
}

fn main() {
    println!("oxide-core started");
    let mut sm = StateMachine::new();
    sm.update(200, 0.0, 80.0, 60.0, true);
    println!("Current state: {:?}", sm.state);
}
