//! this will deal with everything DHCP
//! 

use super::{nic::NetworkCard, packet::{Packet, EtherType, IPv4, Udp, IPProtocol}};

pub fn init(nic: &NetworkCard){

    let udp = Udp::new(&[0u8]);
    let ipv4 = IPv4::new(IPProtocol::UDP);

    let packet = Packet::new(EtherType::IPv4(ipv4));

}