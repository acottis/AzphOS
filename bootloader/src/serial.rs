//! This crate is for managing all Serial Port related functionality and exposes as a macro
//! [`serial_print!`] This macro can take format args and will print to Serial Port0 on
//! the computer
use crate::cpu;

/// The address that contains the data on what serial ports we have
static BDA_SERIALPORTS: u16 = 0x400;

/// Once we `init()` our serial ports we store them here, probably horribily unsafe if we ever had threads
static mut SERIALPORTS: [u16; 4] = [0u16; 4];

/// This function uses the BDA defined area to look for serial ports, it is called if we see `SERIALPORTS` still has a 0
/// at its first index 
fn init() {
    for port in 0..4{
        let addr = unsafe { 
            let addr = core::ptr::read((BDA_SERIALPORTS as *const u16)
                .offset(port));
            SERIALPORTS[port as usize] = addr;
            addr
        };
        // If we find a port, init it using known good values
        if addr != 0{
            cpu::out8(addr + 1, 0x00);    // Disable all interrupts
            cpu::out8(addr + 3, 0x80);    // Enable DLAB (set baud rate divisor)
            cpu::out8(addr + 0, 0x01);    // Set divisor to 1 (lo byte) 115200/1 baud
            cpu::out8(addr + 1, 0x00);    //                  (hi byte)
            cpu::out8(addr + 3, 0x03);    // 8-bit: Bits 0,1  (00000011), no parity Bits 3,4,5(00000000), one stop bit: Bits 2 (00000000)
            cpu::out8(addr + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
            cpu::out8(addr + 4, 0x0B);    // IRQs enabled, RTS/DSR set
            //cpu::out8(addr + 4, 0x1E);    // Set in loopback mode, test the serial chip
            //cpu::out8(addr + 0, 0xAE);    // Test serial chip (send byte 0xAE and check if serial returns same byte)
        }
    }
}
/// this function handles writes, we only write to `SERIALPORTS[0]` right now
/// 
fn write(bytes: &[u8]){
    let port = unsafe{ SERIALPORTS[0] };
    for byte in bytes{
        if *byte == '\n' as u8{
            cpu::out8(port, '\r' as u8);
        }
        cpu::out8(port, *byte);
    }
}
/// This struct is the one we implement [`core::fmt::Write`]
pub struct SerialWriter;

impl core::fmt::Write for SerialWriter{
    /// Trait implementation of [`core::fmt::Write`] will [`init()`] our serial ports if we havent already
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Check if we have initialised the Serial Ports
        let serial_initialised = unsafe { SERIALPORTS[0] != 0 };
        if !serial_initialised{
            init();
            crate::serial_print!("Initialising Serial...\n");
        }
        write(s.as_bytes());
        Ok(())
    }
}
/// This macro is how the user accesses the serial port, our implementation of [`std::print!`](https://doc.rust-lang.org/std/macro.print.html)
/// 
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::serial::SerialWriter, format_args!($($arg)*));
    }
}