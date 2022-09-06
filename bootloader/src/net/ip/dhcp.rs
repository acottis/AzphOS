//! Here we deal with all things DHCP, and publish a function
//! called [update] that can be used called from main network loop
//! to process DHCP 
use super::{Error, Result};

use super::Protocol;
use super::Serialise;
use super::Udp;
use super::MAC_LEN;
use super::MTU;

/// DHCP Magic number to signal this is a DHCP packet
const DHCP_MAGIC: [u8; 4] = [99, 130, 83, 99];
/// Hard coded transaction ID - Should randomise
const TRANSACTION_ID: [u8; 4] = [0x13, 0x37, 0x13, 0x37];
/// The opcode for a boot request
const BOOT_REQUEST: u8 = 1;
/// Hardware type ethernet
const ETHERNET: u8 = 1;
/// Options array buffer size
const OPTIONS_BUF: usize = 20;

/// This struct is a human readable abstraction of DHCP data
/// found on a UDP packet
#[derive(Debug)]
struct Dhcp<'a> {
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
    chaddr: [u8; MAC_LEN],
    sname: [u8; 64],
    file: [u8; 128],
    magic: [u8; 4],
    msg_type: MessageType,
    options: [Option<Options<'a>>; OPTIONS_BUF],
}

impl<'a> Dhcp<'a> {
    /// Allows the user to create a new [Dhcp] packet
    fn new(src_mac: [u8; MAC_LEN], msg_type: MessageType) -> Self {
        Self {
            op: BOOT_REQUEST,
            htype: ETHERNET,
            hlen: MAC_LEN as u8,
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
            options: [None; OPTIONS_BUF],
        }
    }
    /// Parse a buffer and extract a [Dhcp] struct
    fn parse(buf: &'a [u8]) -> Result<Self> {
        // Not a valid DHCP request
        let data_len = buf.len();
        if data_len < 240 {
            return Err(Error::InvalidDhcpPacket);
        }
        let mut xid: [u8; 4] = [0; 4];
        let mut secs: [u8; 2] = [0; 2];
        let mut flags: [u8; 2] = [0; 2];
        let mut ciaddr: [u8; 4] = [0; 4];
        let mut yiaddr: [u8; 4] = [0; 4];
        let mut siaddr: [u8; 4] = [0; 4];
        let mut giaddr: [u8; 4] = [0; 4];
        let mut chaddr: [u8; MAC_LEN] = [0; MAC_LEN];
        let mut sname: [u8; 64] = [0; 64];
        let mut file: [u8; 128] = [0; 128];
        let mut magic: [u8; DHCP_MAGIC.len()] = [0; DHCP_MAGIC.len()];
        let mut msg_type = None;

        let op = buf[0];
        let htype = buf[1];
        let hlen = buf[2];
        let hops = buf[3];
        xid.copy_from_slice(&buf[4..8]);
        secs.copy_from_slice(&buf[8..10]);
        flags.copy_from_slice(&buf[10..12]);
        ciaddr.copy_from_slice(&buf[12..16]);
        yiaddr.copy_from_slice(&buf[16..20]);
        siaddr.copy_from_slice(&buf[20..24]);
        giaddr.copy_from_slice(&buf[24..28]);
        chaddr.copy_from_slice(&buf[28..34]);
        sname.copy_from_slice(&buf[44..108]);
        file.copy_from_slice(&buf[108..236]);
        magic.copy_from_slice(&buf[236..240]);

        // Not a valid DHCP request
        if magic != DHCP_MAGIC {
            return Err(Error::InvalidDhcpPacket);
        }

        let mut options_counter = 0;
        let mut options: [Option<Options>; OPTIONS_BUF] = [None; OPTIONS_BUF];
        let mut options_ptr = 240;

        loop {
            // End Option, break loop
            if buf[options_ptr] == 255 {
                break;
            }

            // Not enough space to have length in the option
            if options_ptr + 1 > data_len {
                break;
            }

            // Get the next Options len
            let len = buf[options_ptr + 1] as usize;
            let opt_start = options_ptr + 2;
            let opt_end = options_ptr + 2 + len;
            let data = match buf.get(opt_start..opt_end) {
                Some(data) => data,
                // Invalid Options Len
                None => return Err(Error::InvalidDhcpPacket),
            };
            let res: Option<Options> = match &buf[options_ptr] {
                // Host name
                12 => {
                    if let Ok(hostname) = core::str::from_utf8(data) {
                        Some(Options::HostName(hostname))
                    } else {
                        return Err(Error::InvalidDhcpPacket);
                    }
                }
                // Requested IP Address
                50 => {
                    if len < 1 {
                        return Err(Error::InvalidDhcpPacket);
                    }
                    let mut ip_addr: [u8; 4] = [0u8; 4];
                    ip_addr.copy_from_slice(data);
                    Some(Options::RequestedIPAddr(ip_addr))
                }
                // DHCP Message Type
                53 => {
                    if len < 1 {
                        return Err(Error::InvalidDhcpPacket);
                    }
                    if let Ok(m_type) = data[0].try_into() {
                        msg_type = Some(m_type);
                        Some(Options::MessageType(m_type))
                    } else {
                        return Err(Error::InvalidDhcpPacket);
                    }
                }
                // DHCP Requested Parameters
                55 => {
                    if len >= 50 {
                        return Err(Error::InvalidDhcpPacket);
                    }
                    let mut params = [0u8; 50];
                    for (i, param) in data.iter().enumerate() {
                        params[i] = *param;
                    }
                    Some(Options::ParameterRequestList(params))
                }
                // Maximum DHCP Message Size
                57 => {
                    if len < 2 {
                        return Err(Error::InvalidDhcpPacket);
                    }
                    // Think this should only ever be 2 length
                    let sz: u16 = (data[0] as u16) << 8 | data[1] as u16;
                    Some(Options::MaxDhcpMessageSize(sz))
                }
                // DHCP Server Identifier | Pfft we ignore this
                54 => None,
                // Vendor class ID | Pfft we ignore this
                60 => None,
                // Client Identifier (MAC)
                61 => {
                    if len < 7 {
                        return Err(Error::InvalidDhcpPacket);
                    }
                    let hardware_type = data[0];
                    let mut client_mac: [u8; 6] = [0u8; 6];
                    client_mac.copy_from_slice(&data[1..]);
                    Some(Options::ClientIdentifier(hardware_type, client_mac))
                }
                // User Class Information, dont need https://www.rfc-editor.org/rfc/rfc3004
                77 => None,
                // Etherchannel, dont need this?
                175 => None,
                _ => None,
            };
            // Add the parsed option
            if res.is_some() {
                options[options_counter] = res;
                // Increment the number of parsed options
                options_counter += 1;
            }
            // Options PTR increment and increment by len of DHCP Option + 1 as
            // options len doesnt count itself
            options_ptr = options_ptr + 1 + buf[options_ptr + 1] as usize + 1;
        }

        let msg_type = match msg_type {
            Some(msg_type) => msg_type,
            None => return Err(Error::InvalidDhcpPacket),
        };

        Ok(Self {
            op,
            htype,
            hlen,
            hops,
            xid,
            secs,
            flags,
            ciaddr,
            yiaddr,
            siaddr,
            giaddr,
            chaddr,
            sname,
            file,
            magic,
            msg_type,
            options,
        })
    }
    /// This function performs and DHCP Request
    fn request(&self, nic: &super::super::nic::NetworkCard) {
        let mut request = Dhcp::new(nic.mac, MessageType::Request);
        request.xid = self.xid;

        // Add on our options
        let opts = [
            Some(Options::MessageType(MessageType::Request)),
            Some(Options::End),
        ];
        request.options[..opts.len()].copy_from_slice(&opts);

        // Send it!
        let mut buf = [0u8; MTU];
        let len = request.serialise(&mut buf);
        nic.send(&buf, len)
    }
    /// Broadcasts out a DHCP discover to everyone asking for an IP
    fn discover(nic: &super::super::nic::NetworkCard) {
        let mut discover = Dhcp::new(nic.mac, MessageType::Discover);

        let opts = [
            Some(Options::MessageType(MessageType::Discover)),
            Some(Options::End),
        ];

        discover.options[..opts.len()].copy_from_slice(&opts);

        let mut buf = [0u8; MTU];
        let len = discover.serialise(&mut buf);

        nic.send(&buf, len)
    }
}

