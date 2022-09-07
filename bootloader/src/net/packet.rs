use super::arp::{Arp, ARP_LEN};
use super::IPv4;
use super::NetworkStack;
use super::Serialise;
use super::MTU;
use super::{
	Ethernet, Protocol, ETHERNET_LEN, IPV4_HEADER_LEN, UDP_HEADER_LEN,
};
use super::{ETH_ETHER_TYPE, IPV4_ETHER_TYPE};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Packet {
	ethernet: Ethernet,
	pub ether_type: EtherType,
	len: usize,
	pub data: Option<[u8; 1458]>,
}

impl Packet {
	/// Takes raw buffer from recv of NIC and turns into human readable packet
	pub fn parse(buf: &[u8], len: usize) -> Option<Self> {
		//crate::serial_print!("Recieved Packet, Len: {}, Data: {:?}\n", len,
		// &buf[..len]);
		let ethernet = Ethernet::deserialise(&buf[..ETHERNET_LEN]);
		// Initialise data as None
		let mut data = None;
		// The ethernet header tells us what type of packet it is, and we parse
		// accordingly
		let ether_type = match ethernet.ethertype {
			ETH_ETHER_TYPE => EtherType::Arp(Arp::deserialise(
				&buf[ETHERNET_LEN..ETHERNET_LEN + ARP_LEN],
			)),
			IPV4_ETHER_TYPE => {
				let ip = IPv4::deserialise(&buf[ETHERNET_LEN..]);
				match ip.protocol {
					Protocol::Udp(udp) => {
						let mut tmp = [0u8; 1458];
						tmp[..(udp.len as usize - 8)].copy_from_slice(
							&buf[ETHERNET_LEN + IPV4_HEADER_LEN + UDP_HEADER_LEN..len],
						);
						data = Some(tmp)
					}
				}
				EtherType::IPv4(ip)
			}
			_ => EtherType::None,
		};

		Some(Self {
			ethernet,
			ether_type,
			len,
			data,
		})
	}
	/// Creates a new [Packet] up to and including L3
	pub fn send(
		ns: &NetworkStack,
		ether_type_opcode: [u8; 2],
		dst_ip: [u8; 4],
		dst_port: u16,
		data: &[u8],
	) {
		// if dst_ip not in ns.arp_table { do logic! } TODO
		// TODO TODO!!!!!!!!!!!!!!!!!!!!
		let dst_mac = [0xFF; 6];

		// Track the size of our packet
		let mut packet_size = 0;
		// Creat buffer that we serialise too
		let mut buf = [0u8; MTU];

		// ETHENET SERIALISE
		// Create out ethernet header with the given opcode
		let ethernet = Ethernet::new(dst_mac, ns.nic.mac, ether_type_opcode);
		let ethernet_len = ethernet.serialise(&mut buf);
		packet_size += ethernet_len;

		// Create the UDP struct so we can pass to IPv4, IPv4 needs to know
		// total packet len
		let udp = super::Udp::new(data.len(), 30000, dst_port);

		// Create an IPv4 header
		let ipv4 = super::IPv4::new(super::Protocol::Udp(udp), dst_ip);

		// SERIALISE IPV4
		let ipv4_len = ipv4.serialise(&mut buf[packet_size..]);
		packet_size += ipv4_len;

		let udp_len = udp.serialise(&mut buf[packet_size..]);
		packet_size += udp_len;

		buf[packet_size..packet_size + data.len()]
			.copy_from_slice(&data[..data.len()]);
		packet_size += data.len();

		ns.nic.send(&buf, packet_size)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum EtherType {
	Arp(Arp),
	IPv4(IPv4),
	None,
}
