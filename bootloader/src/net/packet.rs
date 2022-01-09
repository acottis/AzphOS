//! This is the place where we define network packets and parse to and from
//! 

use core::mem::size_of;
use core::ptr::{slice_from_raw_parts, write};
use crate::net::dhcp::DHCP_TOTAL_LEN;

use super::MAC;
use super::Serialise;

/// The size of IPv4 Headers, we dont support ipv4 options
const IPV4_HEADER_LEN: usize = 20;
/// The size of UDP header
const UDP_HEADER_LEN: u16 = 8;
/// The size of an Ethernet header
const ETH_HEADER_LEN: usize = 14;
/// This is our way of turning a raw packet buffer from the NIC into a more user friendly representation
/// 
#[derive(Debug, Clone, Copy)]
pub struct Packet<'a>{
    ethernet: Ethernet,
    pub ethertype: EtherType<'a>,
}

impl<'a> Packet<'a>{
    /// Creates a packet
    //pub fn new(ethernet: Ethernet, ethertype: EtherType, data: &'static [u8]) -> Self {
    pub fn new(ethertype:  EtherType<'a>) -> Self {
    
        let ethernet = match ethertype {
            EtherType::Arp(_) => Ethernet::new([0xFF; 6], MAC, 0x0608),
            EtherType::IPv4(_) => Ethernet::new([0xFF; 6], MAC, 0x0008),        
        };

        Self{
            ethernet,
            ethertype,
        }

    }
    /// Takes a pointer to a raw packet and its length and returns a user friendly representation
    /// Skips protocols we dont support
    pub fn parse(buffer_address: u64, length: usize) -> Option<Self>{
       // crate::serial_print!("Recieved packet, address: {:#X}, Length {}\n", buffer_address, length);

        // Gets the raw data from the memory buffer given to the NIC
        let raw = unsafe {
            core::slice::from_raw_parts(buffer_address as *const u8, length as usize)
        };
        //crate::serial_print!("{:X?}\n", raw);

        // Parse out the Ethernet header
        let ethernet = Ethernet::deserialise(&raw[0..ETH_HEADER_LEN], ETH_HEADER_LEN).unwrap();
        //crate::serial_print!("{:X?}\n", ethernet);

        // Depending on the Ethertype do...
        match ethernet.ethertype{
            // Ipv4
            0x0800 => {
               // crate::serial_print!("Found IPv4\n");
                let ipv4 = IPv4::deserialise(&raw[ETH_HEADER_LEN..length as usize], IPV4_HEADER_LEN);
                if let Some(ipv4) = ipv4{
                    Some(Self{
                        ethernet,
                        ethertype: EtherType::IPv4(ipv4),
                    })
                }else{
                    None
                }
            },
            // Arp
            0x0806 => {
               // crate::serial_print!("Found Arp\n");
                let arp: Option<Arp> = Arp::deserialise(&raw[ETH_HEADER_LEN..length as usize], length);
                if let Some(arp) = arp{
                    Some(Self{
                        ethernet,
                        ethertype: EtherType::Arp(arp),
                    })
                }else{
                    None
                }
            },
            // We dont recongise the ethertype, drop it
            _=> None
        }
    }
    // Converts struct to raw packet and returns the length to the NIC
    pub fn send(self, buffer: u64) -> u16 {
        unsafe {        
            let mut writer_ptr = buffer;

            // Write the ethernet header
            write(writer_ptr as *mut [u8; size_of::<Ethernet>()], self.ethernet.serialise().try_into().unwrap());
            writer_ptr += size_of::<Ethernet>() as u64;

            match self.ethertype{
                EtherType::Arp(arp) => { 
                        write(
                            writer_ptr as *mut [u8; size_of::<Arp>()], 
                            arp.serialise().try_into().unwrap()
                        );
                        writer_ptr += size_of::<Arp>() as u64;        
                },
                EtherType::IPv4(ipv4) => {                                      
                    // Write the IPv4 Header
                    write(
                        writer_ptr as *mut [u8; IPV4_HEADER_LEN as usize], 
                        ipv4.serialise().try_into().unwrap()
                    ); 
                    writer_ptr += IPV4_HEADER_LEN as u64;

                    match ipv4.protocol_data{
                        Protocol::UDP(udp) => {
                            // Write the UDP Header
                            write(
                                writer_ptr as *mut [u8; UDP_HEADER_LEN as usize], 
                                udp.serialise().try_into().unwrap()
                            );
                            writer_ptr += UDP_HEADER_LEN as u64;

                            // Write the UDP Data
                            write(
                                writer_ptr as *mut [u8; DHCP_TOTAL_LEN],
                                udp.payload.try_into().unwrap()
                            );
                            writer_ptr += DHCP_TOTAL_LEN as u64;
                        }
                    }
                }
                _=> {
                    unreachable!()
                },
            }
            // Send the packet length to the NIC
            (writer_ptr - buffer) as u16
        }
    }
}


/// This struct is a representation of an Ethernet frame
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy)]
pub enum EtherType<'a>{
    IPv4(IPv4<'a>),
    Arp(Arp),
}

/// This struct is a representation of an ARP Header 
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Arp{
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
    pub fn new() -> Self{
        Self{
            htype: 0x0100,
            ptype: 0x0008,
            hlen:  0x06,
            plen:  0x04,
            oper:  0x0100,
            sha:  super::MAC,
            spa:  [0u8; 4],
            tha:  [0x00; 6],
            tpa:  [0xA, 0x63, 0x63, 0x01],
        }
    }
}

