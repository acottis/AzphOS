#![no_std]
#![no_main]
#![feature(asm)]

mod core_reqs;
mod display;
//mod serial;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    //print!("Panic!!!!");
    //unsafe {core::ptr::write(0xB8000 as *mut u16, 0x0245);}
    loop {}
}

#[no_mangle]
fn entry() {
    
    
    print!("Hello World");
    //print!("{}", "Hello world");
    unsafe {
        asm!("cli");
        asm!("hlt");
    }
}