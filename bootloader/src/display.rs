//! This library is the my VGA implementation, this basic implementation implements [`core::fmt::Write`] which gives us 
//! [`core::fmt::Write::write_fmt`] so we can format args to print any values we need during debugging, 
//! the macro's provide a simple way for us to use this crate.

/// This makes the text green when used as the upper bytes in a VGA buffer's u16
static GREEN: u16 = 0x0200;
/// This is the VGA pointer location, currently not limited to max screen size;
static mut OFFSET: isize = 0; 

/// This struct is the main VGA logic and interacts with the VGA buffer at base 0xB8000 with an offset
/// Handles `\n` as a carridge return
/// 
struct Vga;
impl Vga{
    /// Writes to the VGA buffer and handles new lines
    /// 
    fn write(&self, bytes: &[u8]){
        let buffer_base = 0xB8000 as *mut u16;
        unsafe { 
            // This prevents the VGA buffer leaving the screen.
            if OFFSET > 0xFA0 { OFFSET = 0; }
            for byte in bytes{
                // Newline `\n` will act as a CRLF in my OS, 0x50 is 80 decimal, the VGA width
                if *byte == '\n' as u8 {
                    let tmp = OFFSET % 0x50;
                    OFFSET += 0x50-tmp;
                    continue;
                }
                *buffer_base.offset(OFFSET) = GREEN + (*byte as u16);       
                OFFSET += 1;
            }
        }   
    }
    /// Clears the screen by writing from `0xB8000..0xB8FA0` with `0x0000`
    /// 
    fn clear(&self){
        for chr in (0xB8000..0xB8FA0).step_by(2) {
            unsafe{ core::ptr::write(chr as *mut u16, 0x0000 ); }
        }
    }
}
/// This struct is the main VGA logic and interacts with the VGA buffer at base 0xB8000
/// 
pub struct VgaWriter;
impl VgaWriter{
    /// calls the ['Vga::clear']
    pub fn clear(){
        let vga = Vga;
        vga.clear();
    }
}
impl core::fmt::Write for VgaWriter{
    /// Our trait implementation of [`core::fmt::Write`]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let vga = Vga;
        vga.write(s.as_bytes());
        Ok(())
    }
}

/// Our core implementation of [`std::print!`](https://doc.rust-lang.org/std/macro.print.html)
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::display::VgaWriter, format_args!($($arg)*));
    }
}
/// This is a convient way to handle clearing the screen without having to get an instance of [`Vga`] or [`VgaWriter`] in 
/// our other logic, this calls [`Vga::clear`]
#[macro_export]
macro_rules! clear {
    () => {
        crate::display::VgaWriter::clear();
    }
}