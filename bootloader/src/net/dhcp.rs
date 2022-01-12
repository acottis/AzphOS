//! Here we deal with all things DHCP, and publish a service [`Deamon`]
//!
use super::Serialise;
use super::nic::NetworkCard;
use super::{
    packet::{Packet, EtherType, IPv4, Udp, Protocol}, 
    MAC
};

/// This struct represents a DHCP payload of [`DHCP::PAYLOAD_LEN`] size which is fixed due to contraint on knowing size to serialise
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
    magic: [u8;4],
    options: [u8; Self::OPTIONS_LEN]
}

impl DHCP {
    /// We need to know this for our packet send function as we need to know size at runtime and we have no allocator
    pub const PAYLOAD_LEN: usize = Self::OPTIONS_LEN + Self::HEADER_SIZE;
    /// Size of settings befor options, always fixed len
    const HEADER_SIZE: usize = 240;
    /// Arbitary length of our options field, we pad to this.
    const OPTIONS_LEN: usize = 64;
    /// Magic that proves it is DHCP packet not BOOTP
    pub const MAGIC: [u8;4] = [0x63,0x82,0x53,0x63];
    /// We have a static Transaction ID, this is group our conversation with DHCP together
    const TRANSACTION_ID: u32 = 0x13371338;
    /// We have static options for our Discover
    const OPTIONS: [u8; Self::OPTIONS_LEN] = [
        0x35,0x01,0x01, // Request
        0x39,0x02,0x05,0xc0, //
        0x37,0x02,
        0x01,0x42,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0xff // End
    ];
    /// We have static options for our Request
    const OPTIONSREQUEST: [u8; Self::OPTIONS_LEN] = [
        0x35,0x01,0x03, // Request
        0x39,0x02,0x05,0xc0, //
        0x37,0x02,
        0x01,0x42,
        0x32,0x04,0x0a,0x63,0x63,0x0b, // This is the IP we want
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0xff // End
    ];
    /// Creates a new DHCP packet, chooses different options if Discover or Request
    fn new(state: DhcpState) -> Self {
        let options = match state {
            DhcpState::Discover => {
                DHCP::OPTIONS
            },
            DhcpState::Request => {
                DHCP::OPTIONSREQUEST
            },
            _=> unreachable!(),
        };

        Self {
            op: 0x1,
            htype: 0x1,
            hlen: 0x6,
            hops: 0,
            xid: Self::TRANSACTION_ID.to_be(),
            secs: (4u16).to_be(),
            flags: 0,
            ciaddr: 0,
            yiaddr: [0u8; 4],
            siaddr: 0,
            giaddr: 0,
            chaddr: MAC,
            _chaddr_padding: [0u8; 10],
            _bootp_padding: [0u8; 192],
            magic: Self::MAGIC,
            options,
        }
    }    
}
impl Serialise for DHCP{
    fn serialise(&self) -> &'static [u8] {
        unsafe {
            &*core::ptr::slice_from_raw_parts((&*self as *const Self) as *const u8, Self::PAYLOAD_LEN)
        }
    }
    fn deserialise(raw: &[u8]) -> Option<Self> {
        let mut options = [0u8; Self::OPTIONS_LEN];
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
            magic: raw[236..240].try_into().unwrap(),
            options,
        })
    }
}
/// This struct acts as a service for DHCP that responds based on what state we are in
/// TODO is to add the lease time and renewal
/// Also Make it aware of our IP state.
/// 
pub struct Daemon{
    state: DhcpState,
    nic: NetworkCard, 
}

impl Daemon{
    /// Init our DHCP service
    pub fn new(nic: NetworkCard) -> Self{
        Self{
            state: DhcpState::Uninitiated,
            nic,
        }
    }
    /// Main event loop for our DHCP that handles based on [`DhcpState`]
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
                    if dhcp.xid != DHCP::TRANSACTION_ID { return }
                    
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
                    if dhcp.xid != DHCP::TRANSACTION_ID { return }
                    unsafe { super::IP_ADDR = dhcp.yiaddr };
                    self.state = DhcpState::Acknowledged;
                }
            },
            DhcpState::Acknowledged => {
            },
        }
    }
}
/// This is the 4 different states of DHCP that we care about
enum DhcpState{
    Uninitiated,
    Discover,
    Request,
    Acknowledged,
}