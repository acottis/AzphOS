//! We manage all things network in this module, this exposes the networking functionality to the other OS use cases
mod arp;
mod ethernet;
mod ip;
mod nic;
mod packet;

use arp::{Arp, ARP_LEN};
use ethernet::{Ethernet, ETHERNET_LEN};
use ip::dhcp::Dhcp;
use ip::{IPv4, Protocol};
use packet::{EtherType, Packet};

/// Maximum packet size we deal with, this is a mut ref to a buffer we pass around to create
/// our raw packet for sending to the NIC
const MTU: usize = 1500;

pub struct NetworkStack {
    nic: nic::NetworkCard,
    arp_table: [([u8; 6], [u8; 4]); 10],
    ip_addr: [u8; 4],
}

impl NetworkStack {
    /// We start our network stack, we create a NIC if we have a valid driver available
    /// Then we look for an IPv4 Address
    pub fn init() -> Option<Self> {
        match nic::init() {
            Ok(nic) => {
                // Once we have a NIC we can use, we need an IPv4 Address
                Dhcp::discover(&nic);
                loop {
                    let packets = nic.receive();
                    for packet in packets {
                        if let Some(packet) = packet {
                            match packet.ether_type {
                                EtherType::IPv4(ipv4) => {
                                    match ipv4.protocol {
                                        Protocol::Udp(udp) => {
                                            if udp.dst_port == 68{
                                                crate::serial_print!("Recieved DHCP!\n");
                                                let dhcp = Dhcp::deserialise(&udp.data[..udp.len as usize]);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                Some(Self {
                    nic,
                    arp_table: Default::default(),
                    // Hard coding for now!!!!!
                    ip_addr: [192, 168, 10, 101],
                })
            }
            Err(e) => {
                crate::serial_print!("Cannot init network: {:X?}", e);
                None
            }
        }
    }
    /// This will process all network related tasks during the main OS loop
    pub fn update(&self) {
        let packets = self.nic.receive();
        for packet in packets {
            if let Some(packet) = packet {
                match packet.ether_type {
                    EtherType::Arp(arp) => {
                        // If we recieve an Arp we process it, replying to requests and updating
                        // the arp table
                        arp.update(&self);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// This trait will be responsible for turning our human readable
/// structs into packet buffers when can send to the NIC
trait Serialise {
    fn serialise(&self, buf: &mut [u8]) -> usize;

    fn deserialise(buf: &[u8]) -> Self;
}
