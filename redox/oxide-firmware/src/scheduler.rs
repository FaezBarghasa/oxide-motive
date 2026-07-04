use heapless::BinaryHeap;

const TIMER_FREQ: f32 = 100_000_000.0; // 100MHz

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventType {
    Ignition,
    Injection,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScheduledEvent {
    pub trigger_tick: u32,
    pub duration_ticks: u32,
    pub pin_id: u8,
    pub event_type: EventType,
}

// Implement Ord and PartialOrd manually to make it a min-heap
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.trigger_tick.cmp(&self.trigger_tick)
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Scheduler {
    queue: BinaryHeap<ScheduledEvent, 32>,
}

#[derive(Debug)]
pub enum SchedulerError {
    QueueFull,
    ScheduleInPast,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn schedule(
        &mut self,
        target_angle: f32,
        current_angle: f32,
        rpm: f32,
        duration_us: f32,
        pin: u8,
        event: EventType,
        current_tick: u32,
    ) -> Result<(), SchedulerError> {
        let ticks_per_rev = (TIMER_FREQ * 60.0) / rpm;
        let ticks_per_degree = ticks_per_rev / 360.0;
        let angle_diff = (target_angle - current_angle + 360.0) % 360.0;
        let trigger_tick = current_tick.wrapping_add((angle_diff * ticks_per_degree) as u32);

        if trigger_tick < current_tick {
            return Err(SchedulerError::ScheduleInPast);
        }

        let duration_ticks = (duration_us * (TIMER_FREQ / 1_000_000.0)) as u32;

        let scheduled_event = ScheduledEvent {
            trigger_tick,
            duration_ticks,
            pin_id: pin,
            event_type: event,
        };

        self.queue.push(scheduled_event).map_err(|_| SchedulerError::QueueFull)
    }

    pub fn pop_ready(&mut self, current_tick: u32) -> Option<ScheduledEvent> {
        if let Some(event) = self.queue.peek() {
            if event.trigger_tick <= current_tick {
                return self.queue.pop();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_and_pop() {
        let mut scheduler = Scheduler::new();
        let current_tick = 1000;

        scheduler.schedule(90.0, 0.0, 6000.0, 1000.0, 1, EventType::Ignition, current_tick).unwrap();
        scheduler.schedule(180.0, 0.0, 6000.0, 1000.0, 2, EventType::Injection, current_tick).unwrap();

        let event1 = scheduler.pop_ready(current_tick + 25000).unwrap();
        assert_eq!(event1.pin_id, 1);

        let event2 = scheduler.pop_ready(current_tick + 50000).unwrap();
        assert_eq!(event2.pin_id, 2);
    }

    #[test]
    fn test_schedule_in_past() {
        let mut scheduler = Scheduler::new();
        let result = scheduler.schedule(0.0, 90.0, 6000.0, 1000.0, 1, EventType::Ignition, 1000);
        assert!(matches!(result, Err(SchedulerError::ScheduleInPast)));
    }
}
