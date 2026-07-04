use heapless::BinaryHeap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EventType {
    Ignition,
    Injection,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ScheduledEvent {
    pub angle: u16,
    pub event_type: EventType,
    pub channel: u8,
    pub duration: u16,
}

pub struct AngularScheduler {
    queue: BinaryHeap<ScheduledEvent, 64>,
}

impl AngularScheduler {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn schedule(&mut self, event: ScheduledEvent) -> Result<(), ()> {
        self.queue.push(event).map_err(|_| ())
    }

    pub fn pop_ready(&mut self, current_angle: u16) -> Option<ScheduledEvent> {
        if let Some(event) = self.queue.peek() {
            if event.angle <= current_angle {
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
    fn test_angular_scheduler() {
        let mut scheduler = AngularScheduler::new();
        scheduler.schedule(ScheduledEvent {
            angle: 180,
            event_type: EventType::Ignition,
            channel: 0,
            duration: 100,
        }).unwrap();
        scheduler.schedule(ScheduledEvent {
            angle: 360,
            event_type: EventType::Injection,
            channel: 1,
            duration: 200,
        }).unwrap();

        assert_eq!(scheduler.pop_ready(100), None);
        let event = scheduler.pop_ready(200).unwrap();
        assert_eq!(event.angle, 180);
        assert_eq!(scheduler.pop_ready(400).unwrap().angle, 360);
        assert_eq!(scheduler.pop_ready(400), None);
    }
}
