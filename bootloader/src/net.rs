///! This crate will be responsible for all things networking, it will take a [`crate::pci::Device`] from the [`crate::pci`] module
///!

use crate::serial_print;
use crate::pci;

const REG_CTRL: u32 = 0x0000; 
const REG_RCTL: u32 = 0x0100;
const REG_RDBAL: u32 = 0x2800;
const REG_RDBAH: u32 = 0x2804;
const REG_RDLEN0: u32 = 0x2808;
const REG_RDH0: u32 = 0x2810;
const REG_RDT0: u32 = 0x2818;
//const REG_RDLEN1: u32 = 0x2908;
//const REG_RDH1: u32 = 0x2910;
const REG_RAL: u32 = 0x5400; 
const REG_RAH: u32 = 0x5404;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Rdesc{
    buf_addr: u64,
    vlan: u16,
    erros: u8,
    status: u8,
    checksum: u16,
    length: u16,
}

#[derive(Default, Debug)]
struct Nic{
    mmio_base: u32,
    mac: [u8; 6],
}

impl Nic{
    /// Parses the mac address from RAL and RAH registers
    fn get_mac(&mut self){
        let upper32 = self.read_reg(REG_RAL);
        let lower16 = self.read_reg(REG_RAH);
    
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
    fn read_reg(&self, reg_offset: u32) -> u32{
        unsafe { core::ptr::read_volatile((self.mmio_base + reg_offset) as *const u32) }
    }
    /// Write to a register offset in the MMIO buffer
    fn write_reg(&self, reg_offset: u32, val: u32) {
        unsafe { core::ptr::write((self.mmio_base + reg_offset) as *mut u32, val) };
    }
    /// Send a rst to the CTRL register
    fn reset(&self){
        self.write_reg(REG_CTRL, self.read_reg(REG_CTRL) | (1 << 26));
        while self.read_reg(REG_CTRL) & (1 << 26) != 0 {}
        serial_print!("NIC Reset Complete...\n");
    }
    /// Set the memory address of the Recieve Buffer
    fn init_recieve_buffer_ptr(&self, buffer: u32){
        self.write_reg(REG_RDBAH, 0x0 as u32);
        self.write_reg(REG_RDBAL, buffer);
        serial_print!("Recieve Buffer Addr: {:#X}{:04X}\n", self.read_reg(REG_RDBAH), self.read_reg(REG_RDBAL));
    }
    /// Set the receive desciptor length queue, we set them both to 0x4
    fn init_buffer_length(&self){
        self.write_reg(REG_RDLEN0, 2 << 8);
        //self.write_reg(REG_RDLEN1, 8 << 8);
        serial_print!("RDLEN0: {:#032b}\n", self.read_reg(REG_RDLEN0));
        //serial_print!("RDLEN1: {:#032b}\n", self.read_reg(REG_RDLEN1));
    }
    /// Get the descriptor head location
    fn get_descriptor_head(&self) -> u32{
        self.read_reg(REG_RDH0)
    }
    /// Get the descriptor tail pointer
    fn get_descriptor_tail(&self) -> u32{
        self.read_reg(REG_RDT0)
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
    nic.reset();
    nic.get_mac();

    // Allocate the buffer to 0x800000 and give it 32 Rdescs
    let recv_buffer_addr: u32 = 0x800000;
    let recv_buffer_len = 32;
    
    let recv_buffer_ptr = recv_buffer_addr as *mut Rdesc;

    // Init rdesc with the address
    let mut rdesc = Rdesc::default();
    rdesc.buf_addr = recv_buffer_addr as u64;

    //Initialise the memory with the rdesc
    for offset in 0..recv_buffer_len{
        unsafe{ 
            core::ptr::write(recv_buffer_ptr.offset(offset), rdesc);
        }
    }

    serial_print!("Addr: {:#X?}\nPtr: {:#X?}\n", recv_buffer_addr, recv_buffer_ptr);

    nic.init_recieve_buffer_ptr(recv_buffer_addr as u32);

    // Set the length of the buffer, 128 bit aligned
    nic.init_buffer_length();
    
    // REC Buffer seems to default to 2048 Bytes buffer size
    serial_print!("RCTL {:#034b}\n", nic.read_reg(REG_RCTL));

    serial_print!("Head: {}\n", nic.get_descriptor_head());
    serial_print!("Tail: {}\n", nic.get_descriptor_tail());

    // serial_print!("{:#X?}\n", nic);

    loop{
        for offset in 0..recv_buffer_len{
            unsafe{ 
                let buf = core::ptr::read(recv_buffer_ptr.offset(offset));
                if rdesc.length != 0{
                    serial_print!("Raw bytes: {:X?}\n", buf);
                }
            }
        } 
        //cpu::halt()
    }
}
