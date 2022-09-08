//! We manage all things network in this module, this exposes the networking
//! functionality to the other OS use cases
mod error;
mod ethernet;
mod ip;
mod nic;
mod packet;
mod udp;

mod apps;

use apps::arp;
use apps::dhcp;
use error::{Error, Result};
use ethernet::{Ethernet, ETHERNET_LEN};
use ip::{IPv4, Protocol, IPV4_HEADER_LEN};
use packet::{EtherType, Packet};
use udp::{Udp, UDP_HEADER_LEN};

use crate::serial_print;

/// Maximum packet size we deal with, this is a mut ref to a buffer we pass
/// around to create our raw packet for sending to the NIC
const MTU: usize = 1500;
/// DHCP UDP Port number we listen on
const DHCP_PORT_CLIENT: u16 = 68;
/// DHCP UDP Port number we listen on
const MAC_LEN: usize = 6;
/// Ethernet Ether Type Identifier
pub const ETH_ETHER_TYPE: [u8; 2] = [0x08, 0x06];
/// IPv4 Ether Type Identifier
pub const IPV4_ETHER_TYPE: [u8; 2] = [0x08, 0x00];

pub struct NetworkStack {
	nic: nic::NetworkCard,
	arp_table: [([u8; MAC_LEN], [u8; 4]); 10],
	// IPs to send to ARP
	requested_ips: [[u8; 4]; 5],
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
				let mut requested_ips = [[0u8; 4]; 5];
				requested_ips[0] = [0xff; 4];
				Some(Self {
					nic,
					arp_table: Default::default(),
					requested_ips: [[0u8; 4]; 5],
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
							dhcp::update(
								self,
								Some(&packet.data.unwrap()[..udp.len as usize]),
							);
						}
					}
				},
				_ => {}
			}
		}
	}
	fn request_ip(&mut self, ip_addr: [u8; 4]) {
		// Tidy up old ones
		for (_, arp_ip) in self.arp_table {
			for req_ip in self.requested_ips.iter_mut() {
				if arp_ip == *req_ip {
					*req_ip = [0u8; 4]
				}
			}
		}
		// Insert new requested IP
		for ip in self.requested_ips.iter_mut() {
			if *ip == [0u8; 4] {
				*ip = ip_addr
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
