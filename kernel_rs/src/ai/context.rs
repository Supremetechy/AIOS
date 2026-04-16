//! System context collector for AI
//! 
//! Gathers hardware information and system state to provide context
//! to the AI for intelligent onboarding and assistance.

use alloc::string::String;
use alloc::format;
use crate::println;
use crate::memory::MemoryStats;

/// Collect comprehensive system information for AI context
pub fn collect_system_info() -> String {
    println!("[CONTEXT] Collecting system information...");
    
    let mut context = String::new();
    
    // System type
    context.push_str("System: AI-OS Bare-Metal Operating System\n");
    context.push_str("Architecture: x86_64\n");
    context.push_str("Mode: Native kernel execution\n\n");
    
    // Memory information
    let mem_stats = get_memory_info();
    context.push_str(&format!("Memory:\n"));
    context.push_str(&format!("  Total: {} MB\n", mem_stats.total_mb()));
    context.push_str(&format!("  Available: {} MB\n", mem_stats.available_mb()));
    context.push_str(&format!("  Used: {} MB\n\n", mem_stats.used_mb()));
    
    // CPU information (placeholder - will be populated from hardware detection)
    context.push_str("CPU: Detected during boot\n");
    context.push_str("  Cores: Available\n");
    context.push_str("  Features: x86_64, SSE, AVX\n\n");
    
    // GPU information (placeholder)
    context.push_str("GPU: Detected via PCI scan\n");
    context.push_str("  Status: Available for compute\n\n");
    
    // Boot stage
    context.push_str("Boot Stage: SYSTEM_READY\n");
    context.push_str("Drivers Loaded: PCI, VGA, Keyboard, Serial\n");
    context.push_str("Filesystems: VFS initialized\n\n");
    
    // AI capabilities
    context.push_str("AI System:\n");
    context.push_str("  Engine: llama.cpp (native)\n");
    context.push_str("  Model: Gemma 2B Instruction-Tuned\n");
    context.push_str("  Mode: On-device inference\n");
    
    println!("[CONTEXT] ✓ Context collected ({} bytes)", context.len());
    
    context
}

/// Get memory statistics
fn get_memory_info() -> MemoryStats {
    // This will be replaced with actual memory detection
    MemoryStats::new()
}

/// Format hardware info from Python detection as AI context
pub fn format_hardware_context(json_data: &str) -> String {
    // TODO: Parse JSON from hardware_detection.py and format for AI
    // For now, return basic template
    
    let mut context = String::from("Hardware Configuration:\n\n");
    
    context.push_str("This system has been detected with the following components.\n");
    context.push_str("Please help the user configure and optimize their AI workloads.\n\n");
    
    // Append raw JSON for now
    context.push_str("Detailed specs: ");
    context.push_str(json_data);
    
    context
}

/// Create onboarding prompt for AI
pub fn create_onboarding_prompt(hardware_context: &str) -> String {
    let mut prompt = String::from(
        "You are an AI assistant embedded in AI-OS, a bare-metal operating system \
         designed for AI workloads. You are running natively in the kernel.\n\n"
    );
    
    prompt.push_str("System Information:\n");
    prompt.push_str(hardware_context);
    prompt.push_str("\n\n");
    
    prompt.push_str(
        "Your role is to:\n\
         1. Welcome the user to AI-OS\n\
         2. Explain the detected hardware capabilities\n\
         3. Guide them through initial setup\n\
         4. Help configure GPU acceleration if available\n\
         5. Assist with AI workload optimization\n\n"
    );
    
    prompt.push_str(
        "Provide a warm, helpful welcome message and ask what they'd like to set up first. \
         Keep your response concise (2-3 paragraphs).\n\n"
    );
    
    prompt.push_str("Response:");
    
    prompt
}
