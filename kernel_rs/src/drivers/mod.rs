//! Device drivers module

pub mod pci;
pub mod gpu;
pub mod disk;
pub mod network;
pub mod ahci;
pub mod audio;
pub mod camera;
pub mod e1000;

use crate::println;

/// Initialize all device drivers
pub fn init() {
    println!("[DRIVERS] Initializing device drivers...");
    
    // Initialize PCI bus
    pci::init();
    
    // Initialize GPU drivers
    gpu::init();
    
    // Initialize disk drivers (old)
    disk::init();
    
    // Initialize AHCI (SATA) driver
    ahci::init();
    
    // Initialize network drivers
    network::init();
    
    // AI-specific drivers
    println!("[DRIVERS] Initializing AI-specific drivers...");
    
    // Initialize audio (microphone for STT, speaker for TTS)
    audio::init();
    
    // Initialize camera (vision for AI processing)
    camera::init();
    
    println!("[DRIVERS] ✓ All device drivers initialized");
}
