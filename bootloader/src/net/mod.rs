//! We manage all things network in this module, this exposes the networking
//! functionality to the other OS use cases
mod arp;
mod error;
mod ethernet;
mod ip;
mod nic;
mod packet;

use arp::{Arp, ARP_LEN};
use error::{Error, Result};
use ethernet::{Ethernet, ETHERNET_LEN};
use ip::dhcp::{Dhcp, MessageType, Status};
use ip::{IPv4, Protocol};
use packet::{EtherType, Packet};

/// Maximum packet size we deal with, this is a mut ref to a buffer we pass
/// around to create our raw packet for sending to the NIC
const MTU: usize = 1500;

pub struct NetworkStack {
    nic: nic::NetworkCard,
    arp_table: [([u8; 6], [u8; 4]); 10],
    ip_addr: [u8; 4],
    /// State machine for DHCP
    dhcp_status: Status,
}

impl NetworkStack {
    /// We start our network stack, we create a NIC if we have a valid driver
    /// available Then we look for an IPv4 Address
    pub fn init() -> Option<Self> {
        match nic::init() {
            Ok(nic) => {
                // Once we have a NIC we can use, we need an IPv4 Address
                Some(Self {
                    nic,
                    arp_table: Default::default(),
                    ip_addr: [0, 0, 0, 0],
                    dhcp_status: Status::NeedIP,
                })
            }
            Err(e) => {
                crate::serial_print!("Cannot init network: {:X?}", e);
                None
            }
        }
    }
    /// This will process all network related tasks during the main OS loop
    /// Here be dragons!
    pub fn update(&mut self) {
        // If our state is that we need an IP, start the DHCP process
        match self.dhcp_status {
            Status::NeedIP => {
                Dhcp::discover(&self.nic);
                self.dhcp_status = Status::DiscoverSent;
            }
            _ => {}
        }

        let packets = self.nic.receive();
        for packet in packets {
            if let Some(packet) = packet {
                match packet.ether_type {
                    // Handle Arp packets
                    EtherType::Arp(arp) => {
                        // If we recieve an Arp we process it, replying to
                        // requests and updating the arp table
                        arp.update(&self);
                    }
                    // Handle IPv4 packets
                    EtherType::IPv4(ipv4) => match ipv4.protocol {
                        Protocol::Udp(udp) => {
                            if udp.dst_port == 68 {
                                let dhcp =
                                    Dhcp::parse(&udp.data[..udp.len as usize])
                                        .unwrap();
                                match dhcp.msg_type {
                                    MessageType::Offer => {
                                        dhcp.request(&self.nic);
                                        self.dhcp_status = Status::RequestSent
                                    }
                                    MessageType::Ack => {
                                        self.ip_addr = dhcp.yiaddr;
                                        self.dhcp_status = Status::Acquired;
                                        crate::serial_print!("IP Addr: {:?}, Recieved from {:?}\n", self.ip_addr, dhcp.siaddr);
                                    }
                                    // Ignore anything that is not an Offer or
                                    // Ack
                                    _ => {}
                                }
                            }
                        }
                    },
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
