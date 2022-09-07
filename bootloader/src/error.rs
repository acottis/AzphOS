//! This module will provide a custom error handler so we can provide more
//! meaningful errors
pub type Result<T> = core::result::Result<T, self::Error>;

#[derive(Debug)]
pub enum Error {
	/// Found a NIC but it is not one we have a driver for
	UnsupportedNIC((u16, u16)),

	/// No PCI network card found
	NoNICFound,
	//// We have not implemented this network protocol
	// UnsupportedEtherType(u16),
}
