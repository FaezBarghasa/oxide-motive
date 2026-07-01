#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32h7xx_hal::pac;
use panic_halt as _;

const BANK_A_ADDR: u32 = 0x08020000;
const BANK_B_ADDR: u32 = 0x08040000;
const MAGIC_ADDR: *mut u32 = 0x40002850 as *mut u32; // RTC backup register 0

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();
    let rcc = dp.RCC.constrain();
    let _ccdr = rcc.sys_ck(400.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let magic = unsafe { core::ptr::read_volatile(MAGIC_ADDR) };

    let boot_addr = if magic == 0xDEADBEEF {
        // Boot Bank B
        unsafe { core::ptr::write_volatile(MAGIC_ADDR, 0) };
        BANK_B_ADDR
    } else {
        // Boot Bank A
        BANK_A_ADDR
    };

    // In a real scenario, we would verify the signature of the application before jumping.

    unsafe {
        let sp = *(boot_addr as *const u32);
        let reset_handler = *((boot_addr + 4) as *const u32);
        let reset_fn: extern "C" fn() -> ! = core::mem::transmute(reset_handler);
        cortex_m::register::msp::write(sp);
        reset_fn();
    }
}
