pub mod nic;
pub mod packet;
mod dhcp;

static MAC: [u8; 6] = [0x11; 6];
static mut IP_ADDR: [u8; 4] = [0u8; 4];