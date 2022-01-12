//! We manage all things network in this module, this exposes the networking functionality to the other OS use cases
mod nic;
mod packet;
mod tftp;
mod dhcp;
mod arp;
use packet::{EtherType, Protocol};

/// This is a temporary way of exposing our MAC, will change this in future
const MAC: [u8; 6] = [0x52,0x54,0x00,0x12,0x34,0x56];
/// Where we currently hold our IP address, thinking of moving this into [`NetworkCard`]
static mut IP_ADDR: [u8; 4] = [0u8; 4];

/// This is a trait we use in net to turn structs to bytes and back again.
trait Serialise{
    fn serialise(&self) -> &'static [u8] 
        where Self: Sized{
        unsafe {
            &*core::ptr::slice_from_raw_parts((&*self as *const Self) as *const u8, core::mem::size_of::<Self>())
        }
    }

    fn deserialise(raw: &'static [u8]) -> Option<Self> 
    where Self: Sized{
        todo!();
    }
}

/// Finds all the network cards on the system then uses the first one, we currently only support E1000 NIC's
pub fn init(){

    let nic = nic::init().expect("Cant init Network");
    let mut dhcp_daemon = dhcp::Daemon::new(nic);

    loop {
        dhcp_daemon.update(None);
        
        //nic.send(Packet::new(EtherType::Arp(Arp::new())));
        let packets = &nic.receive();
    
        let tftp_init = tftp::init();
        crate::serial_print!("{:X?}", tftp_init);
        nic.send(&tftp_init);
        for packet in packets{
            if let Some(p) = packet{    
                match p.ethertype{
                    EtherType::IPv4(ipv4) => {
                        crate::serial_print!("Found IPv4: ");
                        match ipv4.protocol_data{
                            Protocol::UDP(udp) => {
                                if udp.payload.len() >= 240 && (&udp.payload[236..240] == &dhcp::DHCP::MAGIC){
                                    crate::serial_print!("DHCP!\n");
                                    dhcp_daemon.update(Some(&udp.payload));
                                    unsafe { crate::serial_print!("IP Addr: {:?}\n", IP_ADDR); }
                                }else{
                                    crate::serial_print!("UDP!\n");
                                }
                            }
                        }
                    },
                    EtherType::Arp(_) => {
                        crate::serial_print!("Found ARP\n");
                    },
                    _=> {}
                }
            }
        }    
        crate::time::sleep(3);
    }
}
