#![no_std]
#![no_main]
#![feature(asm)]

mod core_reqs;
//mod display;
mod cpu;
mod serial;
mod time;
mod net;
mod pci;
mod error;

use net::packet::{EtherType, Arp, Packet};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_print!("{}", info);
    cpu::halt();
}

/// This function is called by `stage0.asm` after setting up 32bit mode and a stack at 0x7c00
/// ```x86asm
/// call entry_point
/// ```
#[no_mangle]
fn entry(entry_point: u16) {
    //clear!();
    serial_print!("We entered at: {:#X}\n", entry_point);
    serial_print!("Time is: {}\n", time::DateTime::now());
    let nic = net::nic::init().expect("Cant init Network");

    loop {
        //serial_print!("{:#X?}\n", &packet);

        //nic.send(Packet::new(EtherType::Arp(Arp::new())));
        let packets = nic.receive();
        crate::time::sleep(5);
    }

    serial_print!("\nDone\n");
    cpu::halt();
}