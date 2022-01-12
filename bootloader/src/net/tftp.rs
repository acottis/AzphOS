//! This module will deal with all things TFTP

use super::Serialise;
use super::{
    packet::{Packet, EtherType, IPv4, Udp, Protocol}, 
    MAC
};


const SRC_FILE: &str = "kernel.bin";

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Tftp{
    opcode: u16,
    data: [u8; 512]
}

impl Tftp{
    fn new() -> Tftp{
        Self {
            opcode: (1 as u16).to_be(),
            data: [0u8; 512],
        }
    }
}

impl Serialise for Tftp{

}

pub fn init() -> Packet {
    let tftp = Tftp::new();
    let udp = Udp::new(tftp.serialise());
    let ipv4 = IPv4::new(Protocol::UDP(udp));
    Packet::new(EtherType::IPv4(ipv4))
}