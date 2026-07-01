use heapless::BinaryHeap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledEvent {
    pub channel: u8,
    pub timestamp_ticks: u32,
    pub duration_ticks: u16,
    pub event_type: EventType,
}

// Implement Ord so BinaryHeap works as a min-heap
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.timestamp_ticks.cmp(&self.timestamp_ticks)
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    InjectorStart,
    InjectorEnd,
    IgnitionStart,
    IgnitionEnd,
}

pub struct Scheduler<const N: usize> {
    queue: BinaryHeap<ScheduledEvent, N>,
}

impl<const N: usize> Scheduler<N> {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn schedule(&mut self, event: ScheduledEvent) -> Result<(), ScheduledEvent> {
        self.queue.push(event)
    }

    pub fn pop_ready(&mut self, current_time: u32) -> Option<ScheduledEvent> {
        if let Some(event) = self.queue.peek() {
            if event.timestamp_ticks <= current_time {
                return self.queue.pop();
            }
        }
        None
    }

    pub fn next_deadline(&self) -> Option<u32> {
        self.queue.peek().map(|e| e.timestamp_ticks)
    }
}

impl<const N: usize> Default for Scheduler<N> {
    fn default() -> Self {
        Self::new()
    }
}
