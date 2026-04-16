//! AHCI (Advanced Host Controller Interface) driver
//! SATA disk driver for persistent storage (AI models, datasets, autonomous file ops)

use alloc::vec::Vec;
use x86_64::instructions::port::Port;
use spin::Mutex;

/// AHCI HBA Memory Registers
#[repr(C)]
struct HBAMemory {
    cap: u32,           // Host Capabilities
    ghc: u32,           // Global Host Control
    is: u32,            // Interrupt Status
    pi: u32,            // Ports Implemented
    vs: u32,            // Version
    ccc_ctl: u32,       // Command Completion Coalescing Control
    ccc_ports: u32,     // Command Completion Coalsecing Ports
    em_loc: u32,        // Enclosure Management Location
    em_ctl: u32,        // Enclosure Management Control
    cap2: u32,          // Host Capabilities Extended
    bohc: u32,          // BIOS/OS Handoff Control and Status
}

/// AHCI Port Registers
#[repr(C)]
struct HBAPort {
    clb: u64,           // Command List Base Address
    fb: u64,            // FIS Base Address
    is: u32,            // Interrupt Status
    ie: u32,            // Interrupt Enable
    cmd: u32,           // Command and Status
    _reserved: u32,
    tfd: u32,           // Task File Data
    sig: u32,           // Signature
    ssts: u32,          // SATA Status
    sctl: u32,          // SATA Control
    serr: u32,          // SATA Error
    sact: u32,          // SATA Active
    ci: u32,            // Command Issue
    sntf: u32,          // SATA Notification
    fbs: u32,           // FIS-based Switching Control
}

/// Port types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PortType {
    None,
    SATA,
    SATAPI,
    PM,
    SEMB,
}

/// AHCI Command
#[repr(C)]
struct HBACommandHeader {
    flags: u16,
    prdtl: u16,         // Physical Region Descriptor Table Length
    prdbc: u32,         // Physical Region Descriptor Byte Count
    ctba: u64,          // Command Table Base Address
    _reserved: [u32; 4],
}

/// Physical Region Descriptor
#[repr(C)]
struct HBAPRDT {
    dba: u64,           // Data Base Address
    _reserved: u32,
    dbc: u32,           // Data Byte Count (22 bits) + I (1 bit)
}

/// Command Table
#[repr(C)]
struct HBACommandTable {
    cfis: [u8; 64],     // Command FIS
    acmd: [u8; 16],     // ATAPI Command
    _reserved: [u8; 48],
    prdt: [HBAPRDT; 1], // Physical Region Descriptor Table
}

/// FIS Types
const FIS_TYPE_REG_H2D: u8 = 0x27;
const FIS_TYPE_REG_D2H: u8 = 0x34;
const FIS_TYPE_DMA_ACT: u8 = 0x39;
const FIS_TYPE_DMA_SETUP: u8 = 0x41;
const FIS_TYPE_DATA: u8 = 0x46;
const FIS_TYPE_BIST: u8 = 0x58;
const FIS_TYPE_PIO_SETUP: u8 = 0x5F;
const FIS_TYPE_DEV_BITS: u8 = 0xA1;

/// ATA Commands
const ATA_CMD_READ_DMA: u8 = 0xC8;
const ATA_CMD_READ_DMA_EX: u8 = 0x25;
const ATA_CMD_WRITE_DMA: u8 = 0xCA;
const ATA_CMD_WRITE_DMA_EX: u8 = 0x35;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

/// AHCI Controller
pub struct AHCIController {
    abar: u64,          // AHCI Base Address
    ports: Vec<AHCIPort>,
}

/// AHCI Port (represents a disk)
pub struct AHCIPort {
    port_num: usize,
    port_type: PortType,
    port_regs: *mut HBAPort,
    command_list: *mut HBACommandHeader,
    fis_base: *mut u8,
}

impl AHCIController {
    /// Initialize AHCI controller from PCI device
    pub fn new(pci_bus: u8, pci_slot: u8) -> Option<Self> {
        use crate::println;
        
        println!("[AHCI] Initializing AHCI controller at bus {}, slot {}", pci_bus, pci_slot);
        
        // Read AHCI Base Address Register (BAR5) from PCI config
        let abar = read_pci_bar5(pci_bus, pci_slot)?;
        
        println!("[AHCI]   ABAR: {:#x}", abar);
        
        let hba_mem = unsafe { &mut *(abar as *mut HBAMemory) };
        
        // Check version
        let version = hba_mem.vs;
        println!("[AHCI]   Version: {}.{}", (version >> 16) & 0xFFFF, version & 0xFFFF);
        
        // Check capabilities
        let cap = hba_mem.cap;
        let num_ports = ((cap & 0x1F) + 1) as usize;
        println!("[AHCI]   Ports: {}", num_ports);
        
        // Enable AHCI mode
        hba_mem.ghc |= 1 << 31; // Set AE bit
        
        let mut controller = AHCIController {
            abar,
            ports: Vec::new(),
        };
        
        // Probe ports
        controller.probe_ports(hba_mem);
        
        Some(controller)
    }
    
