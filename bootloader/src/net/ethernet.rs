//! We deal with all things Ethernet in this module
use super::Serialise;

/// Ethernet header is always the same len
const ETHERNET_LEN: u16 = 14;

#[derive(Debug, Clone, Copy)]
pub struct Ethernet{
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    ethertype: [u8; 2],
}

impl Ethernet{
    /// Creates a representation of an ethernet header
    pub fn new(dst_mac: [u8; 6], src_mac: [u8; 6], ethertype: [u8; 2]) -> Self{
        Self{
            dst_mac,
            src_mac,
            ethertype
        }
    }
}

impl Serialise for Ethernet{
    fn serialise(&self, buf: &mut [u8]) -> u16 {
        buf[..6].copy_from_slice(&self.dst_mac);
        buf[6..12].copy_from_slice(&self.src_mac);
        buf[12..14].copy_from_slice(&self.ethertype);

        ETHERNET_LEN
    }
}