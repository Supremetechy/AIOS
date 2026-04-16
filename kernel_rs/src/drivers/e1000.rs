//! Intel e1000 (82540EM) Network Driver
//! 
//! Complete implementation of Intel e1000 Gigabit Ethernet controller
//! with DMA, interrupt handling, and packet transmission/reception.

use core::ptr::{read_volatile, write_volatile};
use alloc::vec::Vec;
use spin::Mutex;
use crate::println;

/// e1000 MMIO register offsets
#[repr(u32)]
#[allow(dead_code)]
enum E1000Reg {
    CTRL = 0x0000,      // Device Control
    STATUS = 0x0008,    // Device Status
    EEPROM = 0x0014,    // EEPROM Read
    CTRL_EXT = 0x0018,  // Extended Device Control
    IMASK = 0x00D0,     // Interrupt Mask
    RCTL = 0x0100,      // Receive Control
    TCTL = 0x0400,      // Transmit Control
    TIPG = 0x0410,      // Transmit IPG
    RDBAL = 0x2800,     // RX Descriptor Base Low
    RDBAH = 0x2804,     // RX Descriptor Base High
    RDLEN = 0x2808,     // RX Descriptor Length
    RDH = 0x2810,       // RX Descriptor Head
    RDT = 0x2818,       // RX Descriptor Tail
    TDBAL = 0x3800,     // TX Descriptor Base Low
    TDBAH = 0x3804,     // TX Descriptor Base High
    TDLEN = 0x3808,     // TX Descriptor Length
    TDH = 0x3810,       // TX Descriptor Head
    TDT = 0x3818,       // TX Descriptor Tail
    MTA = 0x5200,       // Multicast Table Array
    RAL = 0x5400,       // Receive Address Low
    RAH = 0x5404,       // Receive Address High
}

/// Control Register bits
const CTRL_RST: u32 = 1 << 26;    // Software Reset
const CTRL_ASDE: u32 = 1 << 5;    // Auto-Speed Detection Enable
const CTRL_SLU: u32 = 1 << 6;     // Set Link Up

/// Receive Control bits
const RCTL_EN: u32 = 1 << 1;      // Receiver Enable
const RCTL_SBP: u32 = 1 << 2;     // Store Bad Packets
const RCTL_UPE: u32 = 1 << 3;     // Unicast Promiscuous Enable
const RCTL_MPE: u32 = 1 << 4;     // Multicast Promiscuous Enable
const RCTL_BAM: u32 = 1 << 15;    // Broadcast Accept Mode
const RCTL_BSIZE_2048: u32 = 0 << 16; // Buffer Size 2048
const RCTL_SECRC: u32 = 1 << 26;  // Strip Ethernet CRC

/// Transmit Control bits
const TCTL_EN: u32 = 1 << 1;      // Transmit Enable
const TCTL_PSP: u32 = 1 << 3;     // Pad Short Packets
const TCTL_CT_SHIFT: u32 = 4;     // Collision Threshold
const TCTL_COLD_SHIFT: u32 = 12;  // Collision Distance

/// Descriptor constants
const NUM_RX_DESC: usize = 256;
const NUM_TX_DESC: usize = 256;
const PACKET_SIZE: usize = 2048;

