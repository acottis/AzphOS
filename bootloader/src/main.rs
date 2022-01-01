#![no_std]
#![no_main]
#![feature(asm)]

mod core_reqs;
mod display;
mod cpu;
mod serial;
mod time;
mod pci;
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

    let dt = time::DateTime::now();
    print!("Time is: {}\n", dt);

    let devices = pci::PciDevices::init();
    crate::serial_print!("{:#X?}", devices);

    print!("Done\n");

    cpu::halt();
}