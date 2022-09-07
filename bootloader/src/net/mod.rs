//! We manage all things network in this module, this exposes the networking
//! functionality to the other OS use cases
mod arp;
mod dhcp;
mod error;
mod ethernet;
mod ip;
mod nic;
mod packet;
mod udp;

use arp::{Arp, ARP_LEN};
use error::{Error, Result};
use ethernet::{Ethernet, ETHERNET_LEN};
use ip::{IPv4, Protocol};
use packet::{EtherType, Packet, IPV4_ETHER_TYPE};
use udp::Udp;

/// Maximum packet size we deal with, this is a mut ref to a buffer we pass
/// around to create our raw packet for sending to the NIC
const MTU: usize = 1500;
/// DHCP UDP Port number we listen on
const DHCP_PORT_SERVER: u16 = 67;
const DHCP_PORT_CLIENT: u16 = 68;
/// DHCP UDP Port number we listen on
const MAC_LEN: usize = 6;

pub struct NetworkStack {
	nic: nic::NetworkCard,
	arp_table: [([u8; MAC_LEN], [u8; 4]); 10],
	ip_addr: [u8; 4],
	/// State machine for DHCP
	dhcp_status: dhcp::Status,
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
					dhcp_status: dhcp::Status::NeedIP,
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
		dhcp::update(self, None);

		// Get the packets from the NIC and handle them before actioning
		// any required packets
		let packets = self.nic.receive();
		for packet in packets.iter().flatten() {
			match packet.ether_type {
				// Handle Arp packets
				EtherType::Arp(ref arp) => {
					// If we recieve an Arp we process it, replying to
					// requests and updating the arp table
					arp.update(self);
				}
				// Handle IPv4 packets
				EtherType::IPv4(ref ipv4) => match ipv4.protocol {
					Protocol::Udp(ref udp) => {
						if udp.dst_port == DHCP_PORT_CLIENT {
							// If we recieve a DHCP packet, send it off to the
							// DHCP Agent to handle
							dhcp::update(self, Some(&udp.data[..udp.len as usize]));
						}
					}
				},
				_ => {}
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
