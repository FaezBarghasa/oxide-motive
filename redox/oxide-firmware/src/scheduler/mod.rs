use embassy_stm32::interrupt;
use embassy_stm32::pac;
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
            if let Some(next_event) = EVENT_HEAP.peek() {
                pac::TIM2.ccr(0).write_value(next_event.timestamp as u32);
                pac::TIM2.dier().modify(|w| w.set_cc1ie(true));
            }
        }
    }
}

#[interrupt]
fn TIM2() {
    unsafe {
        pac::TIM2.sr().modify(|w| w.set_cc1if(false));
        
        if let Some(event) = EVENT_HEAP.pop() {
            let pin = event.channel as usize;
            let current = pac::GPIOA.odr().read().odr(pin);
            
            if current {
                pac::GPIOA.bsrr().write(|w| w.set_br(pin, true));
            } else {
                pac::GPIOA.bsrr().write(|w| w.set_bs(pin, true));
            }
            
            if let Some(next_event) = EVENT_HEAP.peek() {
                pac::TIM2.ccr(0).write_value(next_event.timestamp as u32);
            } else {
                pac::TIM2.dier().modify(|w| w.set_cc1ie(false));
            }
        } else {
            pac::TIM2.dier().modify(|w| w.set_cc1ie(false));
        }
    }
}
