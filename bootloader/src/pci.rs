//! This is where we enumerate all the PCI devices and find the ones we want to use to expose to other
//! parts of the OS
use crate::cpu;

/// PCI Magic numbers
const PCI_ENABLE_BIT: u32 = 1 << 31;
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// This struct holds the data for a header type 0x0 PCI Device
/// 
#[derive(Debug)]
struct PciDevice{
    device_id: u16,
    vendor_id: u16,
    status: u16,
    command: u16,
    class_code: u8,
    subclass: u8,
    prog_if: u8,
    revision_id: u8,
    bist: u8,
    header_type: u8,
    latency_timer: u8,
    cache_line_size: u8,
    base_addr_0: u32,
    base_addr_1: u32,
    base_addr_2: u32,
    base_addr_3: u32,
    base_addr_4: u32,
    base_addr_5: u32,
    cardbus_cis_ptr: u32,
    subsystem_id: u16,
    subsystem_vendor_id: u16,
    expansion_rom_base_addr: u32,
    capabilities_ptr: u8,
    max_latency: u8,
    min_grant: u8,
    interrupt_pin: u8,
    interrupt_line: u8,
}

impl PciDevice{
    // Goes through the PCI 128-bits and parses out the data
    fn new(bus: u32, device: u32, function: u32) -> Self{

        let tmp = pci_read_word(bus, device, function, 0x0);
        let device_id = (tmp >> 16) as u16;
        let vendor_id = tmp as u16;

        let tmp = pci_read_word(bus, device, function, 0x4);
        let status = (tmp >> 16) as u16;
        let command = tmp as u16;

        let tmp = pci_read_word(bus, device, function, 0x8);
        let class_code = (tmp >> 24) as u8;
        let subclass = (tmp >> 16) as u8;
        let prog_if = (tmp >> 8) as u8;
        let revision_id = tmp as u8;

        let tmp = pci_read_word(bus, device, function, 0xC);
        let bist = (tmp >> 24) as u8;
        let header_type = (tmp >> 16) as u8;
        let latency_timer = (tmp >> 8) as u8;
        let cache_line_size = tmp as u8;

        let base_addr_0 = pci_read_word(bus, device, function, 0x10);
        let base_addr_1 = pci_read_word(bus, device, function, 0x14);
        let base_addr_2 = pci_read_word(bus, device, function, 0x18);
        let base_addr_3 = pci_read_word(bus, device, function, 0x1C);
        let base_addr_4 = pci_read_word(bus, device, function, 0x20);
        let base_addr_5 = pci_read_word(bus, device, function, 0x24);
        let cardbus_cis_ptr = pci_read_word(bus, device, function, 0x28);

        let tmp = pci_read_word(bus, device, function, 0x2C);
        let subsystem_id = (tmp >> 16) as u16;
        let subsystem_vendor_id = tmp as u16;

        let expansion_rom_base_addr = pci_read_word(bus, device, function, 0x30);

        let tmp = pci_read_word(bus, device, function, 0x34);
        let capabilities_ptr = tmp as u8;

        let tmp = pci_read_word(bus, device, function, 0x3C);
        let max_latency = (tmp >> 24) as u8;
        let min_grant = (tmp >> 16) as u8;
        let interrupt_pin = (tmp >> 8) as u8;
        let interrupt_line = tmp as u8;

        Self{
            device_id,
            vendor_id,
            status,
            command,
            class_code,
            subclass,
            prog_if,
            revision_id,
            bist,
            header_type,
            latency_timer,
            cache_line_size,
            base_addr_0,
            base_addr_1,
            base_addr_2,
            base_addr_3,
            base_addr_4,
            base_addr_5,
            cardbus_cis_ptr,
            subsystem_id,
            subsystem_vendor_id,
            expansion_rom_base_addr,
            capabilities_ptr,
            max_latency,
            min_grant,
            interrupt_pin,
            interrupt_line,
        }
    }
}

/// Struct that holds an array of [`PciDevice`] that we can expose to other modules
#[derive(Debug)]
pub struct PciDevices([Option<PciDevice>; 10]);

impl PciDevices{
    /// Starts the process of finding the PCI devices and exposing them to the rest of the program
    pub fn init() -> Self {
        let mut pci_devices: PciDevices = PciDevices(Default::default());
        //let mut pci_devices: [Option<PciDevice>; 10] = [None, None, None, None, None, None, None, None, None, None];
        let mut found = 0;
        for bus in 0..256{
            for device in 0..32{
                for function in 0..8{
                    if pci_read_word(bus, device, function, 0) == 0xFFFFFFFF { 
                        continue 
                    }
                    pci_devices.0[found] = Some(PciDevice::new(bus, device, function));
                    found += 1;
                }
            }
        }
        pci_devices
    }
}

/// This function reads a word (u32) from a PCI device address
fn pci_read_word(bus: u32, device: u32, function: u32, offset: u32) -> u32{
    let address: u32 = PCI_ENABLE_BIT | (bus << 16) | (device << 11) | (function << 8) | (offset & 0xFE);
    cpu::out32(PCI_CONFIG_ADDRESS, address);
    cpu::in32(PCI_CONFIG_DATA)
}