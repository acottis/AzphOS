use super::{slice_to_array, Serialise};

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
		buffer[2..=3].copy_from_slice(&<[u8; 2]>::from(self.protocol_type));
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
	pub const SIZE: usize = 28 + Ethernet::SIZE;
	const REQUEST: [u8; 2] = [0, 1];
	const REPLY: [u8; 2] = [0, 2];
	const HW_TYPE_ETHERNET: [u8; 2] = [0, 1];

	pub fn new(opcode: [u8; 2]) -> Self {
		Self {
			hardware_type: Self::HW_TYPE_ETHERNET,
			protocol_type: EtherType::Arp,
			hardware_size: Ethernet::MAC_LEN as u8,
			protocol_size: Ipv4::LEN as u8,
			opcode,
			sender_mac: Default::default(),
			sender_ip: Default::default(),
			target_mac: Default::default(),
			target_ip: Default::default(),
		}
	}

	pub fn handle(&self, buffer: &mut [u8]) {
		match self.opcode {
			Self::REQUEST => {
				let res = Self::new(Self::REPLY);
				res.serialise(buffer);
			}
			_ => {}
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

impl From<EtherType> for [u8; 2] {
	fn from(value: EtherType) -> Self {
		match value {
			EtherType::Arp => [0x08, 0x06],
			EtherType::Ipv4 => [0x08, 0x00],
			EtherType::Unsupported => [0xFF, 0xFF],
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
}

#[derive(Debug)]
pub struct Packet {
	ethernet: Ethernet,
	pub protocol: Protocol,
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
}
