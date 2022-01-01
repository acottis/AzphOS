///! This crate will be responsible for all things networking, it will take a [`crate::pci::Device`] from the [`crate::pci`] module
///!
use crate::{pci, serial_print};

const REG_CTRL: u32 = 0x0000; 
const REG_RAL: u32 = 0x5400; 
const REG_RAH: u32 = 0x5404; 

#[derive(Default, Debug)]
struct Nic{
    mmio_base: u32,
    mac: [u8; 6],
}

impl Nic{
    /// Parses the mac address from RAL and RAH registers
    fn get_mac(&mut self){
        let upper32 = self.read(REG_RAL);
        let lower16 = self.read(REG_RAH);
    
        self.mac = [
            upper32 as u8,
            (upper32 >> 8) as u8,
            (upper32 >> 16) as u8,
            (upper32 >> 24) as u8,
            lower16 as u8,
            (lower16 >> 8) as u8,
        ]
    
    }
    /// Read from a register offset in the MMIO buffer
    fn read(&self, reg_offset: u32) -> u32{
        unsafe { core::ptr::read_volatile((self.mmio_base + reg_offset) as *const u32) }
    }
    /// Write to a register offset in the MMIO buffer
    fn write(&self, reg_offset: u32, val: u32) {
        unsafe { core::ptr::write((self.mmio_base + reg_offset) as *mut u32, val) };
    }
    /// Send a rst to the CTRL register
    fn reset(&self){
        self.write(REG_CTRL, self.read(REG_CTRL) | (1 << 26));
        while self.read(REG_CTRL) & (1 << 26) != 0 {}
        serial_print!("NIC Reset Complete...\n");
    }
}

/// [`init`] will take a ['crate::pci::Device`] and parse the important information
pub fn init(){

    let pci_nic = match pci::init().get_nic(){
        Some(device) => {
            device
        }
        None => {
            serial_print!("Could not init networking...\n");
            return 
        },
    };

    let mut nic = Nic {
        mmio_base: pci_nic.base_mem_addrs()[0],
        ..Default::default()
    };
    nic.get_mac();
    nic.reset();

    serial_print!("{:#X?}\n", nic);

}
