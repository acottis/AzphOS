#![doc = include_str!("../../README.md")]

#![no_std]
#![no_main]
#![allow(rustdoc::bare_urls)]
// #![deny(rustdoc::all)]

mod core_reqs;
// mod display;
mod cpu;
mod error;
mod net;
mod pci;
mod serial;
mod time;

/// Custom panic handler for our OS
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_print!("{}", info);
    cpu::halt();
}

/// This function is called by `stage0.asm` after setting up 32bit mode and a
/// stack at ~~0x7c00~~ 0x2000000
/// ```x86asm
/// call entry_point
/// ```
#[no_mangle]
fn entry(entry_point: u64) {
    //clear!();
    serial_print!("We entered at: {:#X}\n", entry_point);
    serial_print!("Time is: {}\n", time::DateTime::now());

    // Try to initialise network, dont continue if we fail
    let mut net = net::NetworkStack::init().unwrap();

    // Main OS loop
    loop {
        net.update();
    }
}
