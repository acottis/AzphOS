#![no_std]
#![no_main]

mod core_reqs;
mod display;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !{
    loop {}
}

#[no_mangle]
fn entry() {

    let mut vga = display::Vga::init();
    vga.clear();
    vga.print("Hello from rustia");

    
    loop {}
}