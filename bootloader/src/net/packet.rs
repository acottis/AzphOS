//! This is the place where we define network packets and parse to and from
//! 

use core::mem::size_of;
/// This is our way of turning a raw packet buffer from the NIC into a more user friendly representation
/// 
#[derive(Debug)]
pub struct Packet<'a>{
    ethernet: Ethernet,
    ethertype: EtherType,
    data: &'a [u8],
}

impl<'a> Packet<'a>{
    /// Creates a packet
    //pub fn new(ethernet: Ethernet, ethertype: EtherType, data: &'static [u8]) -> Self {
    pub fn new() -> Self {
    
        let ethernet = Ethernet::new([0xFF; 6], [0x11; 6], 0x0608);
        let arp = Arp::new([0x11; 6]);
        let ethertype= EtherType::Arp(arp);
        let data = &[0u8];

        Self{
            ethernet,
            ethertype,
            data,
        }

    }
    /// Takes a pointer to a raw packet and its length and returns a user friendly representation
    /// Skips protocols we dont support
    pub fn parse(buffer_address: u64, length: u16) -> Option<Self> {
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
    // Converts struct to raw packet
    pub fn send(self, buffer: u64) {
        let ethertype = match self.ethertype{
            EtherType::Arp(arp) => { arp },
            _=> {unreachable!();},
        };
        unsafe {
            let ethernet = &*core::ptr::slice_from_raw_parts(
                (&self.ethernet as *const Ethernet) as *const u8, 
                size_of::<Ethernet>());

            let arp = &*core::ptr::slice_from_raw_parts(
                (&ethertype as *const Arp) as *const u8, 
                size_of::<Arp>());


                // serial_print!("{:X?}\n", ethernet);
                // serial_print!("{:X?}\n", arp);
                core::ptr::write(buffer as *mut [u8; size_of::<Ethernet>()], ethernet.try_into().unwrap());
                core::ptr::write((buffer + size_of::<Ethernet>() as u64) 
                    as *mut [u8; size_of::<Arp>()], arp.try_into().unwrap());
                
           //serial_print!("{:x?}\n", core::ptr::read(buffer as *mut [u8;42])) ;
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

    fn new(dst_mac: [u8; 6], src_mac: [u8; 6], ethertype: u16) -> Self{
        Self{
            dst_mac,
            src_mac,
            ethertype
        }
    }  

    fn parse(start_address: u64) -> Self {
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

impl Arp{
    fn new(src_mac: [u8; 6]) -> Self{
        Self{
            htype: 0x0100,
            ptype: 0x0008,
            hlen:  0x06,
            plen:  0x04,
            oper:  0x0100,
            sha:  src_mac,
            spa:  [0u8; 4],
            tha:  [0x00; 6],
            tpa:  [0xA, 0x63, 0x63, 0x01],
        }
    }
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