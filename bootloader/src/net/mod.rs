//! We manage all things network in this module, this exposes the networking
//! functionality to the other OS use cases
mod error;
mod nic;
mod packet;

use packet::Packet;

/// Maximum packet size we deal with, this is a mut ref to a buffer we pass
/// around to create our raw packet for sending to the NIC
const MTU: usize = 1500;

pub struct NetworkStack {
	nic: nic::NetworkCard,
	ip_addr: [u8; 4],
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
					ip_addr: [0, 0, 0, 0],
				})
			}
			Err(e) => {
				crate::serial_print!("Cannot init network: {:X?}", e);
				None
			}
		}
	}

	pub fn update(&mut self) {
		let mut packets = Default::default();
		self.nic.receive(&mut packets);
	}
}

/// This trait will be responsible for turning our human readable
/// structs into packet buffers when can send to the NIC
trait Serialise {
	fn serialise(&self, buf: &mut [u8]) -> usize;

	fn deserialise(buf: &[u8]) -> Self;
}
