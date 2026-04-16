//! Network device abstraction
//! 
//! Provides interface to physical network devices (e1000, virtio-net, etc.)

use crate::println;

/// Network device type
#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    Intel8254x,  // e1000
    VirtioNet,   // QEMU/KVM
    Rtl8139,     // Realtek
    Unknown,
}

/// Network device
pub struct NetworkDevice {
    device_type: DeviceType,
    mac_address: [u8; 6],
    mtu: usize,
}

static mut DEVICE: Option<NetworkDevice> = None;

/// Initialize network device
pub fn init() -> Result<(), &'static str> {
    println!("[NET] Detecting network device...");
    
    // Scan PCI for network controllers (class 0x02)
    let device_type = detect_device()?;
    
    println!("[NET] Found: {:?}", device_type);
    
    // Initialize device
    let mac = match device_type {
        DeviceType::Intel8254x => init_e1000()?,
        DeviceType::VirtioNet => init_virtio()?,
        DeviceType::Rtl8139 => init_rtl8139()?,
        DeviceType::Unknown => return Err("No supported network device found"),
    };
    
    println!("[NET] MAC address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
             mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    
    unsafe {
        DEVICE = Some(NetworkDevice {
            device_type,
            mac_address: mac,
            mtu: 1500,
        });
    }
    
    println!("[NET] ✓ Device initialized");
    Ok(())
}

/// Detect network device via PCI
fn detect_device() -> Result<DeviceType, &'static str> {
    // TODO: Use actual PCI scanning from crate::drivers::pci
    // For now, assume e1000 (most common in QEMU)
    
    println!("[NET] Scanning PCI bus for network controllers...");
    
    // Check for Intel e1000 (vendor 0x8086, device 0x100E)
    if pci_device_exists(0x8086, 0x100E) {
        return Ok(DeviceType::Intel8254x);
    }
    
    // Check for VirtIO (vendor 0x1AF4, device 0x1000)
    if pci_device_exists(0x1AF4, 0x1000) {
        return Ok(DeviceType::VirtioNet);
    }
    
    // Check for Realtek (vendor 0x10EC, device 0x8139)
    if pci_device_exists(0x10EC, 0x8139) {
        return Ok(DeviceType::Rtl8139);
    }
    
    Ok(DeviceType::Unknown)
}

/// Check if PCI device exists
fn pci_device_exists(_vendor: u16, _device: u16) -> bool {
    // TODO: Integrate with crate::drivers::pci
    // For now, return true for e1000 (QEMU default)
    true // Assume e1000 for testing
}

/// Initialize Intel e1000
fn init_e1000() -> Result<[u8; 6], &'static str> {
    println!("[NET] Initializing Intel e1000...");
    
    // Get e1000 MMIO base from PCI (BAR0)
    let mmio_base = get_e1000_mmio_base()?;
    
    // Initialize e1000 driver
    crate::drivers::e1000::init(mmio_base)?;
    
    // Get MAC address
    let mac = crate::drivers::e1000::get_mac()
        .ok_or("Failed to get MAC address")?;
    
    Ok(mac)
}

/// Initialize VirtIO network
fn init_virtio() -> Result<[u8; 6], &'static str> {
    println!("[NET] Initializing VirtIO network...");
    
    // TODO: VirtIO initialization
    
    Ok([0x52, 0x54, 0x00, 0x12, 0x34, 0x57])
}

/// Initialize Realtek RTL8139
fn init_rtl8139() -> Result<[u8; 6], &'static str> {
    println!("[NET] Initializing Realtek RTL8139...");
    
    // TODO: RTL8139 initialization
    
    Ok([0x52, 0x54, 0x00, 0x12, 0x34, 0x58])
}

/// Send packet
pub fn send_packet(data: &[u8]) -> Result<(), &'static str> {
    crate::drivers::e1000::send_packet(data)
}

/// Receive packet
pub fn recv_packet() -> Option<alloc::vec::Vec<u8>> {
    crate::drivers::e1000::recv_packet()
}

/// Get e1000 MMIO base address from PCI
fn get_e1000_mmio_base() -> Result<usize, &'static str> {
    // TODO: Scan PCI for e1000 device and read BAR0
    // For QEMU default: 0xFEBC0000
    Ok(0xFEBC0000)
}

/// Get MAC address
pub fn get_mac_address() -> Option<[u8; 6]> {
    unsafe {
        DEVICE.as_ref().map(|d| d.mac_address)
    }
}
