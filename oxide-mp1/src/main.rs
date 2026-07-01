#![no_std]
#![no_main]

#[cfg(feature = "m4")]
mod m4_app;

#[cfg(feature = "a7")]
mod a7_app;

#[cfg(feature = "m4")]
#[entry]
fn main() -> ! {
    m4_app::main()
}

#[cfg(feature = "a7")]
fn main() {
    a7_app::main();
}

#[cfg(not(any(feature = "m4", feature = "a7")))]
compile_error!("Either 'm4' or 'a7' feature must be enabled for oxide-mp1");

#[cfg(feature = "m4")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
