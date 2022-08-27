//! We manage all things network in this module, this exposes the networking functionality to the other OS use cases
mod nic;
mod packet;
mod arp;
mod dhcp;

use arp::Arp;
use dhcp::Dhcp;

pub struct NetworkStack{
    nic: nic::NetworkCard
}

impl NetworkStack {
    pub fn init() -> Option<Self> {
        match nic::init() {
            Ok(nic) => Some(Self {
                nic
            }),
            Err(e) => {
                crate::serial_print!("Cannot init network: {:X?}", e);
                None
            }
        } 
    }
    /// This will process all network related tasks during the main OS loop
    pub fn update(&self) {
        self.send_arp();
        self.dhcp_init();
    }

    pub fn send_arp(&self){
        let arp = Arp::new(&self.nic, [192,168,10,1]);
        
        let mut buf = [0u8; 42];
        let len = arp.serialise(&mut buf);
        self.nic.send(&buf, len)
    }

    pub fn dhcp_init(&self){
        //let res = Dhcp::discover();


    }
}


/// This trait will be responsible for turning our human readable
/// structs into packet buffers when can send to the NIC
trait Serialise{
    fn serialise(&self, buf: &mut [u8]) -> u16;
}