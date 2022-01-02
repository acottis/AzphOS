//! This create is helper for any time we need to get information from the processor
//! we wrap [`asm!`] in unsafe so we can reduce the amount of unsafe randomly scattered around our code
//! Using [https://www.felixcloutier.com/x86/](https://www.felixcloutier.com/x86/) as a reference right now
//! 
use core::arch::asm;

/// Prevent the processor from rebooting by halting
#[inline]
pub fn halt() -> ! {
    loop{
        unsafe {
            asm!("cli");
            asm!("hlt");
        }
    }
}
/// [https://www.felixcloutier.com/x86/out](https://www.felixcloutier.com/x86/out)
#[inline]
pub fn out8(addr: u16, val: u8){
    unsafe{
        asm!("out dx, al", in("dx") addr, in("al") val);
    }
}
#[inline]
pub fn out32(addr: u16, val: u32){
    unsafe{
        asm!("out dx, eax", in("dx") addr, in("eax") val);
    }
}
/// [https://www.felixcloutier.com/x86/in](https://www.felixcloutier.com/x86/in)
#[inline]
pub fn in8(addr: u16) -> u8{
    let val: u8;
    unsafe{
        asm!("in al, dx", in("dx") addr, out("al") val);
    }
    val
}
#[inline]
pub fn in32(addr: u16) -> u32{
    let val: u32;
    unsafe{
        asm!("in eax, dx", in("dx") addr, out("eax") val);
    }
    val
}
/// https://wiki.osdev.org/CMOS
#[inline]
pub fn get_rtc_register(offset :u8) -> u8{
    out8(0x70, offset);
    in8(0x71)
}
#[inline]
pub fn get_esp() -> u32{
    unsafe{
        let mut x = 0;
        asm!("mov edx, esp",  out("edx") x);
        x
    }
}
