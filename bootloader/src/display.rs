/// This the static VGA buffer location on an x86
/// 
// static VGA_BUFFER: u32 = 0xB8000;
// static VGA_BUFFER_MAX: u32 = 4000;
// static GREEN: u16 = 0x0200;

struct Vga;
impl Vga{
    fn write(&self, bytes: &[u8]){
        let buffer_base = 0xB8000 as *mut u16;
        let mut offset = 0;
        unsafe { 
            for byte in bytes{
                *buffer_base.offset(offset) = 0x0200 + (*byte as u16);       
                offset += 1;
            }
        }   
    }

    fn clear(&self){
        for chr in (0xB8000..0xB8FFF).step_by(2) {
            unsafe{ core::ptr::write(chr as *mut u16, 0x0000 ); }
        }
    }
}
pub struct VgaWriter;
//static mut offset: isize = 0; 

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