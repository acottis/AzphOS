use super::Serialise;
use super::Arp;
use super::Ethernet;
use super::MTU;
use super::IPv4;

#[derive(Clone, Copy, Debug)]
pub struct Packet {
    ethernet: Ethernet,
    pub ether_type: EtherType,
}

impl Packet {
    /// Takes raw buffer from recv of NIC and turns into human readable packet
    pub fn parse(buf: &[u8; MTU], len: usize) -> Option<Self> {
        //crate::serial_print!("Recieved Packet, Len: {}, Data: {:?}\n", len, &buf[..len]);
        let ethernet = Ethernet::deserialise(&buf[..14]);
        // The ethernet header tells us what type of packet it is, and we parse
        // accordingly
        let ether_type = match ethernet.ethertype {
            [0x8, 0x6] => EtherType::Arp(Arp::deserialise(&buf[..42])),
            _ => {
                //crate::serial_print!("Unknown Ether Type\n");
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