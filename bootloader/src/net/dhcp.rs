//! this will deal with everything DHCP
//! 
const DHCP_OPTIONS_LEN: usize = 118; // Max allowed size
pub const DHCP_TOTAL_LEN: usize = DHCP_OPTIONS_LEN + 240; // 240 is the size before options
const TRANSACTION_ID: u32 = 0x13371338;

const MAGIC: u32 = 0x63825363;
const OPTIONS: [u8; DHCP_OPTIONS_LEN] = [
    0x35,0x01,0x01, // Request
    0x39,0x02,0x05,0xc0, //
    0x37,0x02,
    0x01,0x42,

    // 0x3d,0x07,0x01,0x52,0x54,0x00,0x12,0x34,0x56, // Client ID

    //0x61,0x11,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00, // Client ID Identifier - Required
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,
    0xff // End
];

const OPTIONSREQUEST: [u8; DHCP_OPTIONS_LEN] = [
    0x35,0x01,0x03, // Request
    0x39,0x02,0x05,0xc0, //
    0x37,0x02,
    0x01,0x42,

    0x32,0x04,0x0a,0x63,0x63,0x0b,


    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,
    0xff // End
];

use super::Serialise;

use super::nic::NetworkCard;
use super::{
    packet::{Packet, EtherType, IPv4, Udp, Protocol}, 
    MAC
};

#[derive(Debug, Clone, Copy)]
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
    yiaddr: [u8; 4],
    siaddr: u32,
    giaddr: u32,
    chaddr: [u8; 6],
    _chaddr_padding: [u8; 10],
    _bootp_padding: [u8; 192],
    magic: u32,
    options: [u8; DHCP_OPTIONS_LEN]
}

impl DHCP {
    fn new(state: DhcpState) -> Self {

        let options = match state {
            DhcpState::Discover => {
                OPTIONS
            },
            DhcpState::Request => {
                OPTIONSREQUEST
            },
            _=> unreachable!(),
        };

        Self {
            op: 0x1,
            htype: 0x1,
            hlen: 0x6,
            hops: 0,
            xid: TRANSACTION_ID.to_be(),
            secs: (4u16).to_be(),
            flags: 0,
            ciaddr: 0,
            yiaddr: [0u8; 4],
            siaddr: 0,
            giaddr: 0,
            chaddr: MAC,
            _chaddr_padding: [0u8; 10],
            _bootp_padding: [0u8; 192],
            magic: MAGIC.to_be(),
            options,
        }
    }    
}

impl Serialise for DHCP{
    fn serialise(&self) -> &'static [u8] {
        unsafe {
            &*core::ptr::slice_from_raw_parts((&*self as *const Self) as *const u8, DHCP_TOTAL_LEN)
        }
    }
    fn deserialise(raw: &[u8]) -> Option<Self> {
        let mut options = [0u8; DHCP_OPTIONS_LEN];
        options[0..raw.len() - 240].clone_from_slice(&raw[240..raw.len()]);

        Some(Self{
            op: raw[0],
            htype: raw[1],
            hlen: raw[2],
            hops: raw[3],
            xid: u32::from_be_bytes(raw[4..8].try_into().unwrap()),
            secs: u16::from_be_bytes(raw[8..10].try_into().unwrap()),
            flags: u16::from_be_bytes(raw[10..12].try_into().unwrap()),
            ciaddr: u32::from_be_bytes(raw[12..16].try_into().unwrap()),
            yiaddr: raw[16..20].try_into().unwrap(),
            siaddr: u32::from_be_bytes(raw[20..24].try_into().unwrap()),
            giaddr: u32::from_be_bytes(raw[24..28].try_into().unwrap()),
            chaddr: raw[28..34].try_into().unwrap(),
            _chaddr_padding: raw[34..44].try_into().unwrap(),
            _bootp_padding: raw[44..236].try_into().unwrap(),
            magic: u32::from_be_bytes(raw[236..240].try_into().unwrap()),
            options,
        })
    }
}

pub struct Deamon{
    state: DhcpState,
    nic: NetworkCard, 
}

impl Deamon{
    pub fn new(nic: NetworkCard) -> Self{
        Self{
            state: DhcpState::Uninitiated,
            nic,
        }
    }
    
    pub fn update(&mut self, data: Option<&'static [u8]>) {    
        match self.state {
            DhcpState::Uninitiated => {
                let dhcp = DHCP::new(DhcpState::Discover);
                let udp = Udp::new(dhcp.serialise());
                let ipv4 = IPv4::new(Protocol::UDP(udp));
                let packet = Packet::new(EtherType::IPv4(ipv4));
                self.nic.send(&packet);
                self.state = DhcpState::Discover;
            },
            DhcpState::Discover => {
                if let Some(d) = data{
                    let dhcp = DHCP::deserialise(d).unwrap();
                    // Confirm its a DHCP Offer
                    if dhcp.options[0..3] != [0x35, 0x01, 0x02] { return }
                    // Confirm its our transaction
                    if dhcp.xid != TRANSACTION_ID { return }
                    
                    // Send out the request
                    let dhcp = DHCP::new(DhcpState::Request);
                    let udp = Udp::new(dhcp.serialise());
                    let ipv4 = IPv4::new(Protocol::UDP(udp));
                    let packet = Packet::new(EtherType::IPv4(ipv4));
                    self.nic.send(&packet);

                    self.state = DhcpState::Request;
                }
            },
            DhcpState::Request => {
                if let Some(d) = data{
                    let dhcp = DHCP::deserialise(d).unwrap();
                    // Confirm its a DHCP Ack
                    if dhcp.options[0..3] != [0x35, 0x01, 0x05] { return }
                    // Confirm its our transaction
                    if dhcp.xid != TRANSACTION_ID { return }
                    unsafe { super::IP_ADDR = dhcp.yiaddr };
                    self.state = DhcpState::Acknowledged;
                }
            },
            DhcpState::Acknowledged => {
               
               // self.state = DhcpState::Acknowledged;
            },
        }
    }
}

enum DhcpState{
    Uninitiated,
    Discover,
    Request,
    Acknowledged,
}