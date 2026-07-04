//! Angular-to-time based event scheduler for RTIC.

use heapless::BinaryHeap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EventType {
    IgnitionDwellStart,
    IgnitionDwellEnd,
    InjectionStart,
    InjectionEnd,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ScheduledEvent {
    pub trigger_angle_deg: u16, // Degrees from TDC
    pub duration_ticks: u32,
    pub pin_id: u8,
    pub event_type: EventType,
}

pub struct Scheduler {
    // A min-heap to store events, ordered by their trigger angle.
    queue: BinaryHeap<ScheduledEvent, 32>,
    timer_freq: u32,
}

impl Scheduler {
    pub fn new(timer_freq: u32) -> Self {
        Self {
            queue: BinaryHeap::new(),
            timer_freq,
        }
    }

    /// Schedules an event based on an angular deadline.
    pub fn schedule_event(&mut self, event: ScheduledEvent) -> Result<(), ()> {
        self.queue.push(event).map_err(|_| ())
    }

    /// Converts the next angular event into a timer compare value in ticks.
    ///
    -/// Returns the number of ticks until the next event, and the event itself.
    pub fn get_next_timer_task(&mut self, current_angle_deg: f32, rpm: f32) -> Option<(u32, ScheduledEvent)> {
        if rpm <= 0.0 {
            return None;
        }

        if let Some(event) = self.queue.pop() {
            let mut angle_diff = event.trigger_angle_deg as f32 - current_angle_deg;
            if angle_diff < 0.0 {
                angle_diff += 720.0; // Assuming a 4-stroke cycle
            }

            // Time (s) per degree = 1 / (RPM * 6)
            let time_to_event_s = angle_diff / (rpm * 6.0);
            let ticks_to_event = (time_to_event_s * self.timer_freq as f32) as u32;

            return Some((ticks_to_event, event));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIMER_FREQ: u32 = 1_000_000; // 1MHz

    #[test]
    fn test_schedule_and_get_task() {
        let mut scheduler = Scheduler::new(TIMER_FREQ);
        let event = ScheduledEvent {
            trigger_angle_deg: 90,
            duration_ticks: 1000,
            pin_id: 0,
            event_type: EventType::IgnitionDwellStart,
        };
        scheduler.schedule_event(event).unwrap();

        let current_angle = 30.0;
        let rpm = 1000.0;

        let (ticks, scheduled_event) = scheduler.get_next_timer_task(current_angle, rpm).unwrap();

        // angle_diff = 90 - 30 = 60 degrees
        // time_to_event = 60 / (1000 * 6) = 0.01 s
        // ticks = 0.01 * 1_000_000 = 10000
        assert_eq!(ticks, 10000);
        assert_eq!(scheduled_event.trigger_angle_deg, 90);
    }

    #[test]
    fn test_angle_wraparound() {
        let mut scheduler = Scheduler::new(TIMER_FREQ);
        let event = ScheduledEvent {
            trigger_angle_deg: 20, // Next cycle
            duration_ticks: 1000,
            pin_id: 1,
            event_type: EventType::InjectionStart,
        };
        scheduler.schedule_event(event).unwrap();

        let current_angle = 700.0;
        let rpm = 2000.0;

        let (ticks, _event) = scheduler.get_next_timer_task(current_angle, rpm).unwrap();

        // angle_diff = (20 + 720) - 700 = 40 degrees
        // time_to_event = 40 / (2000 * 6) = 0.003333 s
        // ticks = 0.003333 * 1_000_000 = 3333
        assert_eq!(ticks, 3333);
    }

    #[test]
    fn test_event_ordering() {
        let mut scheduler = Scheduler::new(TIMER_FREQ);
        let event1 = ScheduledEvent { trigger_angle_deg: 180, duration_ticks: 0, pin_id: 0, event_type: EventType::IgnitionDwellEnd };
        let event2 = ScheduledEvent { trigger_angle_deg: 90, duration_ticks: 1000, pin_id: 0, event_type: EventType::IgnitionDwellStart };

        scheduler.schedule_event(event1).unwrap();
        scheduler.schedule_event(event2).unwrap();

        let (_ticks, next_event) = scheduler.get_next_timer_task(0.0, 1000.0).unwrap();
        // Should pop event2 first as it has a smaller angle
        assert_eq!(next_event.trigger_angle_deg, 90);
    }
}
