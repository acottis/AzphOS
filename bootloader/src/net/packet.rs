use super::{slice_to_array, NetworkStack, Serialise};

#[derive(Debug)]
pub struct Ipv4;

impl Ipv4 {
	const LEN: usize = 4;
}

#[derive(Debug)]
pub struct Arp {
	hardware_type: [u8; 2],
	protocol_type: EtherType,
	hardware_size: u8,
	protocol_size: u8,
	opcode: [u8; 2],
	sender_mac: [u8; 6],
	sender_ip: [u8; 4],
	target_mac: [u8; 6],
	target_ip: [u8; 4],
}
impl Serialise for Arp {
	fn try_deserialise(buffer: &[u8]) -> Option<Arp> {
		if buffer.len() < 28 {
			return None;
		}

		Some(Arp {
			hardware_type: [buffer[0], buffer[1]],
			protocol_type: slice_to_array(&buffer[2..=3]).into(),
			hardware_size: buffer[4],
			protocol_size: buffer[5],
			opcode: [buffer[6], buffer[7]],
			sender_mac: slice_to_array(&buffer[8..=13]),
			sender_ip: slice_to_array(&buffer[14..=17]),
			target_mac: slice_to_array(&buffer[18..=23]),
			target_ip: slice_to_array(&buffer[24..=27]),
		})
	}

	fn serialise(&self, buffer: &mut [u8]) -> usize {
		buffer[0..=1].copy_from_slice(&self.hardware_type);
		self.protocol_type.serialise(&mut buffer[2..=3]);
		buffer[4] = self.hardware_size;
		buffer[5] = self.protocol_size;
		buffer[6..=7].copy_from_slice(&self.opcode);
		buffer[8..=13].copy_from_slice(&self.sender_mac);
		buffer[14..=17].copy_from_slice(&self.sender_ip);
		buffer[18..=23].copy_from_slice(&self.target_mac);
		buffer[24..=27].copy_from_slice(&self.target_ip);

		Self::SIZE
	}
}

impl Arp {
	const SIZE: usize = 28;
	const REQUEST: [u8; 2] = [0, 1];
	const REPLY: [u8; 2] = [0, 2];
	const HW_TYPE_ETHERNET: [u8; 2] = [0, 1];

	pub fn new(
		sender_mac: &[u8; 6],
		sender_ip: &[u8; 4],
		target_mac: &[u8; 6],
		target_ip: &[u8; 4],
		opcode: [u8; 2],
	) -> Self {
		Self {
			hardware_type: Self::HW_TYPE_ETHERNET,
			protocol_type: EtherType::Ipv4,
			hardware_size: Ethernet::MAC_LEN as u8,
			protocol_size: Ipv4::LEN as u8,
			opcode,
			sender_mac: *sender_mac,
			sender_ip: *sender_ip,
			target_mac: *target_mac,
			target_ip: *target_ip,
		}
	}

	pub fn handle(&self, net: &NetworkStack, buffer: &mut [u8]) -> usize {
		match self.opcode {
			Self::REQUEST => {
				let arp = Self::new(
					&net.nic.mac,
					&net.ip_addr,
					&self.sender_mac,
					&self.sender_ip,
					Self::REPLY,
				);
				let packet = Packet::new(&net.nic.mac, Protocol::Arp(arp));
				packet.serialise(buffer)
			}
			_ => 0,
		}
	}
}

#[derive(Debug)]
pub enum Protocol {
	Ipv4(Ipv4),
	Arp(Arp),
	Unsupported,
}

#[derive(Debug, Clone, Copy)]
enum EtherType {
	Ipv4,
	Arp,
	Unsupported,
}

impl EtherType {
	const SIZE: usize = 2;
}

impl Serialise for EtherType {
	fn serialise(&self, buffer: &mut [u8]) -> usize {
		match self {
			EtherType::Ipv4 => buffer[..Self::SIZE].copy_from_slice(&[0x08, 0x00]),
			EtherType::Arp => buffer[..Self::SIZE].copy_from_slice(&[0x08, 0x06]),
			EtherType::Unsupported => {
				buffer[..Self::SIZE].copy_from_slice(&[0xFF, 0xFF])
			}
		};
		Self::SIZE
	}
}

impl From<[u8; 2]> for EtherType {
	fn from(value: [u8; 2]) -> Self {
		match value {
			[0x08, 0x06] => Self::Arp,
			[0x08, 0x00] => Self::Ipv4,
			unhandled => {
				crate::serial_print!(
					"Unsupported ethertype {:?} recieved\n",
					unhandled
				);
				Self::Unsupported
			}
		}
	}
}

#[derive(Debug)]
struct Ethernet {
	dst: [u8; 6],
	src: [u8; 6],
	r#type: EtherType,
}

impl Ethernet {
	const SIZE: usize = 14;
	const MAC_LEN: usize = 6;
	const IPV4: [u8; 2] = [8, 6];

	fn new(mac: &[u8; 6], r#type: EtherType) -> Self {
		Self {
			dst: [0xFF; 6],
			src: *mac,
			r#type,
		}
	}
}

impl Serialise for Ethernet {
	fn try_deserialise(buffer: &[u8]) -> Option<Self> {
		if buffer.len() < Self::SIZE {
			return None;
		}

		Some(Self {
			dst: slice_to_array(&buffer[..Self::MAC_LEN]),
			src: slice_to_array(
				&buffer[Self::MAC_LEN..Self::MAC_LEN + Self::MAC_LEN],
			),
			r#type: [buffer[12], buffer[13]].into(),
		})
	}

	fn serialise(&self, buffer: &mut [u8]) -> usize {
		buffer[0..Self::MAC_LEN].copy_from_slice(&self.dst);
		buffer[Self::MAC_LEN..Self::MAC_LEN * 2].copy_from_slice(&self.src);
		self.r#type.serialise(&mut buffer[12..=13]);

		Self::SIZE
	}
}

#[derive(Debug)]
pub struct Packet {
	ethernet: Ethernet,
	pub protocol: Protocol,
}
impl Packet {
	fn new(mac: &[u8; 6], protocol: Protocol) -> Self {
		let ether_type = match &protocol {
			Protocol::Ipv4(_) => EtherType::Ipv4,
			Protocol::Arp(_) => EtherType::Arp,
			Protocol::Unsupported => EtherType::Unsupported,
		};

		Self {
			ethernet: Ethernet::new(mac, ether_type),
			protocol,
		}
	}
}

impl Serialise for Packet {
	fn try_deserialise(buffer: &[u8]) -> Option<Self> {
		Ethernet::try_deserialise(&buffer[..Ethernet::SIZE]).map(|ethernet| {
			let protocol = match ethernet.r#type {
				EtherType::Ipv4 => Protocol::Ipv4(Ipv4),
				EtherType::Arp => Protocol::Arp(
					Arp::try_deserialise(&buffer[Ethernet::SIZE..]).unwrap(),
				),
				EtherType::Unsupported => Protocol::Unsupported,
			};
			return Self { ethernet, protocol };
		})
	}

	fn serialise(&self, buffer: &mut [u8]) -> usize {
		let ethernet_len = self.ethernet.serialise(buffer);
		let protocol_len = match &self.protocol {
			Protocol::Ipv4(_) => todo!(),
			Protocol::Arp(arp) => arp.serialise(&mut buffer[Ethernet::SIZE..]),
			Protocol::Unsupported => todo!(),
		};
		ethernet_len + protocol_len
	}
}
