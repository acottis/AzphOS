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
    use core::fmt::Write;
    let mut vga = display::Vga::init();
    vga.clear();
    vga.write_str("Hello from rustia").unwrap();
    //write!(&mut vga, "Test");

    
    loop {}
}