pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	InvalidDhcpPacket,
	BadDhcpMessageType(u8),
	DestIPNotInArpTable([u8; 4]),
}
