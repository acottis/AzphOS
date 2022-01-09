pub mod nic;
pub mod packet;
mod dhcp;


use packet::{EtherType, Protocol};
use nic::NetworkCard;

const MAC: [u8; 6] = [0x52,0x54,0x00,0x12,0x34,0x56];

static mut IP_ADDR: [u8; 4] = [0u8; 4];

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

pub fn init(){

    let nic = nic::init().expect("Cant init Network");

    let mut dhcp_daemon = dhcp::Deamon::new(nic);

    loop {
        dhcp_daemon.update(None);
        
        //nic.send(Packet::new(EtherType::Arp(Arp::new())));
        let packets = &nic.receive();
    
        for packet in packets{
            if let Some(p) = packet{    
                match p.ethertype{
                    EtherType::IPv4(ipv4) => {
                        crate::serial_print!("Found IPv4: ");
                        match ipv4.protocol_data{
                            Protocol::UDP(udp) => {
                                //crate::serial_print!("{:02X?}\n", udp);
                                if udp.payload.len() >= 240 && (&udp.payload[236..240] == &[0x63,0x82,0x53,0x63]){
                                    crate::serial_print!("DHCP!\n");
                                    dhcp_daemon.update(Some(&udp.payload))
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
        crate::time::sleep(4);
    }
}
