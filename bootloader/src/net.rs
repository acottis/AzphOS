//! This crate is responsible for all things networking
//! WIP

use crate::print;

#[derive(Debug)]
struct RSDPDescripter{
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RSDPDescripter{
    /// This searches for the RSDP Descripter and if it finds it builds a Struct [`RSDPDescripter`] with the fields
    /// 
    fn new() -> Self{
        let ebda = unsafe{ core::ptr::read(0x040E as *const u16) as u32 };

        let ranges: [core::ops::Range<u32>; 2] = [(ebda..ebda+1024) , (0xE0000..0xFFFFF)];
        let mut rsdp: usize = 0;

        'outer: for range in ranges{
            for line in range.step_by(0x10){
                let b = get_mem(line as usize, 16);
                if &b[..8] == b"RSD PTR "{
                    rsdp = line as usize;
                    break 'outer;
                }
            }
        }
        // Validate we found a rsdp
        if rsdp == 0 { panic!("No RDSP Found") }

        let raw = get_mem(rsdp, core::mem::size_of::<RSDPDescripter>());

        let mut sum: usize = 0;
        for r in raw{
            sum += *r as usize;
        };
        // Validate the RSDP is valid, the sum of all elements must end in a 0
        if sum % 10 != 0 { panic!("RDSP Checksum failed, Sum: {:#X}", sum) }

        let mut signature = [0u8; 8];
        signature.copy_from_slice(&raw[..8]);
        let mut oem_id = [0u8; 6];
        oem_id.copy_from_slice(&raw[9..15]);
        let mut rsdt_address = [0u8; 4];
        rsdt_address.copy_from_slice(&raw[16..20]);

        Self{
            signature,
            checksum: raw[8],
            oem_id,
            revision: raw[15],
            rsdt_address: u32::from_le_bytes(rsdt_address),
        }
    }
}

/// This table is found after we get the pointer to it from [`RSDPDescripter`]
/// 
#[derive(Debug)]
struct RSDT{
    signature: [u8; 4],
    length: [u8; 4],
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: [u8; 4],
    creator_id: [u8; 4],
    creator_revision: [u8; 4],
    entries: &'static [u32],
}

impl RSDT{

    /// Takes a pointer to the RSDT and returns a struct with all fields
    fn new(ptr: usize) -> Self{

        let rdst_size = core::mem::size_of::<RSDT>() - core::mem::size_of::<&[u8]>();

        let raw = get_mem(ptr as usize, rdst_size);

        let mut signature = [0u8; 4];
        signature.copy_from_slice(&raw[..4]);

        let mut length = [0u8; 4];
        length.copy_from_slice(&raw[4..8]);

        let mut oem_id = [0u8; 6];
        oem_id.copy_from_slice(&raw[10..16]);

        let mut oem_table_id = [0u8; 8];
        oem_table_id.copy_from_slice(&raw[16..24]);

        let mut oem_revision = [0u8; 4];
        oem_revision.copy_from_slice(&raw[24..28]);
      
        let mut creator_id = [0u8; 4];
        creator_id.copy_from_slice(&raw[28..32]);

        let mut creator_revision = [0u8; 4];
        creator_revision.copy_from_slice(&raw[32..36]);

        // This gets the 32bi addresses from the bottom of the table, their as 9 entries in the table so we subtact 9
        // To get length we first convert length which in an array of [u8;4] to a u32 
        let entries = get_mem32(ptr + rdst_size, (u32::from_le_bytes(length) / 4 - 9) as usize);
        
        Self{
            signature,
            length,
            revision: raw[8],
            checksum: raw[9],
            oem_id,
            oem_table_id,
            oem_revision,
            creator_id,
            creator_revision,
            entries,
        }
    }
}

impl core::fmt::Display for RSDT{
    
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::str::from_utf8;
        write!(f,   
            "Signiture: {}\nlength: {}\nRevision: {:#X}\n\
            Checksum: {:#X}\nOEMID: {}\nOEM Table ID: {}\n\
            OEM Revision: {}\nCreator ID: {}\nCreator Revision: {}\n\
            Entries: {:X?}", 
            from_utf8(&self.signature).unwrap(),
            u32::from_le_bytes(self.length) / 4,
            self.revision,
            self.checksum,
            from_utf8(&self.oem_id).unwrap(),
            from_utf8(&self.oem_table_id).unwrap(),
            u32::from_le_bytes(self.oem_revision),
            from_utf8(&self.creator_id).unwrap(),
            u32::from_le_bytes(self.creator_revision),
            self.entries,
        )
    }
}

struct Pci;

fn get_mem(addr: usize, len: usize) -> &'static [u8]{
    unsafe{
        &*core::ptr::slice_from_raw_parts(addr as *const u8, len)
    }
}

fn get_mem32(addr: usize, len: usize) -> &'static [u32]{
    unsafe{
        &*core::ptr::slice_from_raw_parts(addr as *const u32, len)
    }
}

pub fn init(){
    let rsdp_des = RSDPDescripter::new();
    
    let rsdt = RSDT::new(rsdp_des.rsdt_address as usize);
    print!("{}\n", &rsdt);
}