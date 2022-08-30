//! Deals with all things Arp

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
    pub fn new(target_ipv4: [u8; 4]) -> Self{
        Self{
            htype: 0x0100,
            ptype: 0x0008,
            hlen:  0x06,
            plen:  0x04,
            oper:  0x0100,
            sha:  super::MAC,
            spa:  unsafe { super::IP_ADDR },
            tha:  [0x00; 6],
            tpa:  target_ipv4,
        }
    }
}

impl super::Serialise for Arp{
    fn deserialise(raw: &[u8]) -> Option<Self>{
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