//! This create is helper for any time we need to get information from the
//! processor we wrap [`asm!`] in unsafe so we can reduce the amount of unsafe
//! randomly scattered around our code Using [https://www.felixcloutier.com/x86/](https://www.felixcloutier.com/x86/) as a reference right now
#![allow(dead_code)]
use core::arch::asm;
/// Prevent the processor from rebooting by halting
#[inline]
pub fn halt() -> ! {
	loop {
		unsafe {
			asm!("cli");
			asm!("hlt");
		}
	}
}
/// [https://www.felixcloutier.com/x86/out](https://www.felixcloutier.com/x86/out)
#[inline]
pub fn out8(addr: u16, val: u8) {
	unsafe {
		asm!("out dx, al", in("dx") addr, in("al") val);
	}
}
#[inline]
pub fn out32(addr: u16, val: u32) {
	unsafe {
		asm!("out dx, eax", in("dx") addr, in("eax") val);
	}
}
/// [https://www.felixcloutier.com/x86/in](https://www.felixcloutier.com/x86/in)
#[inline]
pub fn in8(addr: u16) -> u8 {
	let val: u8;
	unsafe {
		asm!("in al, dx", in("dx") addr, out("al") val);
	}
	val
}
#[inline]
pub fn in32(addr: u16) -> u32 {
	let val: u32;
	unsafe {
		asm!("in eax, dx", in("dx") addr, out("eax") val);
	}
	val
}
/// `<https://wiki.osdev.org/CMOS>`
#[inline]
pub fn rtc_register(offset: u8) -> u8 {
	out8(0x70, offset);
	in8(0x71)
}
/// Get the current stack pointer
#[inline]
#[allow(unused_assignments)]
pub fn esp() -> u32 {
	unsafe {
		let mut x = 0;
		asm!("mov edx, esp",  out("edx") x);
		x
	}
}
// /// Does not work well on 32 bit, printing hangs on 64 bits
// #[inline]
// pub fn rdtsc() -> u64 {
//     let mut h: u32 = 0;
//     let mut l: u32 = 0;
//     unsafe {
//         asm!(
//             "rdtsc",
//             out("edx") h,
//             out("eax") l
//         )
//     }
//     // Hacky work around for 32 bit
//     (h  as u64) << 32 | l as u64
// }
// /// Sleep for the specified cycle count
// pub fn sleep(cycles: u64){
//     let start = rdtsc();
//     while rdtsc() < (start + cycles) {}
// }
