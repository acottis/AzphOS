//! Here we deal with all things DHCP, and publish a service [`Deamon`]

use super::Protocol;
use super::Serialise;
use super::Udp;
use super::MTU;

/// DHCP Magic number to signal this is a DHCP packet
const DHCP_MAGIC: [u8; 4] = [99, 130, 83, 99];
/// Hard coded transaction ID - Should randomise
const TRANSACTION_ID: [u8; 4] = [0x13, 0x37, 0x13, 0x37];
/// The opcode for a boot request
const BOOT_REQUEST: u8 = 1;
/// Hardware type ethernet
const ETHERNET: u8 = 1;

/// This struct represents a DHCP payload of [`DHCP::PAYLOAD_LEN`] size which is
/// fixed due to contraint on knowing size to serialise
#[derive(Debug)]
pub struct Dhcp {
    op: u8,
    htype: u8,
    hlen: u8,
    hops: u8,
    xid: [u8; 4],
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
    options: [Option<Options<'static>>; 10],
}

impl Dhcp {
    fn new(src_mac: [u8; 6], msg_type: MessageType) -> Self {
        Self {
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
            msg_type,
            options: [None; 10],
        }
    }

    pub fn discover(nic: &super::super::nic::NetworkCard) {
        let mut discover = Dhcp::new(nic.mac, MessageType::Discover);

        let opts = [
            Some(Options::MessageType(MessageType::Discover)),
            Some(Options::End),
        ];

        discover.options[..opts.len()].copy_from_slice(&opts);

        let mut buf = [0u8; MTU];
        let len = discover.serialise(&mut buf);

        nic.send(&mut buf, len)
    }
}

impl Serialise for Dhcp {
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

        let mut payload = [0u8; 300];
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

        // Set DHCP Options
        let mut dhcp_ptr = 240;
        // For every option we want
        for opt in self.options {
            if let Some(opt) = opt {
                // Allocate a buffer we can pass down to default evil rust!
                let mut tmp_buf = [0u8; 50];
                // Take the length so we can dynamically push on our option
                let len = opt.serialise(&mut tmp_buf);
                // Copy the option serialised into the UDP data
                payload[dhcp_ptr..dhcp_ptr + len]
                    .copy_from_slice(&tmp_buf[..len]);
                // Increment the UDP data len
                dhcp_ptr = dhcp_ptr + len;
            } else {
                break;
            }
        }

        // Create the UDP struct so we can pass to IPv4, IPv4 needs to know
        // total packet len
        let udp = Udp::new(dhcp_ptr);

        // Create an IPv4 header
        let ipv4 = super::IPv4::new(Protocol::Udp(udp));
        let ipv4_len = ipv4.serialise(&mut buf[packet_size..]);
        packet_size += ipv4_len;

        let udp_len = udp.serialise(&mut buf[packet_size..]);
        packet_size += udp_len;

        buf[packet_size..packet_size + dhcp_ptr]
            .copy_from_slice(&payload[..dhcp_ptr]);
        packet_size += dhcp_ptr;

        packet_size
    }

    fn deserialise(buf: &[u8]) -> Self {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    Ack = 5,
    Nak = 6,
    Release = 7,
    Inform = 8,
}

#[derive(Clone, Copy, Debug)]
pub enum Options<'opt> {
    SubnetMask([u8; 4]),
    HostName(&'opt str),
    RequestedIPAddr([u8; 4]),
    LeaseTime(u32),
    MessageType(MessageType),
    ServerIP([u8; 4]),
    ParameterRequestList([u8; 50]),
    MaxDhcpMessageSize(u16),
    ClientIdentifier(u8, [u8; 6]),
    TftpServer(&'opt str),
    BootFile(&'opt str),
    ClientSystemArch(u16),
    ClientNetInterfaceIdent((u8, u8)),
    ClientMachineIdent(u8),
    TftpServerIP([u8; 4]),
    End,
}

impl Options<'_> {
    fn opcode(&self) -> u8 {
        match self {
            Self::SubnetMask(_) => 1,
            Self::HostName(_) => 12,
            Self::RequestedIPAddr(_) => 50,
            Self::LeaseTime(_) => 51,
            Self::MessageType(_) => 53,
            Self::ServerIP(_) => 54,
            Self::ParameterRequestList(_) => 55,
            Self::MaxDhcpMessageSize(_) => 57,
            Self::ClientIdentifier(_, _) => 61,
            Self::TftpServer(_) => 66,
            Self::BootFile(_) => 67,
            Self::ClientSystemArch(_) => 93,
            Self::ClientNetInterfaceIdent(_) => 94,
            Self::ClientMachineIdent(_) => 97,
            Self::TftpServerIP(_) => 150,
            Self::End => 255,
        }
    }
}

impl Serialise for Options<'_> {
    fn serialise(&self, tmp_buf: &mut [u8]) -> usize {
        tmp_buf[0] = self.opcode();
        match self {
            Self::MessageType(msg) => {
                let len: usize = 3;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2] = *msg as u8;
                len
            }
            Self::ServerIP(addr) => {
                let len: usize = 6;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2..6].copy_from_slice(addr);
                len
            }
            Self::TftpServer(addr) => {
                let len: usize = addr.len() + 2;
                tmp_buf[1] = addr.len() as u8;
                tmp_buf[2..2 + addr.len()].copy_from_slice(addr.as_bytes());
                len
            }
            Self::BootFile(file_path) => {
                let len: usize = file_path.len() + 2;
                tmp_buf[1] = file_path.len() as u8;
                tmp_buf[2..2 + file_path.len()]
                    .copy_from_slice(file_path.as_bytes());
                len
            }
            Self::LeaseTime(time) => {
                let len: usize = 6;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2] = (time >> 24) as u8;
                tmp_buf[3] = (time >> 16) as u8;
                tmp_buf[4] = (time >> 8) as u8;
                tmp_buf[5] = *time as u8;
                len
            }
            Self::SubnetMask(addr) => {
                let len: usize = 6;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2..6].copy_from_slice(addr);
                len
            }
            Self::ClientIdentifier(_, _) => 0,
            Self::ParameterRequestList(_) => 0,
            Self::MaxDhcpMessageSize(_) => 0,
            Self::RequestedIPAddr(_) => 0,
            Self::HostName(_) => 0,
            Self::ClientSystemArch(num) => {
                let len: usize = 4;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2] = (num << 8) as u8;
                tmp_buf[3] = *num as u8;
                len
            }
            Self::ClientNetInterfaceIdent((major, minor)) => {
                let len: usize = 5;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2] = 1;
                tmp_buf[3] = *major;
                tmp_buf[4] = *minor;
                len
            }
            Self::ClientMachineIdent(num) => {
                let len: usize = 19;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2] = *num;
                len
            }
            Self::TftpServerIP(addr) => {
                let len: usize = 6;
                tmp_buf[1] = len as u8 - 2;
                tmp_buf[2..6].copy_from_slice(addr);
                len
            }
            Self::End => 1,
        }
    }

    fn deserialise(buf: &[u8]) -> Self {
        todo!()
    }
}
