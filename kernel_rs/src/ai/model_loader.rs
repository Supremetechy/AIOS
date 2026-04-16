//! Model loading infrastructure
//! 
//! Handles loading GGUF models from filesystem/initramfs into memory
//! for inference by llama.cpp

use alloc::string::String;
use crate::println;

/// Model metadata
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: usize,
    pub format: ModelFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum ModelFormat {
    GGUF,
    GGML,
    Unknown,
}

/// Detect model format from file extension
pub fn detect_format(path: &str) -> ModelFormat {
    if path.ends_with(".gguf") {
        ModelFormat::GGUF
    } else if path.ends_with(".ggml") || path.ends_with(".bin") {
        ModelFormat::GGML
    } else {
        ModelFormat::Unknown
    }
}

/// Check if model file exists and is accessible
pub fn verify_model(path: &str) -> Result<ModelInfo, &'static str> {
    println!("[MODEL] Verifying model at: {}", path);
    
    // TODO: Use VFS to check file existence and size
    // For now, return mock info
    
    let format = detect_format(path);
    
    match format {
        ModelFormat::GGUF => {
            println!("[MODEL] ✓ Detected GGUF format");
        }
        ModelFormat::GGML => {
            println!("[MODEL] ⚠ GGML format (legacy)");
        }
        ModelFormat::Unknown => {
            return Err("Unknown model format");
        }
    }
    
    Ok(ModelInfo {
        name: String::from("gemma-2b-it-q4_k_m"),
        path: String::from(path),
        size_bytes: 2_500_000_000, // ~2.5GB
        format,
    })
}

/// Load model into memory
/// Returns pointer to model data and size
pub fn load_model_data(path: &str) -> Result<(*const u8, usize), &'static str> {
    println!("[MODEL] Loading model from: {}", path);
    
    // TODO: Implement actual file loading via VFS
    // For now, return null (llama.cpp will load from file path directly)
    
    // This will be implemented when VFS supports file mapping:
    // 1. Open file via VFS
    // 2. Memory-map the file (mmap)
    // 3. Return pointer to mapped region
    // 4. llama.cpp will use this pointer instead of loading from disk
    
    Err("Direct memory loading not yet implemented - using file path")
}

/// Model loading strategy
pub enum LoadStrategy {
    /// Load from file path (llama.cpp handles I/O)
    FilePath,
    
    /// Load into RAM first, then pass to llama.cpp
    PreLoad,
    
    /// Memory-mapped file (most efficient)
    MemoryMapped,
}

/// Get recommended loading strategy based on available memory
pub fn recommend_strategy(model_size: usize, available_ram: usize) -> LoadStrategy {
    if available_ram > model_size * 2 {
        // Plenty of RAM, preload for faster access
        LoadStrategy::PreLoad
    } else if available_ram > model_size {
        // Sufficient RAM, use memory mapping
        LoadStrategy::MemoryMapped
    } else {
        // Limited RAM, let llama.cpp stream from disk
        LoadStrategy::FilePath
    }
}

/// Print model loading diagnostics
pub fn print_diagnostics(info: &ModelInfo) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                     Model Information                          ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  Name: {:52}║", format_centered(&info.name, 52));
    println!("║  Path: {:52}║", format_centered(&info.path, 52));
    println!("║  Size: {:52}║", format_centered(&format_size(info.size_bytes), 52));
    println!("║  Format: {:50}║", format_centered(&format!("{:?}", info.format), 50));
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1_000_000_000 {
        alloc::format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        alloc::format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else {
        alloc::format!("{} bytes", bytes)
    }
}

fn format_centered(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        let padding = width - s.len();
        let left = padding / 2;
        let right = padding - left;
        alloc::format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
    }
}
