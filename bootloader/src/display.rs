/// This the static VGA buffer location on an x86
/// 
static VGA_BUFFER: u32 = 0xB8000;
static VGA_BUFFER_MAX: u32 = 0xFA0;
static GREEN: u16 = 0x0200;

/// This struct handles outputting to the screen using the VGA Buffer
/// 
pub struct Vga{
    buffer_base: *mut u16,
    offset: isize
}

impl Vga{
    /// Sets up the VGA buffer
    /// 
    pub fn init() -> Self{
        Self{
            buffer_base: VGA_BUFFER as *mut u16,
            offset: 0,
        }
    }
    /// prints a string only supporting green text at the moment for size
    /// 
    fn write_string(&mut self, s: &str){
        let bytes = s.as_bytes();
        unsafe {
            for byte in bytes{
                *self.buffer_base.offset(self.offset) = GREEN + (*byte as u16);
                self.offset += 1;
            }   
        }
    }
    /// Clears the screen by zeroing out the buffer
    /// 
    pub fn clear(&self){
        for chr in (VGA_BUFFER..VGA_BUFFER+VGA_BUFFER_MAX).step_by(2) {
            unsafe{ core::ptr::write(chr as *mut u16, 0x0000 ); }
        }
    }
}

impl core::fmt::Write for Vga{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.write_string(s);
        Ok(())
    }
}