/// This struct is a representation of an IPv4 Header, we dont handle Options
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IPv4<'a>{
    version_ihl: u8, 
    dcp_ecn: u8, 
    total_len: u16,
    identification: u16,
    flags_fragmentoffset: u16,
    ttl: u8,
    protocol_type: u8,
    header_checksum: u16,
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    pub protocol_data: Protocol<'a>,
}

impl<'a> IPv4<'a>{
    pub fn new(protocol: Protocol<'a>) -> Self{
        let len = match protocol {
            Protocol::UDP(udp) => {
                udp.len.to_be()
            },
        };
        let mut ipv4 = Self {
            version_ihl: 0x45, 
            dcp_ecn: 0x00, 
            // PROBLEM
            total_len: (IPV4_HEADER_LEN as u16 + len).to_be(),
            identification: (0x0100u16).to_be(),
            flags_fragmentoffset: 0x00,
            ttl: 0x40,
            protocol_type: 0x11,
            // PROBLEM
            header_checksum: 0,
            src_ip: [0x0; 4],
            dst_ip: [0xFF; 4],
            protocol_data: protocol,
        };
        ipv4.checksum();
        ipv4
    }
    /// This calculates the IPv4 checksum on creation of the header
    fn checksum(&mut self){
        let raw = self.serialise();
        let mut total: u32 = 0;
        for index in (0..raw.len()).step_by(2){
            let tmp: u32 = ((raw[index] as u32) << 8) | (raw[index+1]) as u32;
            total += tmp;
        }
        total = (total + (total >> 16)) & 0x0000FFFF;
        // This catches the wierd edge case where our carry creates another carry
        total = total + (total >> 16);

        self.header_checksum = (!total as u16).to_be();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Protocol<'a>{
    UDP(Udp<'a>)
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Udp<'a>{
    src_port: u16,
    dst_port: u16,
    len: u16,
    checksum: u16,
    payload: &'a [u8]
}
impl<'a> Udp<'a>{
    pub fn new(payload: &'a[u8]) -> Self{
        Self {
            src_port: (68 as u16).to_be(),
            dst_port: (67 as u16).to_be(),
            len: (payload.len() as u16 + UDP_HEADER_LEN).to_be(),
            // Unimplemented
            checksum: 0,
            payload,
        }
    }
}
impl<'a> Serialise for Udp<'a>{
    fn serialise(&self) -> &[u8]
    where Self: Sized {
        unsafe {
            &*slice_from_raw_parts((&*self as *const Self) as *const u8, UDP_HEADER_LEN as usize)
        }
    }
    fn deserialise(raw: &'static [u8], length: usize) -> Option<Self> {
        Some(Self{
            src_port: u16::from_be_bytes(raw[0..2].try_into().unwrap()),
            dst_port: u16::from_be_bytes(raw[2..4].try_into().unwrap()),
            len: u16::from_be_bytes(raw[4..6].try_into().unwrap()),
            checksum: u16::from_be_bytes(raw[6..8].try_into().unwrap()),
            payload: &raw[8..length],
        })
    }
}
impl<'a> Serialise for IPv4<'a>{
    fn serialise(&self) -> &[u8]
    {
        unsafe {
            &*slice_from_raw_parts((&*self as *const Self) as *const u8, IPV4_HEADER_LEN as usize)
        }
    }
    fn deserialise(raw: &'static [u8], length: usize) -> Option<Self> {

        let protocol_type = u8::from_be_bytes(raw[9..10].try_into().unwrap());

        let protocol_data = match protocol_type {
            0x11 => {
                let udp = Udp::deserialise(&raw[ETH_HEADER_LEN+IPV4_HEADER_LEN..], length);
                if let Some(udp) = udp{
                    Protocol::UDP(udp)
                }else{
                    return None
                }
            },
            _=> return None,
        };

        Some(Self {
            version_ihl: u8::from_be_bytes(raw[0..1].try_into().unwrap()), 
            dcp_ecn: u8::from_be_bytes(raw[1..2].try_into().unwrap()), 
            total_len: u16::from_be_bytes(raw[2..4].try_into().unwrap()),
            identification: u16::from_be_bytes(raw[4..6].try_into().unwrap()),
            flags_fragmentoffset: u16::from_be_bytes(raw[6..8].try_into().unwrap()),
            ttl: u8::from_be_bytes(raw[8..9].try_into().unwrap()),
            protocol_type,
            header_checksum: u16::from_be_bytes(raw[10..12].try_into().unwrap()),
            src_ip: raw[12..16].try_into().unwrap(),
            dst_ip: raw[16..20].try_into().unwrap(),
            protocol_data,
        })
    }
}

impl Serialise for Ethernet{
    fn deserialise(raw: &[u8], length: usize) -> Option<Self> {
        Some(Self{
            dst_mac: raw[0..6].try_into().unwrap(),
            src_mac: raw[6..12].try_into().unwrap(),
            ethertype: u16::from_be_bytes(raw[12..14].try_into().unwrap()),
        })
    }
}

impl Serialise for Arp{
    fn deserialise(raw: &'static [u8], length: usize) -> Option<Self>{
        Some(Self {
            htype: u16::from_be_bytes(raw[0..2].try_into().unwrap()),
            ptype: u16::from_be_bytes(raw[2..4].try_into().unwrap()),
            hlen:  raw[4],
            plen:  raw[5],
            oper:  u16::from_be_bytes(raw[6..8].try_into().unwrap()),
            sha:  raw[8..14].try_into().unwrap(),
            spa:  raw[14..18].try_into().unwrap(),
            tha:  raw[18..24].try_into().unwrap(),
            tpa:  raw[24..28].try_into().unwrap(),
        })
    }
}