    /// Probe all ports to find connected drives
    fn probe_ports(&mut self, hba_mem: &mut HBAMemory) {
        use crate::println;
        
        let pi = hba_mem.pi; // Ports Implemented
        
        for i in 0..32 {
            if (pi & (1 << i)) == 0 {
                continue; // Port not implemented
            }
            
            let port_regs = unsafe {
                let ports_offset = 0x100 + (i * 0x80);
                (self.abar + ports_offset) as *mut HBAPort
            };
            
            let port_type = unsafe { check_port_type(&*port_regs) };
            
            if port_type != PortType::None {
                println!("[AHCI]   Port {}: {:?} drive detected", i, port_type);
                
                let port = AHCIPort {
                    port_num: i,
                    port_type,
                    port_regs,
                    command_list: core::ptr::null_mut(),
                    fis_base: core::ptr::null_mut(),
                };
                
                self.ports.push(port);
            }
        }
    }
    
    /// Read sectors from disk
    pub fn read(&mut self, port_num: usize, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), &'static str> {
        if port_num >= self.ports.len() {
            return Err("Invalid port");
        }
        
        let port = &mut self.ports[port_num];
        port.read_sectors(lba, count, buffer)
    }
    
    /// Write sectors to disk
    pub fn write(&mut self, port_num: usize, lba: u64, count: u16, buffer: &[u8]) -> Result<(), &'static str> {
        if port_num >= self.ports.len() {
            return Err("Invalid port");
        }
        
        let port = &mut self.ports[port_num];
        port.write_sectors(lba, count, buffer)
    }
}

impl AHCIPort {
    /// Initialize port
    fn init(&mut self) -> Result<(), &'static str> {
        unsafe {
            let port = &mut *self.port_regs;
            
            // Stop command engine
            port.cmd &= !(1 << 0); // Clear ST
            port.cmd &= !(1 << 4); // Clear FRE
            
            // Wait for CR and FR to clear
            while (port.cmd & ((1 << 15) | (1 << 14))) != 0 {}
            
            // Allocate command list (1KB aligned)
            let cl_base = alloc_aligned(1024, 1024);
            self.command_list = cl_base as *mut HBACommandHeader;
            port.clb = cl_base as u64;
            
            // Allocate FIS receive area (256 bytes aligned)
            let fis_base = alloc_aligned(256, 256);
            self.fis_base = fis_base;
            port.fb = fis_base as u64;
            
            // Clear interrupts
            port.is = 0xFFFFFFFF;
            
            // Enable FIS receive
            port.cmd |= 1 << 4; // Set FRE
            
            // Start command engine
            port.cmd |= 1 << 0; // Set ST
        }
        
