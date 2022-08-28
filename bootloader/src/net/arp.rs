//! Deals with all things Arp
use super::NetworkStack;
use super::Serialise;
use super::ETHERNET_LEN;
use super::MTU;

const ARP_LEN: usize = 42;

/// This struct is a representation of an ARP Header 
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Arp{
    /// Hardware type
    htype: [u8; 2],
    /// Protocol Address Length
    ptype: [u8; 2],
    /// Hardware Address Length
    hlen: u8,
    /// Protocol Address Length
    plen: u8,
    /// Operation
    oper: [u8; 2],
    /// Sender hardware address
    sha: [u8; 6],
    /// Sender protocol address
    spa: [u8; 4],
    /// Target hardware address
    tha: [u8; 6],
    /// Target protocol address
    tpa: [u8; 4],
}

impl Arp{
    /// Create a new arp packet
    fn new(src_mac: [u8; 6], target_ipv4: [u8; 4]) -> Self{
        Self{
            htype: [0, 1],
            ptype: [8, 0],
            hlen:  0x06,
            plen:  0x04,
            oper:  [0, 1],
            sha:  src_mac,
            spa:  [192,168,1,101], // Hard coded TODO
            tha:  [0xFF; 6],
            tpa:  target_ipv4,
        }
    }

    /// This function sends an arp request to find the Target MAC for a given IP
    pub fn who_has(ns: &NetworkStack, target_ipv4: [u8; 4]) {

        let mut buf = [0u8; MTU];
        
        let arp = Arp::new(ns.nic.mac, target_ipv4);
        let len = arp.serialise(&mut buf);

        ns.nic.send(&buf, len)
    }
}

impl Serialise for Arp{
    fn serialise(&self, buf: &mut [u8]) -> usize {

        // Create an ethernet header
        let eth = super::ethernet::Ethernet::new(
            [0xff,0xff,0xff,0xff,0xff,0xff], 
            self.sha,
            [0x8, 0x6]
        );
        eth.serialise(buf);

        buf[ETHERNET_LEN..16].copy_from_slice(&self.htype);
        buf[16..18].copy_from_slice(&self.ptype);
        buf[18] = self.hlen;
        buf[19] = self.plen;
        buf[20..22].copy_from_slice(&self.oper);
        buf[22..28].copy_from_slice(&self.sha);
        buf[28..32].copy_from_slice(&self.spa);
        buf[32..38].copy_from_slice(&self.tha);
        buf[38..42].copy_from_slice(&self.tpa);

        ARP_LEN
    }
}

