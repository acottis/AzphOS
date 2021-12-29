use crate::cpu::out8;
use crate::print;
use crate::cpu;

#[derive(Debug)]
pub struct SerialPort{
    ports: [u16; 4]
}

impl SerialPort{
    
    pub fn init() -> Self{
        let mut ports = [0u16; 4];
        for offset in 0..4{
            let addr = unsafe { 
                core::ptr::read((0x400+(offset*16) as u16) as *const u16) 
            };
            if addr != 0{
                cpu::out8(addr + 1, 0x00);    // Disable all interrupts
                cpu::out8(addr + 3, 0x80);    // Enable DLAB (set baud rate divisor)
                cpu::out8(addr + 0, 0x03);    // Set divisor to 3 (lo byte) 38400 baud
                cpu::out8(addr + 1, 0x00);    //                  (hi byte)
                cpu::out8(addr + 3, 0x03);    // 8 bits, no parity, one stop bit
                cpu::out8(addr + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
                cpu::out8(addr + 4, 0x0B);    // IRQs enabled, RTS/DSR set
                //cpu::out8(addr + 4, 0x1E);    // Set in loopback mode, test the serial chip
                //cpu::out8(addr + 0, 0xAE);    // Test serial chip (send byte 0xAE and check if serial returns same byte)
            }
            ports[offset] = addr;
        }
        Self{
            ports
        }
    }

    pub fn read(&self){
       
        print!("HEY: {:#X}\n", cpu::in8(0x3f8+5));
        loop{
            out8(self.ports[1], 0x50);
            out8(self.ports[0], 0x50);
        }
    }
}