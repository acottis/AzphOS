#![no_std]
#![no_main]
#![feature(asm)]

mod core_reqs;
mod display;
mod cpu;
mod serial;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("{}", info);
    cpu::halt();
}

#[no_mangle]
fn entry() {
    serial_print!("{}", "Hello from rust");

    cpu::halt();
}