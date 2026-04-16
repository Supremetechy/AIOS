//! AI subsystem for AI-OS
//! Provides on-device language model inference using llama.cpp
//!
//! This module integrates llama.cpp for native AI capabilities without
//! requiring network connectivity or cloud services.

pub mod llama;
pub mod context;
pub mod conversation;
pub mod model_loader;
pub mod gpu_accel;
pub mod cloud;
pub mod benchmark;
pub mod streaming;

use crate::println;

/// AI subsystem configuration
pub struct AIConfig {
    pub model_path: &'static str,
    pub context_size: usize,
    pub threads: usize,
    pub gpu_layers: usize,
}

impl Default for AIConfig {
    fn default() -> Self {
        AIConfig {
            model_path: "/models/gemma-2b-it-q4_k_m.gguf",
            context_size: 2048,
            threads: 4,
            gpu_layers: 32, // Offload layers to GPU if available
        }
    }
}

/// Initialize AI subsystem
pub fn init(config: AIConfig) -> Result<(), &'static str> {
    println!("[AI] Initializing AI subsystem...");
    println!("[AI]   Model: {}", config.model_path);
    println!("[AI]   Context size: {} tokens", config.context_size);
    println!("[AI]   Threads: {}", config.threads);
    
    // Detect and initialize GPU acceleration
    let mut gpu = gpu_accel::GPUAccelerator::new();
    gpu.detect_devices()?;
    gpu.initialize()?;
    gpu.print_diagnostics();
    
    // Determine GPU layers based on hardware
    let gpu_layers = if config.gpu_layers > 0 {
        config.gpu_layers
    } else {
        gpu.recommended_layers(32) // Gemma 2B has 32 layers
    };
    
    println!("[AI]   GPU layers: {} (auto-detected)", gpu_layers);
    
    // Verify model exists
    let model_info = model_loader::verify_model(config.model_path)?;
    model_loader::print_diagnostics(&model_info);
    
    // Initialize llama.cpp backend
    llama::init()?;
    
    // Load model with GPU acceleration
    llama::load_model(config.model_path, config.context_size, gpu_layers)?;
    
    println!("[AI] ✓ AI subsystem initialized");
    Ok(())
}

/// Query the AI with a prompt
pub fn query(prompt: &str, max_tokens: usize) -> Result<alloc::string::String, &'static str> {
    llama::generate(prompt, max_tokens)
}

/// Get system information for AI context
pub fn get_system_context() -> alloc::string::String {
    context::collect_system_info()
}

/// Start interactive conversation mode
pub fn start_conversation() -> ! {
    conversation::run_loop()
}
