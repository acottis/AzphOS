#![no_std]
#![no_main]

mod core_reqs;
//mod display;
mod cpu;
mod serial;
mod time;
mod net;
mod pci;
mod error;

// #[cfg(not(test))] is a fix for a Rust analyzer bug
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_print!("{}", info);
    cpu::halt();
}

/// This function is called by `stage0.asm` after setting up 32bit mode and a stack at 0x7c00
/// ```x86asm
/// call entry_point
/// ```
#[no_mangle]
fn entry(entry_point: u16) {
    //clear!();
    serial_print!("We entered at: {:#X}\n", entry_point);
    serial_print!("Time is: {}\n", time::DateTime::now());

    // Try to initialise network, dont continue if we fail
    let net = net::NetworkStack::init().unwrap();
    net.update();
    net.update();

    cpu::halt();
}