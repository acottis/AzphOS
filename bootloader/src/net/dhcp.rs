//! this will deal with everything DHCP
//! 
const DHCP_OPTIONS_LEN: usize = 0xB0; // Max allowed size
// pub const DHCP_TOTAL_LEN: usize = DHCP_OPTIONS_LEN + 240; // 240 is the size before options
const MAGIC: u32 = 0x63825363;
const OPTIONS: [u8; DHCP_OPTIONS_LEN] = [
    53, 1, 1, // DHCP Message type = Request
    57, 2, 0x05, 0xC0, // DHCP Max Size = 1472
    0x5d,0x02,0x00,0x00, // Client system architecture
    0x5e,0x03,0x01,0x02,0x01, // Client network device interface
    0x3c,0x20,0x50,0x58,0x45,0x43,0x6c,0x69,0x65,0x6e,0x74,0x3a,0x41,0x72,0x63,0x68,
    0x3a,0x30,0x30,0x30,0x30,0x30,0x3a,0x55,0x4e,0x44,0x49,0x3a,0x30,0x30,0x32,0x30,
    0x30,0x31, // Vendor class information
    0x4d,0x04,0x69,0x50,0x58,0x45, // User class
    55, 23, // DHCP Parameter Request List, length 2
    1, // Subnet Mask
    3, // Router
    6,
    7,
    12,
    15,
    17,
    26,
    43,
    60,
    66,
    67,
    119,
    128,
    129, 130, 131, 132,133,134,135, // WTF?
    175, 203,

    0xaf,0x2d,0xb1,0x05,0x01,0x80,0x86,0x10,0x0e,0xeb,0x03,0x01,0x14,0x01,0x17,0x01,
    0x01, 0x22,0x01,0x01,0x13,0x01,0x01,0x11,0x01,0x01,0x27,0x01,0x01,0x19,0x01,0x01,
    0x10, 0x01,0x02,0x21,0x01,0x01,0x15,0x01,0x01,0x18,0x01,0x01,0x12,0x01,0x01,

    0x3d,0x07,0x01,0x52,0x54,0x00,0x12,0x34,0x56, // Client ID
    97, 17,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // Client UUID
    0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF// End
];

use super::Serialise;

use super::{
    nic::NetworkCard, 
    packet::{Packet, EtherType, IPv4, Udp, Protocol}, 
    MAC
};

pub fn init(nic: &NetworkCard){

    let dhcp = DHCP::new();
    let udp = Udp::new(dhcp.serialise());
    let ipv4 = IPv4::new(Protocol::UDP(udp));

    let packet = Packet::new(EtherType::IPv4(ipv4));

    nic.send(packet);

}

#[repr(C)]
pub struct DHCP{
    op: u8,
    htype: u8,
    hlen: u8,
    hops: u8,
    xid: u32,
    secs: u16,
    flags: u16,
    ciaddr: u32,
    yiaddr: u32,
    siaddr: u32,
    giaddr: u32,
    chaddr: [u8; 6],
    _chaddr_padding: [u8; 10],
    _bootp_padding: [u8; 192],
    magic: u32,
    options: [u8; DHCP_OPTIONS_LEN]
}

impl DHCP {
    fn new() -> Self {
        Self {
            op: 0x1,
            htype: 0x1,
            hlen: 0x6,
            hops: 0,
            xid: (0x13371337 as u32).to_be(),
            secs: 0,
            flags: 0,
            ciaddr: 0,
            yiaddr: 0,
            siaddr: 0,
            giaddr: 0,
            chaddr: MAC,
            _chaddr_padding: [0u8; 10],
            _bootp_padding: [0u8; 192],
            magic: MAGIC.to_be(),
            options: OPTIONS,
        }
    }
}

impl Serialise for DHCP{}
