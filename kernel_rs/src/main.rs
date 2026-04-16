//! AI-OS Kernel - Main Entry Point
//! Bare-metal Rust kernel for AI workloads

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

mod vga_buffer;
mod serial;
mod interrupts;
mod gdt;
mod memory;
mod allocator;
mod drivers;
mod task;
mod process;
mod syscall;
mod fs;
mod python;
mod init;
mod security;
mod ai;
mod net;

/// Kernel entry point called from assembly bootloader
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Initialize kernel subsystems
    init();
    
    // Print boot banner
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                  AI-OS Bare-Metal Kernel                      ║");
    println!("║                    Version 1.0.0-alpha                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    println!("[KERNEL] Initializing AI-OS...");
    
    // Initialize memory management
    println!("[KERNEL] Setting up memory management...");
    memory::init();
    
    // Initialize interrupt handling
    println!("[KERNEL] Setting up interrupt handlers...");
    interrupts::init();
    
    // Initialize system calls
    println!("[KERNEL] Setting up system call interface...");
    syscall::init();

    // Initialize security subsystem
    println!("[KERNEL] Setting up security subsystem...");
    security::init();
    
    // Initialize process scheduler
    println!("[KERNEL] Setting up process scheduler...");
    process::init();
    
    // Initialize filesystem
    println!("[KERNEL] Setting up filesystem...");
    fs::init();
    
    // Initialize hardware drivers
    println!("[KERNEL] Loading device drivers...");
    drivers::init();
    
    // Initialize task scheduler (legacy)
    println!("[KERNEL] Starting task scheduler...");
    task::init();
    
    // Initialize network subsystem (optional)
    println!("[KERNEL] Initializing network subsystem...");
    match net::init(net::NetworkConfig::default()) {
        Ok(_) => {
            println!("[KERNEL] ✓ Network subsystem ready");
            
            // Initialize TLS optimizations
            net::tls_optimized::init_session_cache();
            net::tls_optimized::init_connection_pool();
            println!("[KERNEL] ✓ TLS optimizations enabled");
        }
        Err(e) => println!("[KERNEL] ⚠ Network init failed: {} (offline mode)", e),
    }
    
    // Initialize AI subsystem
    println!("[KERNEL] Initializing AI subsystem...");
    match ai::init(ai::AIConfig::default()) {
        Ok(_) => println!("[KERNEL] ✓ AI subsystem ready"),
        Err(e) => println!("[KERNEL] ⚠ AI init failed: {} (continuing without AI)", e),
    }
    
    println!("[KERNEL] ✓ Boot complete!");
    println!();
    
    // Launch init process (PID 1) with Python shell
    init::launch();
    
    println!("[KERNEL] Init process launched");
    println!();
    
    // Kernel main loop
    loop {
        x86_64::instructions::hlt();
    }
}

/// Initialize kernel subsystems
fn init() {
    // Initialize GDT
    gdt::init();
    
    // Initialize serial port for debugging
    serial::init();
    
    // Initialize VGA text mode
    vga_buffer::init();
}

/// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n[KERNEL PANIC] {}", info);
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

mod shell {
    use super::println;
    
    pub fn run() -> ! {
        println!("aios> ");
        
        loop {
            // Simple shell loop
            // In a real implementation, this would handle keyboard input
            x86_64::instructions::hlt();
        }
    }
}
