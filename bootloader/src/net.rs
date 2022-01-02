//! This crate will manage the finding of network cards in a [`crate::pci::Device`] and initialising them and exposing
//! to the rest of the OS

use crate::serial_print;
use crate::error::{Result, Error};

// Supported Nics
// E1000 Qemu Versions
const E1000: (u16, u16) = (0x100E, 0x8086);

// Register offsets of the E1000
const REG_CTRL: u32 = 0x0000; 
const REG_RCTL: u32 = 0x0100;
const REG_RDBAL: u32 = 0x2800;
const REG_RDBAH: u32 = 0x2804;
const REG_RDLEN: u32 = 0x2808;
const REG_RDH: u32 = 0x2810;
const REG_RDT: u32 = 0x2818;
const REG_RAL: u32 = 0x5400; 
const REG_RAH: u32 = 0x5404;

// Base addresses
const RECEIVE_DESC_BASE_ADDRESS: u64 = 0x800000;
const RECEIVE_DESC_BUF_LENGTH: u32 = 32;
const BASE_RECV_BUFFER_ADDRESS: u64 = 0x900000;
const PACKET_SIZE: u64 = 2048;

/// This struct is the receive descriptor format that stores the packet metadata and the buffer points to the packet
/// location in memory
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct Rdesc {
    buffer:   u64,
    len:      u16,
    checksum: u16,
    status:   u8,
    errors:   u8,
    special:  u16,
}

impl Rdesc{
    /// Sets up a buffer of [`Rdesc`]'s with [RECEIVE_DESC_BUF_LENGTH] length and writes them to [RECEIVE_DESC_BASE_ADDRESS]
    /// We set the [`Rdesc.buffer`] field to the address where we want the raw packet to be, this packet size is determined by
    /// [PACKET_SIZE] this allocation is YOLO as we dont check ANYTHING
    pub fn init(){
        let rdesc_base_ptr = RECEIVE_DESC_BASE_ADDRESS as *mut Rdesc;
        for offset in 0..RECEIVE_DESC_BUF_LENGTH as isize{
            let rdesc = Self {
                buffer: BASE_RECV_BUFFER_ADDRESS + (offset as u64 * PACKET_SIZE),
                ..Default::default()
            };
            unsafe{
                core::ptr::write(rdesc_base_ptr.offset(offset), rdesc);
           }
        };
    }
}

/// This struct finds the network card and stores information we need from it
#[derive(Default, Debug)]
struct NetworkCard {
    mmio_base: u32,
    mac: [u8; 6],
}

impl NetworkCard{
    /// Create new instance of Network card and get MAC
    fn new(device: crate::pci::Device) -> Self{
        let mut nic = Self {
            mmio_base: device.base_mem_addrs()[0],
            ..Default::default()
        };
        nic.get_mac();
        nic
    }
    /// Read from a register offset in the MMIO buffer
    fn read(&self, reg_offset: u32) -> u32{
        unsafe { core::ptr::read((self.mmio_base + reg_offset) as *const u32) }
    }
    /// Write to a register offset in the MMIO buffer
    fn write(&self, reg_offset: u32, val: u32) {
        unsafe { core::ptr::write((self.mmio_base + reg_offset) as *mut u32, val) };
    }
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
}


/// Main entry point to net that sets up the drivers
/// 
pub fn init() -> Result<()> {
    // This will get us the first device that is an Ethernet Network Card or return an Error
    let device = match crate::pci::init().get_nic(){
        Some(device) => {
            // Error if we dont recongise NIC
            let did_vid = device.did_vid();
            if did_vid != E1000{
                return Err(Error::UnsupportedNIC(did_vid))
            }
            device
        },
        None => {
            return Err(Error::NoNICFound)
        },
    };

    // Create a new NIC
    let nic = NetworkCard::new(device);

    // !TODO give them a size we want Set the Receive Descriptor Base Address
    nic.write(REG_RDBAH, (RECEIVE_DESC_BASE_ADDRESS >> 32) as u32 );
    nic.write(REG_RDBAL, RECEIVE_DESC_BASE_ADDRESS as u32);

    // Set the Receive Descriptor Length
    nic.write(REG_RDLEN, RECEIVE_DESC_BUF_LENGTH << 8);

    // Allocates all the buffers and memory
    Rdesc::init();

    serial_print!("{:X?}\n", nic);
    serial_print!("Head: {:#X?}, Tail: {:#X?}\n", nic.read(REG_RDH), nic.read(REG_RDT));

    let rdesc_base_ptr = RECEIVE_DESC_BASE_ADDRESS as *mut Rdesc;
    loop{
        for offset in 0..RECEIVE_DESC_BUF_LENGTH as isize{
            unsafe{ 
                let rdesc = core::ptr::read(rdesc_base_ptr.offset(offset));
                if rdesc.len != 0{
                    serial_print!("Head: {:#X?}, Tail: {:#X?}, ", nic.read(REG_RDH), nic.read(REG_RDT));
                    serial_print!("Location: {}, {:X?}\n", offset, rdesc);

                    let packet = Packet::new(rdesc.buffer);
                    serial_print!("{:X?}\n", packet);
                    crate::cpu::halt();

                }
            }
        } 
        //cpu::halt()
    }

    Ok(())
}

#[derive(Debug)]
struct Packet{
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    typ: u8,
    ip_version: u8,
    _unknown: u8,
    total_len: u8,
}

impl Packet{
    fn new(buffer_address: u64) -> Self{
        unsafe {
            core::ptr::read(buffer_address as *const Self)
        }
    }
}