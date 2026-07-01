use embassy_stm32::interrupt;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use heapless::BinaryHeap;
use oxide_protocol::HostToMcu;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ScheduledEvent {
    pub timestamp: u64,
    pub channel: u8,
    pub duration: u16,
}

pub static SCHEDULER_CHANNEL: Channel<ThreadModeRawMutex, ScheduledEvent, 64> = Channel::new();
pub static mut EVENT_HEAP: BinaryHeap<ScheduledEvent, 64> = BinaryHeap::new();

#[embassy_executor::task]
pub async fn scheduler_manager_task() {
    loop {
        let event = SCHEDULER_CHANNEL.receive().await;
        unsafe {
            EVENT_HEAP.push(event).ok().unwrap();
            // TODO: Update hardware timer compare register
        }
    }
}

#[interrupt]
fn TIM2() {
    unsafe {
        if let Some(event) = EVENT_HEAP.pop() {
            // TODO: Toggle GPIO pin
            // TODO: Update hardware timer compare register with next event
        } else {
            // TODO: Disable compare interrupt
        }
    }
}