/// Receive Descriptor
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct RxDesc {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

impl RxDesc {
    fn new() -> Self {
        RxDesc {
            addr: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            special: 0,
        }
    }
}

/// Transmit Descriptor
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TxDesc {
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

impl TxDesc {
    fn new() -> Self {
        TxDesc {
            addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: 0,
            css: 0,
            special: 0,
        }
    }
}

/// TX Command bits
const CMD_EOP: u8 = 1 << 0;   // End of Packet
const CMD_RS: u8 = 1 << 3;    // Report Status

/// Status bits
const STATUS_DD: u8 = 1 << 0; // Descriptor Done

/// e1000 device structure
pub struct E1000Device {
    mmio_base: usize,
    mac_address: [u8; 6],
    rx_descs: Vec<RxDesc>,
    tx_descs: Vec<TxDesc>,
    rx_buffers: Vec<Vec<u8>>,
    tx_buffers: Vec<Vec<u8>>,
    rx_current: usize,
    tx_current: usize,
}

static E1000: Mutex<Option<E1000Device>> = Mutex::new(None);

impl E1000Device {
    /// Create new e1000 device
    pub fn new(mmio_base: usize) -> Self {
        E1000Device {
            mmio_base,
            mac_address: [0; 6],
            rx_descs: vec![RxDesc::new(); NUM_RX_DESC],
            tx_descs: vec![TxDesc::new(); NUM_TX_DESC],
            rx_buffers: Vec::new(),
            tx_buffers: Vec::new(),
            rx_current: 0,
            tx_current: 0,
        }
    }
    
    /// Read from MMIO register
    fn read_reg(&self, reg: E1000Reg) -> u32 {
        unsafe {
            read_volatile((self.mmio_base + reg as usize) as *const u32)
        }
    }
    
    /// Write to MMIO register
    fn write_reg(&self, reg: E1000Reg, value: u32) {
        unsafe {
            write_volatile((self.mmio_base + reg as usize) as *mut u32, value);
        }
    }
    
    /// Initialize the device
    pub fn init(&mut self) -> Result<(), &'static str> {
        println!("[E1000] Initializing Intel e1000 controller...");
        
        // 1. Reset the device
        self.reset()?;
        
        // 2. Read MAC address from EEPROM
        self.read_mac_address()?;
        
        println!("[E1000]   MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                 self.mac_address[0], self.mac_address[1], self.mac_address[2],
                 self.mac_address[3], self.mac_address[4], self.mac_address[5]);
        
        // 3. Setup link
        self.setup_link();
        
        // 4. Initialize RX
        self.init_rx()?;
        
        // 5. Initialize TX
        self.init_tx()?;
        
        // 6. Enable interrupts
        self.enable_interrupts();
        
        println!("[E1000] ✓ Initialization complete");
        
        Ok(())
    }
    
    /// Reset the device
    fn reset(&mut self) -> Result<(), &'static str> {
        println!("[E1000]   Resetting device...");
        
        // Disable interrupts
        self.write_reg(E1000Reg::IMASK, 0);
        
        // Issue software reset
        let ctrl = self.read_reg(E1000Reg::CTRL);
        self.write_reg(E1000Reg::CTRL, ctrl | CTRL_RST);
        
        // Wait for reset to complete (10ms)
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
        
        // Disable interrupts again after reset
        self.write_reg(E1000Reg::IMASK, 0);
        
        Ok(())
    }
    
    /// Read MAC address from EEPROM
    fn read_mac_address(&mut self) -> Result<(), &'static str> {
        // Read from Receive Address registers (set by EEPROM)
        let low = self.read_reg(E1000Reg::RAL);
        let high = self.read_reg(E1000Reg::RAH);
        
        self.mac_address[0] = (low & 0xFF) as u8;
        self.mac_address[1] = ((low >> 8) & 0xFF) as u8;
        self.mac_address[2] = ((low >> 16) & 0xFF) as u8;
        self.mac_address[3] = ((low >> 24) & 0xFF) as u8;
        self.mac_address[4] = (high & 0xFF) as u8;
        self.mac_address[5] = ((high >> 8) & 0xFF) as u8;
        
        Ok(())
    }
    
    /// Setup link
    fn setup_link(&self) {
        println!("[E1000]   Setting up link...");
        
        let mut ctrl = self.read_reg(E1000Reg::CTRL);
        ctrl |= CTRL_SLU | CTRL_ASDE;
        self.write_reg(E1000Reg::CTRL, ctrl);
        
        // Wait for link up
        for _ in 0..1000 {
            let status = self.read_reg(E1000Reg::STATUS);
            if (status & 0x02) != 0 {  // Link Up bit
                println!("[E1000]   ✓ Link is up");
                return;
            }
            // Small delay
            for _ in 0..10000 {
                core::hint::spin_loop();
            }
        }
        
        println!("[E1000]   ⚠ Link status unknown");
    }
    
    /// Initialize receive ring
    fn init_rx(&mut self) -> Result<(), &'static str> {
        println!("[E1000]   Initializing RX ring...");
        
        // Allocate RX buffers
        for i in 0..NUM_RX_DESC {
            let buffer = vec![0u8; PACKET_SIZE];
            let addr = buffer.as_ptr() as u64;
            self.rx_descs[i].addr = addr;
            self.rx_buffers.push(buffer);
        }
        
        // Set descriptor base and length
        let desc_addr = self.rx_descs.as_ptr() as u64;
        self.write_reg(E1000Reg::RDBAL, (desc_addr & 0xFFFFFFFF) as u32);
        self.write_reg(E1000Reg::RDBAH, (desc_addr >> 32) as u32);
        self.write_reg(E1000Reg::RDLEN, (NUM_RX_DESC * 16) as u32);
        
        // Set head and tail
        self.write_reg(E1000Reg::RDH, 0);
        self.write_reg(E1000Reg::RDT, (NUM_RX_DESC - 1) as u32);
        
        // Enable receiver
        let rctl = RCTL_EN | RCTL_BAM | RCTL_BSIZE_2048 | RCTL_SECRC;
        self.write_reg(E1000Reg::RCTL, rctl);
        
        Ok(())
    }
    
    /// Initialize transmit ring
    fn init_tx(&mut self) -> Result<(), &'static str> {
        println!("[E1000]   Initializing TX ring...");
        
        // Allocate TX buffers
        for _ in 0..NUM_TX_DESC {
            self.tx_buffers.push(vec![0u8; PACKET_SIZE]);
        }
        
        // Set descriptor base and length
        let desc_addr = self.tx_descs.as_ptr() as u64;
        self.write_reg(E1000Reg::TDBAL, (desc_addr & 0xFFFFFFFF) as u32);
        self.write_reg(E1000Reg::TDBAH, (desc_addr >> 32) as u32);
        self.write_reg(E1000Reg::TDLEN, (NUM_TX_DESC * 16) as u32);
        
        // Set head and tail
        self.write_reg(E1000Reg::TDH, 0);
        self.write_reg(E1000Reg::TDT, 0);
        
        // Set IPG (Inter-Packet Gap)
        self.write_reg(E1000Reg::TIPG, 0x00602006);
        
        // Enable transmitter
        let tctl = TCTL_EN | TCTL_PSP | 
                   (0x10 << TCTL_CT_SHIFT) | 
                   (0x40 << TCTL_COLD_SHIFT);
        self.write_reg(E1000Reg::TCTL, tctl);
        
        Ok(())
    }
    
    /// Enable interrupts
    fn enable_interrupts(&self) {
        // Enable link status change and receive interrupts
        self.write_reg(E1000Reg::IMASK, 0x1F6DC);
    }
    
    /// Transmit a packet
    pub fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > PACKET_SIZE {
            return Err("Packet too large");
        }
        
        let tail = self.read_reg(E1000Reg::TDT) as usize;
        let next = (tail + 1) % NUM_TX_DESC;
        
        // Check if descriptor is available
        if self.tx_descs[tail].status & STATUS_DD == 0 && tail != 0 {
            return Err("TX queue full");
        }
        
        // Copy data to buffer
        self.tx_buffers[tail][..data.len()].copy_from_slice(data);
        
        // Setup descriptor
        self.tx_descs[tail].addr = self.tx_buffers[tail].as_ptr() as u64;
        self.tx_descs[tail].length = data.len() as u16;
        self.tx_descs[tail].cmd = CMD_EOP | CMD_RS;
        self.tx_descs[tail].status = 0;
        
        // Update tail
        self.write_reg(E1000Reg::TDT, next as u32);
        
        Ok(())
    }
    
    /// Receive a packet
    pub fn recv(&mut self) -> Option<Vec<u8>> {
        let current = self.rx_current;
        
        // Check if descriptor has data
        if self.rx_descs[current].status & STATUS_DD == 0 {
            return None;
        }
        
        let length = self.rx_descs[current].length as usize;
        let mut packet = vec![0u8; length];
        packet.copy_from_slice(&self.rx_buffers[current][..length]);
        
        // Reset descriptor
        self.rx_descs[current].status = 0;
        
        // Update tail
        self.write_reg(E1000Reg::RDT, current as u32);
        
        // Move to next descriptor
        self.rx_current = (current + 1) % NUM_RX_DESC;
        
        Some(packet)
    }
    
    /// Get MAC address
    pub fn mac(&self) -> [u8; 6] {
        self.mac_address
    }
}

/// Initialize e1000 device
pub fn init(mmio_base: usize) -> Result<(), &'static str> {
    let mut device = E1000Device::new(mmio_base);
    device.init()?;
    
    let mut e1000 = E1000.lock();
    *e1000 = Some(device);
    
    Ok(())
}

/// Send packet
pub fn send_packet(data: &[u8]) -> Result<(), &'static str> {
    let mut e1000 = E1000.lock();
    if let Some(device) = e1000.as_mut() {
        device.send(data)
    } else {
        Err("e1000 not initialized")
    }
}

/// Receive packet
pub fn recv_packet() -> Option<Vec<u8>> {
    let mut e1000 = E1000.lock();
    if let Some(device) = e1000.as_mut() {
        device.recv()
    } else {
        None
    }
}

/// Get MAC address
pub fn get_mac() -> Option<[u8; 6]> {
    let e1000 = E1000.lock();
    e1000.as_ref().map(|d| d.mac())
}
