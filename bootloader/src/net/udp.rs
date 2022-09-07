use super::Serialise;

/// The size of UDP header
pub const UDP_HEADER_LEN: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct Udp {
	src_port: u16,
	pub dst_port: u16,
	pub len: u16,
	checksum: u16,
}

impl Udp {
	pub fn new(len: usize, src_port: u16, dst_port: u16) -> Self {
		Self {
			src_port,
			dst_port,
			len: (len + UDP_HEADER_LEN) as u16,
			// Unimplemented
			checksum: 0,
		}
	}
}

impl Serialise for Udp {
	fn serialise(&self, buf: &mut [u8]) -> usize {
		buf[0] = (self.src_port << 8) as u8;
		buf[1] = self.src_port as u8;
		buf[2] = (self.dst_port << 8) as u8;
		buf[3] = self.dst_port as u8;
		buf[4] = (self.len << 8) as u8;
		buf[5] = self.len as u8;
		buf[6] = (self.checksum << 8) as u8;
		buf[7] = self.checksum as u8;

		UDP_HEADER_LEN
	}

	fn deserialise(buf: &[u8]) -> Self {
		let len = (buf[4] as u16) << 8 | buf[5] as u16;

		Self {
			src_port: (buf[0] as u16) << 8 | buf[1] as u16,
			dst_port: (buf[2] as u16) << 8 | buf[3] as u16,
			len,
			checksum: (buf[6] as u16) << 8 | buf[7] as u16,
		}
	}
}
