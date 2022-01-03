//! This crate will manage the finding of network cards in a [`crate::pci::Device`] and initialising them and exposing
//! to the rest of the OS
use crate::serial_print;
use crate::error::{Result, Error};
use core::mem::size_of;

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

const RECEIVE_QUEUE_HEAD_START: u32 = 20;
const RECEIVE_QUEUE_TAIL_START: u32 = 4;

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

    // Set the Receive Descriptor Length
    nic.write(REG_RDLEN, RECEIVE_DESC_BUF_LENGTH << 8);
    
    // Set the Receive Descriptor Head/Tail
    nic.write(REG_RDH, RECEIVE_QUEUE_HEAD_START);
    nic.write(REG_RDT, RECEIVE_QUEUE_TAIL_START);
    serial_print!("Head: {:#X?}, Tail: {:#X?}\n", nic.read(REG_RDH), nic.read(REG_RDT));

    // give them a size we want Set the Receive Descriptor Base Address
    nic.write(REG_RDBAH, (RECEIVE_DESC_BASE_ADDRESS >> 32) as u32 );
    nic.write(REG_RDBAL, RECEIVE_DESC_BASE_ADDRESS as u32);


    // Allocates all the buffers and memory
    Rdesc::init();
    
    // Main network loop, need to move out of here at some point, Loop through it in our "Main" and check in here from time to
    // time
    let rdesc_base_ptr = RECEIVE_DESC_BASE_ADDRESS as *mut Rdesc;
    loop{
        for offset in 0..RECEIVE_DESC_BUF_LENGTH as isize{
            unsafe{ 
                // Get the current Recieve Descriptor from our allocated memory and put it on the stack
                let mut rdesc: Rdesc = core::ptr::read_volatile(rdesc_base_ptr.offset(offset));
                
                //A non zero status means a packet has arrived and is ready for processing
                if rdesc.status != 0{
                    let packet = Packet::new(rdesc.buffer, rdesc.len);
                    // We only care about IPv4/ARP this will drop all the others without processing as when detected
                    // they return [`None`]
                    if let Some(p) = packet {
                        // Only print ARPs
                        if p.ethernet.ethertype == 0x0608{
                            serial_print!("H: {}, T: {}, Pos: {}, {:X?}\n", 
                                nic.read(REG_RDH), 
                                nic.read(REG_RDT), 
                                offset, 
                                rdesc);
                            serial_print!("{:X?}\n", p);
                        }   
                    }
                    // We have processed the packet and set status to 0 to indicate the buffer can overwrite
                    rdesc.status = 0;
                    rdesc.len = 0;
                    core::ptr::write_volatile(rdesc_base_ptr.offset(offset), rdesc);
                    nic.write(REG_RDT, (nic.read(REG_RDT) + 1) % 32)       
                }
            }
        } 
    }
    Ok(())
}

/// This is our way of turning a raw packet buffer from the NIC into a more user friendly representation
/// 
#[derive(Debug)]
struct Packet<'a>{
    ethernet: Ethernet,
    ethertype: EtherType,
    data: &'a [u8],
}

impl<'a> Packet<'a>{
    /// Takes a pointer to a raw packet and its length and returns a user friendly representation
    /// Skips protocols we dont support
    fn new(buffer_address: u64, length: u16) -> Option<Self> {
        let ethernet = Ethernet::parse(buffer_address);
        match ethernet.ethertype {
            // Ipv4
            0x0008 => {
                Some(Self{
                    ethernet,
                    ethertype: EtherType::IPv4(IPv4::headers(buffer_address)),
                    data: IPv4::data::<IPv4>(buffer_address, length),
                })
            },
            // Arp
            0x0608 => {
                Some(Self{
                    ethernet,
                    ethertype: EtherType::Arp(Arp::headers(buffer_address)),
                    data: Arp::data::<Arp>(buffer_address, length),
                })
            },
            // IPv6
            0xDD86 =>{ None },
            _ => { None },
        }
    }
}

/// This struct is a representation of an Ethernet frame
#[repr(C)]
#[derive(Debug)]
struct Ethernet{
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    ethertype: u16,
}

impl Ethernet{
    fn parse(start_address: u64) -> Ethernet {
        unsafe{
            core::ptr::read(start_address as *const Self)
        }
    }
}

#[derive(Debug)]
enum EtherType{
    IPv4(IPv4),
    Arp(Arp),
}

/// This struct is a representation of an ARP Header 
#[derive(Debug)]
#[repr(C)]
struct Arp{
    /// Hardware type
    htype: u16,
    /// Protocol Address Length
    ptype: u16,
    /// Hardware Address Length
    hlen: u8,
    /// Protocol Address Length
    plen: u8,
    /// Operation
    oper: u16,
    /// Sender hardware address
    sha: [u8; 6],
    /// Sender protocol address
    spa: [u8; 4],
    /// Target hardware address
    tha: [u8; 6],
    /// Target protocol address
    tpa: [u8; 4],
}

impl ParsePacket for Arp{}

/// This struct is a representation of an IPv4 Header, we dont handle Options
#[repr(C)]
#[derive(Debug)]
struct IPv4{
    version_ihl: u8, 
    dcp_ecn: u8, 
    total_len: u16,
    identification: u16,
    flags_fragmentoffset: u16,
    ttl: u8,
    protocol: IPProtocol,
    header_checksum: u16,
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
}

impl ParsePacket for IPv4{}

#[repr(u8)]
#[derive(Debug)]
enum IPProtocol{
    UDP = 0x11,
}

/// This trait allows us to generically handle getting the headers and data for IPv4 and ARP (ARP Doesnt have data though...?)
trait ParsePacket {
    /// Starts reading the packet from the end point of the ethernet header for the length of the EtherType
    fn headers<T>(start_address: u64) -> T{
        unsafe{
            core::ptr::read_unaligned((start_address+(size_of::<Ethernet>()) as u64) as *const T)
        }
    }  
    /// Reads the extra bytes at the end of the packets headers
    fn data<T>(start_address: u64, length: u16) -> &'static [u8]{
        let data_len = length as u16 - (size_of::<Ethernet>() as u16 + size_of::<T>() as u16);
        unsafe{
            &*core::ptr::slice_from_raw_parts(
                (start_address + 
                size_of::<Ethernet>() as u64 + 
                size_of::<T>() as u64) as *const u8, data_len as usize)
        }
    }
}