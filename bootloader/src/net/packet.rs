use super::IPv4;
use super::Serialise;
use super::{Arp, ARP_LEN};
use super::{Ethernet, ETHERNET_LEN};

/// Ethernet Ether Type Identifier
const ETH_ETHER_TYPE: [u8; 2] = [0x08, 0x06];
/// IPv4 Ether Type Identifier
const IPV4_ETHER_TYPE: [u8; 2] = [0x08, 0x00];

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Packet {
    ethernet: Ethernet,
    pub ether_type: EtherType,
    len: usize,
}

impl Packet {
    /// Takes raw buffer from recv of NIC and turns into human readable packet
    pub fn parse(buf: &[u8], len: usize) -> Option<Self> {
        //crate::serial_print!("Recieved Packet, Len: {}, Data: {:?}\n", len,
        // &buf[..len]);
        let ethernet = Ethernet::deserialise(&buf[..ETHERNET_LEN]);
        crate::serial_print!("{ethernet:?}\n");
        // The ethernet header tells us what type of packet it is, and we parse
        // accordingly
        let ether_type = match ethernet.ethertype {
            ETH_ETHER_TYPE => EtherType::Arp(Arp::deserialise(
                &buf[ETHERNET_LEN..ETHERNET_LEN + ARP_LEN],
            )),
            IPV4_ETHER_TYPE => {
                EtherType::IPv4(IPv4::deserialise(&buf[ETHERNET_LEN..len]))
            }
            _ => EtherType::Unknown,
        };

        Some(Self {
            ethernet,
            ether_type,
            len,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EtherType {
    Arp(Arp),
    IPv4(IPv4),
    Unknown,
}
