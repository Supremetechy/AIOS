//! Init process (PID 1) for AI-OS
//! First user-space process that launches the Python AI shell

use crate::println;
use crate::python;
use crate::process;

/// Init process entry point
pub fn init_process_main() -> ! {
    println!("[INIT] Starting init process (PID 1)...");
    
    // Initialize Python interpreter
    if let Err(e) = python::init() {
        println!("[INIT] Failed to initialize Python: {}", e);
        // Continue in minimal mode without Python
        // In a production system, we might reboot or drop to rescue shell
    }
    
    // Print welcome banner
    print_welcome();
    
    // Start Python AI shell
    println!("[INIT] Launching AI-OS Python shell...");
    
    // Execute startup script
    if let Err(e) = python::execute_file("/aios/scripts/startup.py") {
        println!("[INIT] No startup script found, entering interactive mode");
    }
    
    // Start interactive Python REPL
    python_repl();
}

fn print_welcome() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║                    AI-OS Python Shell                          ║");
    println!("║         Voice-Interactive AI Operating System                  ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    println!("Python {} on AI-OS", "3.12.0");
    println!("Type 'help' for commands, 'exit()' to shutdown\n");
}

fn python_repl() -> ! {
    println!(">>> ");
    
    // Check if AI is available and start AI conversation mode
    println!("[INIT] Starting AI conversation mode...");
    
    // Launch AI conversation loop
    crate::ai::start_conversation();
    
    // Fallback: Traditional Python REPL
    // (This code is unreachable but kept for reference)
    loop {
        let _ = python::execute("print('AI-OS Ready!')");
        let _ = python::execute("import aios_kernel");
        let _ = python::execute("print('Kernel module loaded')");
        
        x86_64::instructions::hlt();
    }
}

/// Launch init process
pub fn launch() {
    println!("[KERNEL] Launching init process...");
    
    // Create init process
    let init_entry = init_process_main as *const () as u64;
    let pid = process::spawn(
        alloc::string::String::from("init"),
        init_entry,
        process::Priority::High
    );
    
    println!("[KERNEL] Init process created (PID {})", pid);
}