impl Serialise for Dhcp<'_> {
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
                dhcp_ptr += len;
            } else {
                break;
            }
        }

        // Create the UDP struct so we can pass to IPv4, IPv4 needs to know
        // total packet len
        let udp = Udp::new(dhcp_ptr, 68, 67);

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

    fn deserialise(_: &[u8]) -> Self {
        unimplemented!("See Dhcp::parse()")
    }
}

#[derive(Clone, Copy, Debug)]
enum MessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    Ack = 5,
    Nak = 6,
    Release = 7,
    Inform = 8,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Discover),
            2 => Ok(Self::Offer),
            3 => Ok(Self::Request),
            4 => Ok(Self::Decline),
            5 => Ok(Self::Ack),
            6 => Ok(Self::Nak),
            7 => Ok(Self::Release),
            8 => Ok(Self::Inform),
            t => {
                unimplemented!("We dont handle bad DHCP Msg Types yet")
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Options<'opt> {
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

/// This enum is used to let us state machine way to getting an IP address
#[derive(PartialEq, Eq, Debug)]
pub enum Status {
    NeedIP,
    DiscoverSent,
    RequestSent,
    Acquired,
}

/// User accessible DHCP interface
/// This function handles the DHCP state and transitions as we
/// process DHCP packets
#[inline(always)]
pub fn update(ns: &mut super::super::NetworkStack, data: Option<&[u8]>) {
    // If need an IP send a discover
    if ns.dhcp_status == Status::NeedIP {
        Dhcp::discover(&ns.nic);
        ns.dhcp_status = Status::DiscoverSent;
        return;
    }

    // If we get a UDP packet on port [DHCP_PORT] lets check if any data
    // is in it
    let dhcp = if let Some(data) = data {
        Dhcp::parse(data).unwrap()
    } else {
        // No UDP data provided
        return;
    };
    match dhcp.msg_type {
        MessageType::Offer => {
            dhcp.request(&ns.nic);
            ns.dhcp_status = Status::RequestSent
        }
        MessageType::Ack => {
            ns.ip_addr = dhcp.yiaddr;
            ns.dhcp_status = Status::Acquired;
            crate::serial_print!(
                "IP Addr: {:?}, Recieved from {:?}\n",
                ns.ip_addr,
                dhcp.siaddr
            );
        }
        // Ignore anything that is not an Offer or
        // Ack
        _ => {}
    }
}
