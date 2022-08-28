//! We manage all things network in this module, this exposes the networking functionality to the other OS use cases
mod nic;
mod packet;
mod arp;
mod dhcp;
mod ethernet;

use arp::Arp;
use ethernet::{Ethernet, ETHERNET_LEN};
use packet::{EtherType, Packet};

/// Maximum packet size we deal with, this is a mut ref to a buffer we pass around to create 
/// our raw packet for sending to the NIC
const MTU: usize = 1500;

pub struct NetworkStack{
    nic: nic::NetworkCard,
    arp_table: [([u8;6], [u8; 4]); 10]
}

impl NetworkStack {
    pub fn init() -> Option<Self> {
        match nic::init() {
            Ok(nic) => Some(Self {
                nic,
                arp_table: Default::default()
            }),
            Err(e) => {
                crate::serial_print!("Cannot init network: {:X?}", e);
                None
            }
        } 
    }
    /// This will process all network related tasks during the main OS loop
    pub fn update(&self, asked: &mut bool) {

        // This is ugly and for testing, will be removed
        let target = if *asked == false {
            *asked = true;
            Some([192,168,10,1])
        }else{
            None
        };

        // We call this every network update to check if anything has changed relating to Arp
        // such as a request for more IP addresses or someone on the network wants us to anounce our
        // IP address
        Arp::update(&self, target);
        
        let packets = self.nic.receive();
        for packet in packets{
            if let Some(packet) = packet{
                match packet.ether_type{
                    EtherType::Arp(arp) => {
                        crate::serial_print!("{arp:?}\n");
                    },
                    _ => {},
                }
            }
        }
        // If packets contains a ARP packet
        // Arp::update_table() | Arp::AnnouceIP
    }

    // pub fn dhcp_init(&self){
    //     //let res = Dhcp::discover();


    // }
}


/// This trait will be responsible for turning our human readable
/// structs into packet buffers when can send to the NIC
trait Serialise{
    fn serialise(&self, buf: &mut [u8]) -> usize;

    fn deserialise(buf: &[u8]) -> Self;
}