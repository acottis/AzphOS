//! This create is helper for any time we need to get information from the processor
//! we wrap [`asm!`] in unsafe so we can reduce the amount of unsafe randomly scattered around our code
//! Using [https://www.felixcloutier.com/x86/](https://www.felixcloutier.com/x86/) as a reference right now
//! 
/// Prevent the processor from rebooting by halting
#[inline]
pub fn halt() -> ! {
    unsafe {
        asm!("cli");
        asm!("hlt");
    }
    // Never hit this, needed because rust doesnt trust the above
    loop {}
}
/// [https://www.felixcloutier.com/x86/out](https://www.felixcloutier.com/x86/out)
#[inline]
pub fn out8(addr: u16, val: u8){
    unsafe{
        asm!("out dx, al", in("dx") addr, in("al") val);
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