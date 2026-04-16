//! PCI bus driver for device enumeration

use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy)]
pub struct PCIDevice {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
}

impl PCIDevice {
    pub fn is_valid(&self) -> bool {
        self.vendor_id != 0xFFFF
    }

    pub fn is_gpu(&self) -> bool {
        // VGA compatible controller or Display controller
        self.class_code == 0x03
    }

    pub fn is_network(&self) -> bool {
        // Network controller
        self.class_code == 0x02
    }

    pub fn is_storage(&self) -> bool {
        // Mass storage controller
        self.class_code == 0x01
    }
}

pub fn init() {
    use crate::println;
    
    println!("[PCI] Scanning PCI bus...");
    
    let devices = scan_pci_bus();
    
    println!("[PCI] Found {} device(s)", devices.len());
    
    for device in devices {
        println!(
            "[PCI]   Bus {:02x}, Slot {:02x}: Vendor {:04x}, Device {:04x}, Class {:02x}:{:02x}",
            device.bus,
            device.slot,
            device.vendor_id,
            device.device_id,
            device.class_code,
            device.subclass
        );
        
        if device.is_gpu() {
            println!("[PCI]     -> GPU detected");
        } else if device.is_network() {
            println!("[PCI]     -> Network controller");
        } else if device.is_storage() {
            println!("[PCI]     -> Storage controller");
        }
    }
}

fn scan_pci_bus() -> Vec<PCIDevice> {
    let mut devices = Vec::new();

    for bus in 0..256 {
        for slot in 0..32 {
            let device = probe_device(bus as u8, slot as u8, 0);
            if device.is_valid() {
                devices.push(device);
            }
        }
    }

    devices
}

fn probe_device(bus: u8, slot: u8, function: u8) -> PCIDevice {
    let vendor_device = read_config_dword(bus, slot, function, 0x00);
    let class_info = read_config_dword(bus, slot, function, 0x08);

    PCIDevice {
        bus,
        slot,
        function,
        vendor_id: (vendor_device & 0xFFFF) as u16,
        device_id: ((vendor_device >> 16) & 0xFFFF) as u16,
        class_code: ((class_info >> 24) & 0xFF) as u8,
        subclass: ((class_info >> 16) & 0xFF) as u8,
    }
}

fn read_config_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = 0x80000000u32
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        let mut port_addr = Port::new(CONFIG_ADDRESS);
        let mut port_data = Port::new(CONFIG_DATA);

        port_addr.write(address);
        port_data.read()
    }
}
