// use core::fmt::Write;
// /// This the static VGA buffer location on an x86
// /// 
// static VGA_BUFFER: u32 = 0xB8000;
// static VGA_BUFFER_MAX: u32 = 4000;
// static GREEN: u16 = 0x0200;

// /// This struct handles outputting to the screen using the VGA Buffer
// /// 
// pub struct Vga{
//     buffer_base: *mut u16,
//     offset: isize
// }

// impl Vga{
//     /// Sets up the VGA buffer
//     /// 
//     pub fn new() -> Self{
//         Self{
//             buffer_base: VGA_BUFFER as *mut u16,
//             offset: 0,
//         }
//     }
//     /// prints a string only supporting green text at the moment for size
//     /// 
//     pub fn write_bytes<'a>(&mut self, s: &'a[u8]){
//         unsafe {
//             for byte in s{
//                 *self.buffer_base.offset(self.offset) = GREEN + (*byte as u16);
//                 self.offset += 1;
//             }   
//         }
//     }
//     /// Clears the screen by zeroing out the buffer
//     /// 
//     pub fn clear(&self){
//         for chr in (VGA_BUFFER..VGA_BUFFER+VGA_BUFFER_MAX).step_by(2) {
//             unsafe{ core::ptr::write(chr as *mut u16, 0x0000 ); }
//         }
//     }
// }

// impl Write for Vga {
//     fn write_str(&mut self, s: &str) -> core::fmt::Result {
//         unsafe {
//             for byte in s.as_bytes(){
//                 *self.buffer_base.offset(self.offset) = GREEN + (*byte as u16);
//                 self.offset += 1;
//             }   
//         }
//         Ok(())
//     }
    
//     fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
//         //self.write_string(args);
//         if let Some(s) = args.as_str() {
//             self.write_str(s);
//         } else {
//             panic!("Could not write format");
//             //self.write_str(format_args!("{:?}", args));
//         }
//         Ok(())
//     }

// }

pub struct VgaWriter;

impl core::fmt::Write for VgaWriter{

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let buffer_base = 0xB8000 as *mut u16;
        let mut offset = 0;
        unsafe {
            for byte in s.as_bytes(){
                *buffer_base.offset(offset) = 0x0200 + (*byte as u16);
                offset += 1;
            }   
        }
        Ok(())
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        if let Some(s) = args.as_str() {
            self.write_str(s);
        } else {
            self.write_str("Error while writing string");
            //self.write_str(format_args!("{:?}", args));
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut crate::display::VgaWriter, 
            format_args!($($arg)*)
        );
    }
}