#[inline]
pub fn halt() -> ! {
    unsafe {
        asm!("cli");
        asm!("hlt");
    }
    // Never hit this
    loop {}
}

#[inline]
pub fn out8(addr: u16, val: u8){
    unsafe{
        asm!("out dx, al", in("dx") addr, in("al") val);
    }
}
#[inline]
pub fn in8(addr: u16) -> u8{
    let val: u8;
    unsafe{
        asm!("in al, dx", in("dx") addr, out("al") val);
    }
    val
}