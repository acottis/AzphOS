#![no_std]
#![no_main]

mod core_reqs;
mod display;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !{
    print!("Panic!!!!");
    loop {}
}

#[no_mangle]
fn entry() {

    print!("Hello World");
    //print!("{}", 10);
    loop {}
}

