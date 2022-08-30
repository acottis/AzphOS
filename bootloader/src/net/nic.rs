//! This crate will manage the finding of network cards in a [`crate::pci::Device`] and initialising them and exposing
//! to the rest of the OS our main entry points from our OS to our nic are [NetworkCard::send] and [NetworkCard::recieve]
use super::Packet;
use super::MTU;
use crate::error::{Error, Result};

// Supported Nics
// E1000 Qemu Versions
const E1000: (u16, u16) = (0x100E, 0x8086);
const PACKET_SIZE: u64 = 2048;

// Register offsets of the E1000
const REG_RCTL: u32 = 0x0100;
const REG_RDBAL: u32 = 0x2800;
const REG_RDBAH: u32 = 0x2804;
const REG_RDLEN: u32 = 0x2808;
const REG_RDH: u32 = 0x2810;
const REG_RDT: u32 = 0x2818;
const REG_TDBAL: u32 = 0x3800;
const REG_TDBAH: u32 = 0x3804;
const REG_TDLEN: u32 = 0x3808;
const REG_TDH: u32 = 0x3810;
const REG_TDT: u32 = 0x3818;
const REG_TCTL: u32 = 0x0400;
const REG_RAL: u32 = 0x5400;
const REG_RAH: u32 = 0x5404;

// Recieve Addresses Base addresses
const RECEIVE_DESC_BASE_ADDRESS: u64 = 0x800000;
const RECEIVE_DESC_BUF_LENGTH: u32 = 16;
const RECEIVE_BASE_BUFFER_ADDRESS: u64 = 0x880000;
const RECEIVE_QUEUE_HEAD_START: u32 = 20;
const RECEIVE_QUEUE_TAIL_START: u32 = 4;

// Transmit base addresses
const TRANSMIT_DESC_BASE_ADDRESS: u64 = 0x900000;
const TRANSMIT_DESC_BUF_LENGTH: u32 = 16;
const TRANSMIT_BASE_BUFFER_ADDRESS: u64 = 0x980000;
const TRANSMIT_QUEUE_HEAD_START: u32 = 0;
const TRANSMIT_QUEUE_TAIL_START: u32 = 0;

