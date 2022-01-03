//! This is where we enumerate all the PCI devices and find the ones we want to use to expose to other
//! parts of the OS
use crate::cpu;

/// PCI Magic numbers
const PCI_ENABLE_BIT: u32 = 1 << 31;
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;
const PCI_CLASS_CODE_NETWORK: u8 = 0x2;
const PCI_SUBCLASS_CODE_ETHERNET: u8 = 0x0;
const PCI_REGISTER_LEN: usize = 0xF;
const PCI_BUS_LEN: u32 = 256;
const PCI_DEVICE_LEN: u32 = 32;
const PCI_FUNCTION_LEN: u32 = 8;

/// This struct holds the data for a header type 0x0 PCI Device
/// 
#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(dead_code)]
struct Header{
    vendor_id: u16,
    device_id: u16,
    command: u16,
    status: u16,
    revision_id: u8,
    prog_if: u8,
    subclass: u8,
    class_code: u8,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
    base_addr_0: u32,
    base_addr_1: u32,
    base_addr_2: u32,
    base_addr_3: u32,
    base_addr_4: u32,
    base_addr_5: u32,
    cardbus_cis_ptr: u32,
    subsystem_vendor_id: u16,
    subsystem_id: u16,
    expansion_rom_base_addr: u32,
    capabilities_ptr: u8,
    _reserved: [u8; 7],
    interrupt_line: u8,
    interrupt_pin: u8,
    min_grant: u8,
    max_latency: u8,
}

impl Header{
    // Goes through the PCI 128-bits and parses out the data
    fn new(bus: u32, device: u32, function: u32) -> Self{

        let mut raw_device = [0u32; PCI_REGISTER_LEN];
        for (i,register )in raw_device.iter_mut().enumerate(){
            *register = pci_read_32(bus, device, function, i*4);
        }

        unsafe{
            core::ptr::read(raw_device.as_ptr() as *const Self)
        }
    }
}

/// Struct that holds an Pci [`Device`] that we can expose to other modules
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct Device{
    header: Header,
    bus: u32,
    device: u32,
    function: u32,
}

impl Device{
    /// Creates a new Device when scanned in the PCI memory
    fn new(bus: u32, device: u32, function: u32) -> Self {
        Self {
            header: Header::new(bus, device, function),
            bus,
            device,
            function
        }
    }
    /// Returns an array of the BAR addresses to a driver
    pub fn base_mem_addrs(&self) -> [u32; 6]{
        [
            self.header.base_addr_0,
            self.header.base_addr_1,
            self.header.base_addr_2,
            self.header.base_addr_3,
            self.header.base_addr_4,
            self.header.base_addr_5
        ]
    }
    /// Returns the [`device_id`] and [`vendor_id`] for validation that we support this device
    pub fn did_vid(&self) -> (u16, u16){
        (self.header.device_id, self.header.vendor_id)
    }
}
/// Struct that holds an Array of  [`Devices`] that we can expose to other modules
#[derive(Debug)]
pub struct Devices([Option<Device>; 10]);
/// Starts the process of finding the PCI devices and exposing them to the rest of the program
/// We brute force all possible addresses
pub fn init() -> Devices {
    let mut pci_devices: Devices = Devices(Default::default());
    let mut found = 0;
    for bus in 0..PCI_BUS_LEN{
        for device in 0..PCI_DEVICE_LEN{
            for function in 0..PCI_FUNCTION_LEN{
                if pci_read_32(bus, device, function, 0) == !0 { 
                    continue 
                }
                pci_devices.0[found] = Some(Device::new(bus, device, function));
                found += 1;
            }
        }
    }
    pci_devices
}

impl Devices{
    /// Returns the first NIC it finds of type [`Some`] [`Device`] and [`None`] if no PCI NIC is found
    /// 
    pub fn get_nic(&self) -> Option<Device>{
        self.0.iter().find_map(| &device |{
            match device{
                Some(d) => {
                    if (d.header.class_code == PCI_CLASS_CODE_NETWORK) && (d.header.subclass == PCI_SUBCLASS_CODE_ETHERNET){
                        Some(d)
                    }else{
                        None
                    }
                },
                None => None,
            }
        })
    }
}

/// This function reads a dword (u32) from a PCI device address
fn pci_read_32(bus: u32, device: u32, function: u32, offset: usize) -> u32{
    let address: u32 = PCI_ENABLE_BIT | (bus << 16) | (device << 11) | (function << 8) | (offset & 0xFE) as u32;
    cpu::out32(PCI_CONFIG_ADDRESS, address);
    cpu::in32(PCI_CONFIG_DATA)
}