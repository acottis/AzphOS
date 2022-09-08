//! Deals with all things Arp

use crate::serial_print;

use super::NetworkStack;
use super::Serialise;
use super::MTU;

pub const ARP_LEN: usize = 28;
pub const ETHERNET_LEN: usize = 14;

/// This struct is a representation of an ARP Header
#[derive(Debug, Clone, Copy)]
pub struct Arp {
	/// Hardware type
	htype: [u8; 2],
	/// Protocol Address Length
	ptype: [u8; 2],
	/// Hardware Address Length
	hlen: u8,
	/// Protocol Address Length
	plen: u8,
	/// Operation
	oper: [u8; 2],
	/// Sender hardware address
	sha: [u8; 6],
	/// Sender protocol address
	spa: [u8; 4],
	/// Target hardware address
	tha: [u8; 6],
	/// Target protocol address
	tpa: [u8; 4],
}

impl Arp {
	/// Create a new arp packet
	pub fn new(
		opcode: [u8; 2],
		src_mac: [u8; 6],
		target_mac: [u8; 6],
		src_ipv4: [u8; 4],
		target_ipv4: [u8; 4],
	) -> Self {
		Self {
			htype: [0, 1],
			ptype: [8, 0],
			hlen: 0x06,
			plen: 0x04,
			oper: opcode,
			sha: src_mac,
			spa: src_ipv4,
			tha: target_mac,
			tpa: target_ipv4,
		}
	}

	/// This function sends an arp request to find the Target MAC for a given IP
	fn who_has(ns: &NetworkStack, target_ipv4: [u8; 4]) {
		let mut buf = [0u8; MTU];

		let arp =
			Arp::new([0, 1], ns.nic.mac, [0xFFu8; 6], ns.ip_addr, target_ipv4);
		let len = arp.serialise(&mut buf);

		ns.nic.send(&buf, len)
	}
	/// This function sends out an ARP saying we own an IP when asked
	fn reply(&self, ns: &NetworkStack) {
		let mut buf = [0u8; MTU];

		let reply = Arp::new([0, 2], ns.nic.mac, self.sha, ns.ip_addr, self.spa);
		let len = reply.serialise(&mut buf);

		ns.nic.send(&buf, len)
	}
	/// This function updates the arp table when we recieve ARP packets
	fn update_arp_table(&self, ns: &mut NetworkStack) {
		if self.sha == [0u8; 6] { return }
		let mut first_free_index: Option<usize> = None;
		for (i, (sha, spa)) in ns.arp_table.iter_mut().enumerate() {
			if *spa == self.spa{
				if *sha == self.sha { 
					// if IP and Hardware address are in already do nothing
					return 
				}else{
					// Update the MAC for the IP address
					*sha = self.sha;
					return
				}
			}
			if *spa == [0u8; 4] && first_free_index.is_none() {
				first_free_index = Some(i)
			};
		}
		// If it was not found/updated enter in first empty slot
		if let Some(i) = first_free_index{
			ns.arp_table[i] = (self.sha, self.spa)
		};
	}

	/// This function deals with any arp work required
	pub fn update(self, ns: &mut NetworkStack) {
		// Update our arp table with any new information
		self.update_arp_table(ns);

		// If we see a request for our IP, reply
		if self.tpa == ns.ip_addr {
			self.reply(ns);
		}
	}
}

impl Serialise for Arp {
	fn serialise(&self, buf: &mut [u8]) -> usize {
		// Create an ethernet header
		let eth = super::Ethernet::new(
			[0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
			self.sha,
			[0x8, 0x6],
		);
		eth.serialise(&mut buf[..ETHERNET_LEN]);

		buf[ETHERNET_LEN + 0..ETHERNET_LEN + 2].copy_from_slice(&self.htype);
		buf[ETHERNET_LEN + 2..ETHERNET_LEN + 4].copy_from_slice(&self.ptype);
		buf[ETHERNET_LEN + 4] = self.hlen;
		buf[ETHERNET_LEN + 5] = self.plen;
		buf[ETHERNET_LEN + 6..ETHERNET_LEN + 8].copy_from_slice(&self.oper);
		buf[ETHERNET_LEN + 8..ETHERNET_LEN + 14].copy_from_slice(&self.sha);
		buf[ETHERNET_LEN + 14..ETHERNET_LEN + 18].copy_from_slice(&self.spa);
		buf[ETHERNET_LEN + 18..ETHERNET_LEN + 24].copy_from_slice(&self.tha);
		buf[ETHERNET_LEN + 24..ETHERNET_LEN + 28].copy_from_slice(&self.tpa);

		ARP_LEN
	}

	fn deserialise(buf: &[u8]) -> Self {
		let mut htype = [0u8; 2];
		let mut ptype = [0u8; 2];
		let mut oper = [0u8; 2];
		let mut sha = [0u8; 6];
		let mut spa = [0u8; 4];
		let mut tha = [0u8; 6];
		let mut tpa = [0u8; 4];

		htype.copy_from_slice(&buf[..2]);
		ptype.copy_from_slice(&buf[2..4]);
		let hlen = buf[4];
		let plen = buf[6];
		oper.copy_from_slice(&buf[6..8]);
		sha.copy_from_slice(&buf[8..14]);
		spa.copy_from_slice(&buf[14..18]);
		tha.copy_from_slice(&buf[18..24]);
		tpa.copy_from_slice(&buf[24..ARP_LEN]);

		Self {
			htype,
			ptype,
			hlen,
			plen,
			oper,
			sha,
			spa,
			tha,
			tpa,
		}
	}
}
