#![doc = include_str!("../../README.md")]
#![no_std]
#![no_main]
#![allow(rustdoc::bare_urls)]
// #![deny(rustdoc::all)]

#[macro_use]
mod serial;

mod core_reqs;
// mod display;
mod cpu;
mod error;
mod net;
mod pci;
mod time;

/// Custom panic handler for our OS
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	print!("{}", info);
	cpu::halt();
}

/// This function is called by `stage0.asm` after setting up 32bit mode and a
/// stack at ~~0x7c00~~ 0x2000000
/// ```x86asm
/// call entry_point
/// ```
#[no_mangle]
fn entry(entry_point: u64) {
	//clear!();
	print!("We entered at: {:#X}\n", entry_point);
	print!("Time is: {}\n", time::DateTime::now());

	// Try to initialise network, dont continue if we fail
	let mut net = net::NetworkStack::init().unwrap();

	// Main OS loop
	loop {
		net.update();
	}
}
