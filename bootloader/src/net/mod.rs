//! We manage all things network in this module, this exposes the networking
//! functionality to the other OS use cases
mod apps;
mod error;
mod nic;
mod packet;

use packet::Packet;

use packet::{Arp, Protocol};

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

		packets.into_iter().filter(Option::is_some).for_each(|p| {
			match p.unwrap().protocol {
				Protocol::Ipv4(_) => {}
				Protocol::Arp(arp) => {
					let mut buf = [0u8; MTU];
					arp.handle(&mut buf);
					self.nic.send(&buf, MTU);
				}
				Protocol::Unsupported => {}
			}
		});

		//crate::serial_print!("{p:?}\n");
	}
}

/// This trait will be responsible for turning our human readable
/// structs into packet buffers when can send to the NIC
trait Serialise {
	fn serialise(&self, buffer: &mut [u8]) -> usize {
		todo!()
	}

	fn deserialise(buffer: &[u8]) -> Self
	where
		Self: Sized,
	{
		todo!()
	}

	fn try_deserialise(buffer: &[u8]) -> Option<Self>
	where
		Self: Sized,
	{
		todo!()
	}
}

/// Convience function for getting fixed size arrays from slices
fn slice_to_array<const N: usize>(sli: &[u8]) -> [u8; N] {
	let mut arr = [0u8; N];
	arr.copy_from_slice(sli);
	arr
}
