#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !{
    loop {}
}

#[no_mangle]
fn entry() {
    loop {}
}
