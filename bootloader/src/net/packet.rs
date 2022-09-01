use super::{Arp, ARP_LEN};
use super::{ETHERNET_LEN, Ethernet};
use super::{IPv4, IPV4_HEADER_LEN};
use super::Serialise;

#[derive(Clone, Copy, Debug)]
pub struct Packet {
    ethernet: Ethernet,
    pub ether_type: EtherType,
}

impl Packet {
    /// Takes raw buffer from recv of NIC and turns into human readable packet
    pub fn parse(buf: &[u8], len: usize) -> Option<Self> {
        //crate::serial_print!("Recieved Packet, Len: {}, Data: {:?}\n", len, &buf[..len]);
        let ethernet = Ethernet::deserialise(&buf[..ETHERNET_LEN]);
        // The ethernet header tells us what type of packet it is, and we parse
        // accordingly
        let ether_type = match ethernet.ethertype {
            [0x80, 0x60] => EtherType::Arp(Arp::deserialise(&buf[ETHERNET_LEN..ETHERNET_LEN+ARP_LEN])),
            [0x08, 0x00] => EtherType::IPv4(IPv4::deserialise(&buf[ETHERNET_LEN..ETHERNET_LEN+IPV4_HEADER_LEN])),
            _ => {
                EtherType::Unknown
            }
        };

        Some(Self {
            ethernet,
            ether_type,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EtherType {
    Arp(Arp),
    IPv4(IPv4),
    Unknown,
}
