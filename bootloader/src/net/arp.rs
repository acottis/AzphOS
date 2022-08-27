//! Deals with all things Arp
use super::nic::NetworkCard;

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
    /// Create a new arp packet
    pub fn new(nic: &NetworkCard, target_ipv4: [u8; 4]) -> Self{
        Self{
            htype: 0x0100,
            ptype: 0x0008,
            hlen:  0x06,
            plen:  0x04,
            oper:  0x0100,
            sha:  nic.mac,
            spa:  [192,168,1,101], // Hard coded TODO
            tha:  [0x00; 6],
            tpa:  target_ipv4,
        }
    }
}

impl super::Serialise for Arp{
    fn serialise(&self, buf: &mut [u8]) -> u16 {

    
        buf[..6].copy_from_slice(&[0xff,0xff,0xff,0xff,0xff,0xff]);
        buf[6..12].copy_from_slice(&self.sha);
        buf[12..14].copy_from_slice(&[0x8, 0x6]);
        buf[14..42].copy_from_slice(
            &[
            0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 
            0x01, 0x00, 0xff, 0xbe, 0xd5, 0x22, 0x05, 
            0xc0, 0xa8, 0x0a, 0x01, 0x52, 0x54, 0x00, 
            0x12, 0x34, 0x56, 0xc0, 0xa8, 0x0a, 0x65]);
        // buf[0] = self.htype as u8;
        // buf[1] = (self.htype << 8) as u8;
        42
    }
}

