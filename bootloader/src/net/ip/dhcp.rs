//! Here we deal with all things DHCP, and publish a service [`Deamon`]
//!
use crate::serial_print;

use super::Serialise;
use super::Udp;
use super::MTU;
use super::Protocol;

/// DHCP Magic number to signal this is a DHCP packet
const DHCP_MAGIC: [u8; 4] = [99, 130, 83, 99];
/// Hard coded transaction ID - Should randomise
const TRANSACTION_ID: [u8; 4] = [0x13, 0x37, 0x13, 0x37];
/// The opcode for a boot request
const BOOT_REQUEST: u8 = 1;
/// Hardware type ethernet
const ETHERNET: u8 = 1;

/// This struct represents a DHCP payload of [`DHCP::PAYLOAD_LEN`] size which is fixed due to contraint on knowing size to serialise
#[derive(Debug)]
pub struct Dhcp{
    op: u8,
    htype: u8,
    hlen: u8,
    hops: u8,
    xid: [u8;4],
    secs: [u8; 2],
    flags: [u8; 2],
    ciaddr: [u8; 4],
    yiaddr: [u8; 4],
    siaddr: [u8; 4],
    giaddr: [u8; 4],
    chaddr: [u8; 6],
    sname: [u8; 64],
    file: [u8; 128],
    magic: [u8; 4],
    msg_type: MessageType,
    //options: [Option<Options>; 20],
}

impl Dhcp{
    fn new(src_mac: [u8;6]) -> Self{
        Self{
            op: BOOT_REQUEST,
            htype: ETHERNET,
            hlen: 6,
            hops: 0,
            xid: TRANSACTION_ID,
            secs: [0u8; 2],
            flags: [0u8; 2],
            ciaddr: [0u8; 4],
            yiaddr: [0u8; 4],
            siaddr: [0u8; 4],
            giaddr: [0u8; 4],
            chaddr: src_mac,
            sname: [0u8; 64],
            file: [0u8; 128],
            magic: DHCP_MAGIC,
            msg_type: MessageType::Discover,
        }        
    }

    pub fn discover(nic: &super::super::nic::NetworkCard) {
        let discover = Dhcp::new(nic.mac);

        let mut buf = [0u8; MTU];
        let len = discover.serialise(&mut buf);

        nic.send(&mut buf, len)
    }
}

impl Serialise for Dhcp{
    fn serialise(&self, buf: &mut [u8]) -> usize {
        let mut packet_size = 0;
        // Create an ethernet header
        let eth = super::Ethernet::new(
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            self.chaddr,
            [0x08, 0x00],
        );
        let eth_len = eth.serialise(buf);
        packet_size += eth_len;

        let mut payload = [0u8; 240];
        payload[0] = self.op; // op
        payload[1] = self.htype; // hytpe
        payload[2] = self.hlen; // hlen
        payload[3] = self.hops; // hops
        payload[4..8].copy_from_slice(&self.xid); // Client ID
        payload[8..10].copy_from_slice(&self.secs); // Seconds
        payload[10..12].copy_from_slice(&self.flags); // Bootp flags
        payload[12..16].copy_from_slice(&self.ciaddr); // Client IP
        payload[16..20].copy_from_slice(&self.yiaddr); // Yiaddr
        payload[20..24].copy_from_slice(&self.siaddr); // Our Server IP
        payload[24..28].copy_from_slice(&self.giaddr); // Relay IP
        payload[28..34].copy_from_slice(&self.chaddr); // Requester MAC
        payload[44..108].copy_from_slice(&self.sname); // Unused
        payload[108..236].copy_from_slice(&self.file); // Unused
        payload[236..240].copy_from_slice(&self.magic); // DHCP Magic bytes
        // Create the UDP struct so we can pass to IPv4, IPv4 needs to know total packet len
        let udp = Udp::new(240);

        // Create an IPv4 header
        let ipv4 = super::IPv4::new(Protocol::Udp(udp));
        let ipv4_len = ipv4.serialise(&mut buf[packet_size..]);
        packet_size += ipv4_len;

        let udp_len = udp.serialise(&mut buf[packet_size..]);
        packet_size += udp_len;

        buf[packet_size..packet_size+240].copy_from_slice(&payload[..240]);
        packet_size += 240;

        packet_size
    }

    fn deserialise(buf: &[u8]) -> Self {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MessageType {
    Discover    = 1,
    Offer       = 2,
    Request     = 3,
    Decline     = 4,
    Ack         = 5,
    Nak         = 6,
    Release     = 7,
    Inform      = 8,
}

impl TryFrom<u8> for MessageType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => { Ok(Self::Discover) },
            2 => { Ok(Self::Offer) },
            3 => { Ok(Self::Request) },
            4 => { Ok(Self::Decline) },
            5 => { Ok(Self::Ack) },
            6 => { Ok(Self::Nak) },
            7 => { Ok(Self::Release) },
            8 => { Ok(Self::Inform) },
            t => { Err(())}
        }
    }
}
