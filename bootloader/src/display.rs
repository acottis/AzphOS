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
            //*buffer_base.offset(offset) = 0x0245;
            for byte in bytes{
                *buffer_base.offset(offset) = 0x0200 + (*byte as u16);
                
                offset += 1;
            }
        }   
    }

    pub fn clear(&self){
        for chr in (0xB8000..0xB8FFF).step_by(2) {
            unsafe{ core::ptr::write(chr as *mut u16, 0x0000 ); }
        }
    }
}
pub struct VgaWriter;
//static mut offset: isize = 0; 
impl core::fmt::Write for VgaWriter{

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let vga = Vga;
        vga.clear();
        vga.write(s.as_bytes());
        Ok(())
    }

    // fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
    //     if let Some(s) = args.as_str() {
    //         self.write_str(s);
    //     } else {
    //         //write(&mut self, args);
    //     }
    //     Ok(())
    // }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::display::VgaWriter, format_args!($($arg)*));
    }
}