/// This struct is the receive descriptor format that stores the packet metadata and the buffer points to the packet
/// location in memory
#[derive(Debug, Default)]
#[repr(C)]
struct Rdesc {
    buffer: u64,
    len: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

impl Rdesc {
    /// First we put the recieve registers on the NIC into our desired state, such
    /// as the memory base address, tail/head, and size of buffer
    /// Sets up a buffer of [`Rdesc`]'s with [RECEIVE_DESC_BUF_LENGTH] length and writes them
    /// to [RECEIVE_DESC_BASE_ADDRESS]
    /// We set the [`Rdesc.buffer`] field to the address where we want the raw packet to be, this packet size is determined by
    /// [PACKET_SIZE] this allocation is YOLO as we dont check ANYTHING
    pub fn init(nic: &NetworkCard) {
        // Set the Receive Descriptor Length
        nic.write(REG_RDLEN, RECEIVE_DESC_BUF_LENGTH << 8);

        // Set the Receive Descriptor Head/Tail
        nic.write(REG_RDH, RECEIVE_QUEUE_HEAD_START);
        nic.write(REG_RDT, RECEIVE_QUEUE_TAIL_START);

        // give them a size we want Set the Receive Descriptor Base Address
        nic.write(REG_RDBAH, (RECEIVE_DESC_BASE_ADDRESS >> 32) as u32);
        nic.write(REG_RDBAL, RECEIVE_DESC_BASE_ADDRESS as u32);

        // Enable Recv | Dont store bad packets | Enable Unicast Promiscuous | Enable Multicast Promiscuous |
        // Enable Broadcast Accept Mode | Set the RTCL BSIZE to 2048 |
        nic.write(
            REG_RCTL,
            (1 << 1) | (2 << 0) | (1 << 3) | (1 << 4) | (1 << 15) | (1 << 26),
        );

        // Zero out the chosen memory location and place the memory location for the raw packets in the
        // Recieve buffer field in the [`Rdesc`] struct
        let rdesc_base_ptr = RECEIVE_DESC_BASE_ADDRESS as *mut Rdesc;
        for offset in 0..RECEIVE_DESC_BUF_LENGTH as isize {
            let rdesc = Self {
                buffer: RECEIVE_BASE_BUFFER_ADDRESS + (offset as u64 * PACKET_SIZE),
                ..Default::default()
            };
            unsafe {
                core::ptr::write(rdesc_base_ptr.offset(offset), rdesc);
            }
        }
    }
}

/// This struct is the transmit descriptor format that stores the packet metadata and the buffer points to the packet
/// location in memory
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct Tdesc {
    buffer: u64,
    len: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

impl Tdesc {
    pub fn init(nic: &NetworkCard) {
        // Set the Transmit Descriptor Length
        nic.write(REG_TDLEN, TRANSMIT_DESC_BUF_LENGTH << 8);

        // Set the Transmit Descriptor Head/Tail
        nic.write(REG_TDH, TRANSMIT_QUEUE_HEAD_START);
        nic.write(REG_TDT, TRANSMIT_QUEUE_TAIL_START);

        // give them a size we want Set the Transmit Descriptor Base Address
        nic.write(REG_TDBAH, (TRANSMIT_DESC_BASE_ADDRESS >> 32) as u32);
        nic.write(REG_TDBAL, TRANSMIT_DESC_BASE_ADDRESS as u32);

        // Enable Packet transmissiong, need to look into this more
        //serial_print!("TX CTRL: {:#b}\n",nic.read(0x400));
        nic.write(REG_TCTL, 1 << 1);

        // Zero out the chosen memory location and place the memory location for the raw packets in the
        // Transmit buffer field in the [`Rdesc`] struct
        let tdesc_base_ptr = TRANSMIT_DESC_BASE_ADDRESS as *mut Tdesc;
        for offset in 0..TRANSMIT_DESC_BUF_LENGTH as isize {
            let tdesc = Self {
                buffer: TRANSMIT_BASE_BUFFER_ADDRESS + (offset as u64 * PACKET_SIZE),
                ..Default::default()
            };
            unsafe {
                core::ptr::write(tdesc_base_ptr.offset(offset), tdesc);
            }
        }
    }
}
/// This struct finds the network card and stores information we need from it
#[derive(Default, Debug, Clone, Copy)]
pub struct NetworkCard {
    mmio_base: u32,
    pub mac: [u8; 6],
}

impl NetworkCard {
    /// Create new instance of Network card and get MAC
    fn new(device: crate::pci::Device) -> Self {
        let mut nic = Self {
            mmio_base: device.base_mem_addrs()[0],
            ..Default::default()
        };
        nic.get_mac();
        nic
    }
    /// Read from a register offset in the MMIO buffer
    fn read(&self, reg_offset: u32) -> u32 {
        unsafe { core::ptr::read((self.mmio_base + reg_offset) as *const u32) }
    }
    /// Write to a register offset in the MMIO buffer
    fn write(&self, reg_offset: u32, val: u32) {
        unsafe { core::ptr::write((self.mmio_base + reg_offset) as *mut u32, val) };
    }
    /// Parses the mac address from RAL and RAH registers
    fn get_mac(&mut self) {
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
    /// This function will be able to send packets and will be exposed
    /// We currently only support one descriptor in the buffer
    pub fn send(&self, buf: &[u8; MTU], len: usize) {
        // 48 is the minimum packet size
        let len = if len < 48 { 48 } else { len };

        // Get a ptr to the base address of the descriptors
        let tdesc_base_ptr = TRANSMIT_DESC_BASE_ADDRESS as *mut Tdesc;
        unsafe {
            // Get the current tdesc (index 0)
            let mut tdesc: Tdesc = core::ptr::read(tdesc_base_ptr.offset(0));
            // If the status indicates it has been procesed, move the tail down again
            if tdesc.status == 1 {
                self.write(REG_TDT, (self.read(REG_TDT) - 1) % 32);
                tdesc.status = 0;
            }
            // Write the packet to the buffer
            core::ptr::write(tdesc.buffer as *mut [u8; MTU], *buf);

            // serial_print!("Sent Packet! H: {}, T: {}, Pos: {}, {:X?}\n",
            //     self.read(REG_TDH),
            //     self.read(REG_TDT),
            //     0,
            //     tdesc);

            // The len to send on the wire, this allows us to give it a buffer
            // of any size then only send the len we have chosen
            tdesc.len = len as u16;
            // Sets the command for End of Packet | Insert FCS/CRC | Enable Report Status
            tdesc.cmd = (1 << 3) | (1 << 1) | 1;
            // Writes out modified descriptor to the memory location of the descriptor
            core::ptr::write(tdesc_base_ptr.offset(0), tdesc);
            // Moves the Tail up to request the NIC to process the packet
            self.write(REG_TDT, (self.read(REG_TDT) + 1) % 32);
        }
    }
    /// This function processes the emails in buffer of buffer size [RECEIVE_DESC_BUF_LENGTH]
    pub fn receive(&self) -> [Option<Packet>; RECEIVE_DESC_BUF_LENGTH as usize] {
        
        let mut received_packets: [Option<Packet>; RECEIVE_DESC_BUF_LENGTH as usize] =
            [Default::default(); RECEIVE_DESC_BUF_LENGTH as usize];
        let mut packet_counter = 0;
        let rdesc_base_ptr = RECEIVE_DESC_BASE_ADDRESS as *mut Rdesc;

        for offset in 0..RECEIVE_DESC_BUF_LENGTH as isize {
            unsafe {
                // Get the current Recieve Descriptor from our allocated memory and put it on the stack
                let mut rdesc: Rdesc = core::ptr::read(rdesc_base_ptr.offset(offset));

                //A non zero status means a packet has arrived and is ready for processing
                if rdesc.status != 0 {
                    // Read the data from the packet
                    let buf: [u8; MTU] = core::ptr::read(rdesc.buffer as *const [u8; MTU]);

                    // Try to parse the packet and add it to the array to hand back to the OS
                    let packet = Packet::parse(&buf, rdesc.len as usize);
                    //crate::serial_print!("{:X?}\n", packet);
                    received_packets[packet_counter] = packet;
                    packet_counter += 1;

                    // We have processed the packet and set status to 0 to indicate the buffer can overwrite
                    rdesc.status = 0;
                    rdesc.len = 0;

                    // Write modified rdesc pack to memory
                    core::ptr::write(rdesc_base_ptr.offset(offset), rdesc);
                    // Adds one to the tail to let the NIC know we are done with that one
                    self.write(REG_RDT, (self.read(REG_RDT) + 1) % RECEIVE_DESC_BUF_LENGTH)
                }
            }
        }
        received_packets
    }
}
/// Main entry point to net that sets up the drivers
///
pub fn init() -> Result<NetworkCard> {
    // This will get us the first device that is an Ethernet Network Card or return an Error
    let device = match crate::pci::init().get_nic() {
        Some(device) => {
            // Error if we dont recongise NIC
            let did_vid = device.did_vid();
            if did_vid != E1000 {
                return Err(Error::UnsupportedNIC(did_vid));
            }
            device
        }
        None => return Err(Error::NoNICFound),
    };

    // Create a new NIC
    let nic = NetworkCard::new(device);

    // Puts the Recieve registers into our desired state and Allocates all the buffers and memory
    Rdesc::init(&nic);

    // Puts the Transmit registers into our desired state
    Tdesc::init(&nic);

    Ok(nic)
}