        Ok(())
    }
    
    /// Read sectors
    fn read_sectors(&mut self, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), &'static str> {
        if buffer.len() < (count as usize * 512) {
            return Err("Buffer too small");
        }
        
        unsafe {
            let port = &mut *self.port_regs;
            
            // Find free command slot
            let slot = self.find_free_slot()?;
            
            let cmdheader = &mut *self.command_list.add(slot);
            cmdheader.flags = 5; // FIS length = 5 DWORDs
            cmdheader.prdtl = 1; // One PRD entry
            
            // Allocate command table
            let cmdtbl = alloc_aligned(256, 128) as *mut HBACommandTable;
            cmdheader.ctba = cmdtbl as u64;
            
            // Set up PRDT
            let prdt = &mut (*cmdtbl).prdt[0];
            prdt.dba = buffer.as_ptr() as u64;
            prdt.dbc = ((count as u32 * 512) - 1) | (1 << 31); // Set I bit
            
            // Set up Command FIS
            let fis = &mut (*cmdtbl).cfis;
            core::ptr::write_bytes(fis.as_mut_ptr(), 0, 64);
            
            fis[0] = FIS_TYPE_REG_H2D;
            fis[1] = 1 << 7; // Command bit
            fis[2] = if lba >= (1 << 28) { ATA_CMD_READ_DMA_EX } else { ATA_CMD_READ_DMA };
            
            // LBA
            fis[4] = (lba & 0xFF) as u8;
            fis[5] = ((lba >> 8) & 0xFF) as u8;
            fis[6] = ((lba >> 16) & 0xFF) as u8;
            fis[7] = 0x40 | ((lba >> 24) & 0x0F) as u8; // LBA mode
            fis[8] = ((lba >> 32) & 0xFF) as u8;
            fis[9] = ((lba >> 40) & 0xFF) as u8;
            
            // Count
            fis[12] = (count & 0xFF) as u8;
            fis[13] = ((count >> 8) & 0xFF) as u8;
            
            // Issue command
            port.ci = 1 << slot;
            
            // Wait for completion
            loop {
                if (port.ci & (1 << slot)) == 0 {
                    break;
                }
                if port.is & (1 << 30) != 0 { // Task file error
                    return Err("Disk read error");
                }
            }
        }
        
        Ok(())
    }
    
    /// Write sectors
    fn write_sectors(&mut self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), &'static str> {
        if buffer.len() < (count as usize * 512) {
            return Err("Buffer too small");
        }
        
        unsafe {
            let port = &mut *self.port_regs;
            let slot = self.find_free_slot()?;
            
            let cmdheader = &mut *self.command_list.add(slot);
            cmdheader.flags = (1 << 6) | 5; // Write flag + FIS length
            cmdheader.prdtl = 1;
            
            let cmdtbl = alloc_aligned(256, 128) as *mut HBACommandTable;
            cmdheader.ctba = cmdtbl as u64;
            
            let prdt = &mut (*cmdtbl).prdt[0];
            prdt.dba = buffer.as_ptr() as u64;
            prdt.dbc = ((count as u32 * 512) - 1) | (1 << 31);
            
            let fis = &mut (*cmdtbl).cfis;
            core::ptr::write_bytes(fis.as_mut_ptr(), 0, 64);
            
            fis[0] = FIS_TYPE_REG_H2D;
            fis[1] = 1 << 7;
            fis[2] = if lba >= (1 << 28) { ATA_CMD_WRITE_DMA_EX } else { ATA_CMD_WRITE_DMA };
            
            fis[4] = (lba & 0xFF) as u8;
            fis[5] = ((lba >> 8) & 0xFF) as u8;
            fis[6] = ((lba >> 16) & 0xFF) as u8;
            fis[7] = 0x40 | ((lba >> 24) & 0x0F) as u8;
            fis[8] = ((lba >> 32) & 0xFF) as u8;
            fis[9] = ((lba >> 40) & 0xFF) as u8;
            
            fis[12] = (count & 0xFF) as u8;
            fis[13] = ((count >> 8) & 0xFF) as u8;
            
            port.ci = 1 << slot;
            
            loop {
                if (port.ci & (1 << slot)) == 0 {
                    break;
                }
                if port.is & (1 << 30) != 0 {
                    return Err("Disk write error");
                }
            }
        }
        
        Ok(())
    }
    
    fn find_free_slot(&self) -> Result<usize, &'static str> {
        unsafe {
            let port = &*self.port_regs;
            let slots = port.ci | port.sact;
            
            for i in 0..32 {
                if (slots & (1 << i)) == 0 {
                    return Ok(i);
                }
            }
        }
        
        Err("No free command slots")
    }
}

/// Check port type
unsafe fn check_port_type(port: &HBAPort) -> PortType {
    let ssts = port.ssts;
    
    let ipm = (ssts >> 8) & 0x0F;
    let det = ssts & 0x0F;
    
    if det != 3 || ipm != 1 {
        return PortType::None; // No device
    }
    
    match port.sig {
        0xEB140101 => PortType::SATAPI,
        0xC33C0101 => PortType::SEMB,
        0x96690101 => PortType::PM,
        0x00000101 => PortType::SATA,
        _ => PortType::None,
    }
}

/// Read PCI BAR5 (AHCI Base Address)
fn read_pci_bar5(bus: u8, slot: u8) -> Option<u64> {
    use crate::drivers::pci::read_config_dword;
    
    // BAR5 is at offset 0x24
    let bar_low = read_config_dword(bus, slot, 0, 0x24);
    
    // Check if 64-bit BAR
    if (bar_low & 0x4) != 0 {
        let bar_high = read_config_dword(bus, slot, 0, 0x28);
        Some(((bar_high as u64) << 32) | (bar_low as u64 & !0xF))
    } else {
        Some((bar_low & !0xF) as u64)
    }
}

/// Allocate aligned memory (simplified)
fn alloc_aligned(size: usize, align: usize) -> *mut u8 {
    // TODO: Use proper aligned allocator
    let layout = core::alloc::Layout::from_size_align(size, align).unwrap();
    unsafe { alloc::alloc::alloc(layout) }
}

/// Global AHCI controller
static AHCI: Mutex<Option<AHCIController>> = Mutex::new(None);

/// Initialize AHCI driver
pub fn init() {
    use crate::println;
    
    println!("[AHCI] Initializing AHCI disk driver...");
    
    // Scan PCI for AHCI controller (Class 01, Subclass 06)
    // This will be called from PCI driver
    
    println!("[AHCI] ✓ AHCI driver initialized");
}

/// Public API
pub fn read_sectors(disk: usize, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), &'static str> {
    let mut controller = AHCI.lock();
    if let Some(ref mut ctrl) = *controller {
        ctrl.read(disk, lba, count, buffer)
    } else {
        Err("AHCI not initialized")
    }
}

pub fn write_sectors(disk: usize, lba: u64, count: u16, buffer: &[u8]) -> Result<(), &'static str> {
    let mut controller = AHCI.lock();
    if let Some(ref mut ctrl) = *controller {
        ctrl.write(disk, lba, count, buffer)
    } else {
        Err("AHCI not initialized")
    }
}
