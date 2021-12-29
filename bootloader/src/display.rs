/// This library is the my VGA implementation, this basic implementation implements `core::fmt::Write` which gives us `write_fmt` so we can
/// format args to print any values we need during debugging, the macro's provide a simple way for us to use this crate.

// This makes the text green when used as the upper bytes in a VGA buffer's u16
static GREEN: u16 = 0x0200;
// This is the VGA pointer location, currently not limited to max screen size;
static mut OFFSET: isize = 0; 

/// This struct is the main VGA logic and interacts with the VGA buffer at base 0xB8000
/// 
struct Vga;
impl Vga{
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
    pub fn clear(){
        let vga = Vga;
        vga.clear();
    }
}

impl core::fmt::Write for VgaWriter{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let vga = Vga;
        vga.write(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::display::VgaWriter, format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! clear {
    () => {
        crate::display::VgaWriter::clear();
    }
}