//! This is the place where we define network packets and parse to and from
//! 

use core::mem::size_of;
use core::ptr::{read, write, slice_from_raw_parts, read_unaligned};
use crate::net::dhcp::DHCP_TOTAL_LEN;

use super::MAC;

use super::Serialise;

const IPV4_HEADER_LEN: u16 = 20;
const UPD_HEADER_LEN: u16 = 8;
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
            EtherType::Arp(arp) => Ethernet::new([0xFF; 6], MAC, 0x0608),
            EtherType::IPv4(ipv4) => Ethernet::new([0xFF; 6], MAC, 0x0008),        
        };

        Self{
            ethernet,
            ethertype,
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
                })
            },
            // Arp
            0x0608 => {
                Some(Self{
                    ethernet,
                    ethertype: EtherType::Arp(Arp::headers(buffer_address)),
                })
            },
            // IPv6
            0xDD86 =>{ None },
            _ => { None },
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
                        // let arp = &*slice_from_raw_parts(
                        //     (&arp as *const Arp) as *const u8, 
                        //     size_of::<Arp>());
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
                                writer_ptr as *mut [u8; UPD_HEADER_LEN as usize], 
                                udp.serialise().try_into().unwrap()
                            );
                            writer_ptr += UPD_HEADER_LEN as u64;

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

    fn parse(start_address: u64) -> Self {
        unsafe{
            read(start_address as *const Self)
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
            sha:  crate::net::MAC,
            spa:  [0u8; 4],
            tha:  [0x00; 6],
            tpa:  [0xA, 0x63, 0x63, 0x01],
        }
    }
}

impl ParsePacket for Arp{}

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
    protocol_data: Protocol<'a>,
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
            total_len: (IPV4_HEADER_LEN + len).to_be(),
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

impl<'a> ParsePacket for IPv4<'a>{}

impl Serialise for Arp{}

// #[repr(u8)]
// #[derive(Debug, Clone, Copy)]
// pub enum IPProtocol{
//     UDP = 0x11,
// }

/// This trait allows us to generically handle getting the headers and data for IPv4 and ARP (ARP Doesnt have data though...?)
trait ParsePacket {
    /// Starts reading the packet from the end point of the ethernet header for the length of the EtherType
    fn headers<T>(start_address: u64) -> T{
        unsafe{
            read_unaligned((start_address+(size_of::<Ethernet>()) as u64) as *const T)
        }
    }  
    /// Reads the extra bytes at the end of the packets headers
    fn data<T>(start_address: u64, length: u16) -> &'static [u8]{
        let data_len = length as u16 - (size_of::<Ethernet>() as u16 + size_of::<T>() as u16);
        unsafe{
            &*slice_from_raw_parts(
                (start_address + 
                size_of::<Ethernet>() as u64 + 
                size_of::<T>() as u64) as *const u8, data_len as usize)
        }
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
            len: (payload.len() as u16 + UPD_HEADER_LEN).to_be(),
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
            &*core::ptr::slice_from_raw_parts((&*self as *const Self) as *const u8, UPD_HEADER_LEN as usize)
        }
    }
}

impl<'a> Serialise for IPv4<'a>{
    fn serialise(&self) -> &[u8]
    where Self: Sized {
        unsafe {
            &*core::ptr::slice_from_raw_parts((&*self as *const Self) as *const u8, IPV4_HEADER_LEN as usize)
        }
    }
}

impl Serialise for Ethernet{}