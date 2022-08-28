//! We manage all things network in this module, this exposes the networking functionality to the other OS use cases
mod nic;
mod packet;
mod arp;
mod dhcp;
mod ethernet;

use arp::Arp;
use ethernet::ETHERNET_LEN;

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
    pub fn update(&self) {

        Arp::who_has(&self, [192,168,10,1]);
        

    }

    // pub fn dhcp_init(&self){
    //     //let res = Dhcp::discover();


    // }
}


/// This trait will be responsible for turning our human readable
/// structs into packet buffers when can send to the NIC
trait Serialise{
    fn serialise(&self, buf: &mut [u8]) -> usize;
}