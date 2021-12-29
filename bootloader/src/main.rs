#![no_std]
#![no_main]
#![feature(asm)]

mod core_reqs;
mod display;
mod cpu;
mod serial;

#[panic_handler]
#[allow(unreachable_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("{}", info);
    cpu::halt();
}

#[no_mangle]
fn entry() {
    clear!();
    print!("{}", 69);
    print!("Helo world\n");
    let s = serial::SerialPort::init();
    print!("{:#X?}\n", s);
    s.read();
    print!("Helo world2\n");
    s.read();
    cpu::halt();
}