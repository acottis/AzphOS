pub mod nic;
pub mod packet;
mod dhcp;

const MAC: [u8; 6] = [0x52,0x54,0x00,0x12,0x34,0x56];

static mut IP_ADDR: [u8; 4] = [0u8; 4];

trait Serialise{
    fn serialise(&self) -> &[u8] 
        where Self: Sized{
        unsafe {
            &*core::ptr::slice_from_raw_parts((&*self as *const Self) as *const u8, core::mem::size_of::<Self>())
        }
    }

    fn deserialise(raw: &'static [u8], length: usize) -> Option<Self> 
    where Self: Sized{
        todo!();
    }
}
