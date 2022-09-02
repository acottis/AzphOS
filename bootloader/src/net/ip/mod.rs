pub mod dhcp;
mod udp;

use super::Ethernet;
use super::Serialise;
use super::MTU;
use udp::Udp;

/// The size of IPv4 Headers, we dont support ipv4 options
pub const IPV4_HEADER_LEN: usize = 20;

#[derive(Debug, Clone, Copy)]
pub struct IPv4 {
    version_ihl: u8,
    dcp_ecn: u8,
    total_len: u16,
    identification: u16,
    flags_fragmentoffset: u16,
    ttl: u8,
    protocol_type: u8,
    header_checksum: u16,
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    pub protocol: Protocol,
}

impl IPv4 {
    /// Create a new IPv4 header
    pub fn new(protocol: Protocol) -> Self {
        let len = match protocol {
            Protocol::Udp(udp) => udp.len,
        };
        let mut ipv4 = Self {
            version_ihl: 0x45,
            dcp_ecn: 0x00,
            total_len: (IPV4_HEADER_LEN as u16 + len),
            identification: (0x0100u16),
            flags_fragmentoffset: 0x00,
            ttl: 0x40,
            protocol_type: 0x11,
            header_checksum: 0,
            src_ip: [0x0; 4],
            dst_ip: [0xFF; 4],
            protocol,
        };
        ipv4.checksum();
        ipv4
    }
    /// This calculates the IPv4 checksum on creation of the header
    fn checksum(&mut self) {
        let mut raw = [0u8; 20];
        let len = self.serialise(&mut raw);
        let mut total: u32 = 0;
        for index in (0..len).step_by(2) {
            let tmp: u32 = ((raw[index] as u32) << 8) | (raw[index + 1]) as u32;
            total += tmp;
        }
        total = (total + (total >> 16)) & 0x0000FFFF;
        // This catches the wierd edge case where our carry creates another
        // carry
        total = total + (total >> 16);

        self.header_checksum = !total as u16;
    }
}

impl Serialise for IPv4 {
    fn serialise(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.version_ihl;
        buf[1] = self.dcp_ecn;
        buf[2] = (self.total_len >> 8) as u8;
        buf[3] = (self.total_len) as u8;
        buf[4] = (self.identification >> 8) as u8;
        buf[5] = (self.identification) as u8;
        buf[6] = (self.flags_fragmentoffset >> 8) as u8;
        buf[7] = (self.flags_fragmentoffset) as u8;
        buf[8] = self.ttl;
        buf[9] = self.protocol_type;
        buf[10] = (self.header_checksum >> 8) as u8;
        buf[11] = (self.header_checksum) as u8;
        buf[12..16].copy_from_slice(&self.src_ip);
        buf[16..20].copy_from_slice(&self.dst_ip);

        IPV4_HEADER_LEN
    }

    fn deserialise(buf: &[u8]) -> Self {
        let mut src_ip = [0u8; 4];
        let mut dst_ip = [0u8; 4];

        let version_ihl = buf[0];
        let dcp_ecn = buf[1];
        let total_len = (buf[2] as u16) << 8 | buf[3] as u16;
        let identification = (buf[4] as u16) << 8 | buf[5] as u16;
        let flags_fragmentoffset = (buf[6] as u16) << 8 | buf[7] as u16;
        let ttl = buf[8];
        let protocol_type = buf[9];
        let header_checksum = (buf[10] as u16) << 8 | buf[11] as u16;
        src_ip.copy_from_slice(&buf[12..16]);
        dst_ip.copy_from_slice(&buf[16..20]);

        let protocol = match protocol_type {
            0x11 => Protocol::Udp(Udp::deserialise(&buf[IPV4_HEADER_LEN..])),
            _ => unimplemented!(),
        };

        Self {
            version_ihl,
            dcp_ecn,
            total_len,
            identification,
            flags_fragmentoffset,
            ttl,
            protocol_type,
            header_checksum,
            src_ip,
            dst_ip,
            protocol,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Udp(Udp),
}
