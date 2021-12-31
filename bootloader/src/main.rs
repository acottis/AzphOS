#![no_std]
#![no_main]
#![feature(asm)]

mod core_reqs;
mod display;
mod cpu;
mod serial;
mod time;
mod net;


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("{}", info);
    cpu::halt();
}

/// This function is called by `stage0.asm` after setting up 32bit mode and a stack at 0x7c00
/// ```x86asm
/// call entry_point
/// ```
#[no_mangle]
fn entry(entry_point: u16) {
    clear!();
    print!("We entered at: {:#X}\n", entry_point);

    // let dt = time::DateTime::now();
    // print!("Time is: {}\n", dt);

    net::init();

    print!("Done\n");
    //serial_print!("{}", "Hello from rust");

    cpu::halt();
}