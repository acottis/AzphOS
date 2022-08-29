// //! Here we deal with all things DHCP, and publish a service [`Deamon`]
// //!
// use super::Serialise;

// /// DHCP Magic number to signal this is a DHCP packet
// const DHCP_MAGIC: [u8; 4] = [99, 130, 83, 99];
// /// The size we give our empty buffers by default, code should truncate to correct size
// const BUFFER_SIZE: usize = 1500;

// /// This struct represents a DHCP payload of [`DHCP::PAYLOAD_LEN`] size which is fixed due to contraint on knowing size to serialise
// #[derive(Debug)]
// pub struct Dhcp<'dhcp>{
//     op: u8,
//     htype: u8,
//     hlen: u8,
//     hops: u8,
//     xid: [u8;4],
//     secs: [u8; 2],
//     flags: [u8; 2],
//     ciaddr: [u8; 4],
//     yiaddr: [u8; 4],
//     siaddr: [u8; 4],
//     giaddr: [u8; 4],
//     chaddr: [u8; 6],
//     sname: [u8; 64],
//     file: [u8; 128],
//     magic: [u8; 4],
//     msg_type: MessageType,
//     //options: [Option<Options<'dhcp>>; 20],
// }

// impl<'dhcp> Dhcp<'dhcp>{
//     pub fn discover() -> Self {

//     }
// }

// impl<'dhcp> Serialise for Dhcp<'dhcp>{
//     fn serialise(&self, buf: &mut [u8]) -> u16 {
//         buf[0] = self.op; // op
//         buf[1] = self.htype; // hytpe
//         buf[2] = self.hlen; // hlen
//         buf[3] = self.hops; // hops
//         buf[4..8].copy_from_slice(&self.xid); // Client ID
//         buf[8..10].copy_from_slice(&self.secs); // Seconds
//         buf[10..12].copy_from_slice(&self.flags); // Bootp flags
//         buf[12..16].copy_from_slice(&self.ciaddr); // Client IP
//         buf[16..20].copy_from_slice(&self.yiaddr); // Yiaddr
//         buf[20..24].copy_from_slice(&self.siaddr); // Our Server IP
//         buf[24..28].copy_from_slice(&self.giaddr); // Relay IP
//         buf[28..34].copy_from_slice(&self.chaddr); // Requester MAC
//         buf[44..108].copy_from_slice(&[0u8; 64]); // Unused
//         buf[108..236].copy_from_slice(&[0u8; 128]); // Unused
//         buf[236..240].copy_from_slice(&DHCP_MAGIC); // DHCP Magic bytes

//         240
//     }
// }

// #[derive(Clone, Copy, Debug)]
// pub enum MessageType {
//     Discover    = 1,
//     Offer       = 2,
//     Request     = 3,
//     Decline     = 4,
//     Ack         = 5,
//     Nak         = 6,
//     Release     = 7,
//     Inform      = 8,
// }

// impl TryFrom<u8> for MessageType {
//     type Error = ();
//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             1 => { Ok(Self::Discover) },
//             2 => { Ok(Self::Offer) },
//             3 => { Ok(Self::Request) },
//             4 => { Ok(Self::Decline) },
//             5 => { Ok(Self::Ack) },
//             6 => { Ok(Self::Nak) },
//             7 => { Ok(Self::Release) },
//             8 => { Ok(Self::Inform) },
//             t => { Err(())}
//         }
//     }
// }
