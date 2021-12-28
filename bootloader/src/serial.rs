pub struct SerialPort;

impl SerialPort{
    
    pub fn init(){
        
        for offset in 0..1{
            let val = unsafe { core::ptr::read((0x400+(offset*16) as u16) as *const u16) };
            
            if val != 0{
                crate::print!("Port found\n");
                crate::print!("Port found\n");
            }else{
                crate::print!("Port not found\n");
            }
        }
    }
    
}