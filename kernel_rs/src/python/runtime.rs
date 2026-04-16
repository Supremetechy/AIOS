//! Python runtime environment for embedded kernel
//! Provides Python standard library and AI-OS integration

use crate::println;
use alloc::string::String;

/// Embedded Python standard library (minimal)
pub const PYTHON_STDLIB: &[(&str, &str)] = &[
    // Core modules
    ("sys.py", include_str!("../../../python_embed/sys.py")),
    ("os.py", include_str!("../../../python_embed/os.py")),
    ("io.py", include_str!("../../../python_embed/io.py")),
    
    // AI-OS kernel module
    ("aios_kernel.py", include_str!("../../../python_embed/aios_kernel.py")),
];

/// Python runtime configuration
pub struct PythonRuntime {
    python_home: String,
    module_search_paths: [&'static str; 5],
}

impl PythonRuntime {
    pub fn new() -> Self {
        PythonRuntime {
            python_home: String::from("/aios/lib/python3.12"),
            module_search_paths: [
                "/aios/lib/python3.12",
                "/aios/lib/python3.12/site-packages",
                "/aios/models",  // AI model scripts
                "/aios/scripts", // User scripts
                ".",             // Current directory
            ],
        }
    }
    
    /// Setup Python environment
    pub fn setup(&self) -> Result<(), &'static str> {
        println!("[PYTHON] Setting up runtime environment...");
        
        // Create Python directories
        self.create_directories()?;
        
        // Extract embedded stdlib
        self.extract_stdlib()?;
        
        // Set environment variables
        self.set_environment()?;
        
        println!("[PYTHON] ✓ Runtime environment ready");
        Ok(())
    }
    
    fn create_directories(&self) -> Result<(), &'static str> {
        use crate::fs::vfs;
        use crate::fs::FileType;
        
        // Create Python home directory
        vfs::create(&self.python_home, FileType::Directory).ok();
        
        // Create module paths
        for path in &self.module_search_paths {
            vfs::create(path, FileType::Directory).ok();
        }
        
        Ok(())
    }
    
    fn extract_stdlib(&self) -> Result<(), &'static str> {
        use crate::fs::vfs;
        use crate::fs::FileType;
        
        println!("[PYTHON] Extracting embedded standard library...");
        
        for (filename, content) in PYTHON_STDLIB {
            let path = format!("{}/{}", self.python_home, filename);
            
            // Create file
            vfs::create(&path, FileType::Regular).ok();
            
            // Open and write
            if let Ok(fd) = vfs::open(&path, 0x02) { // O_RDWR
                vfs::write(fd, content.as_bytes()).ok();
                vfs::close(fd).ok();
            }
            
            println!("[PYTHON]   Extracted: {}", filename);
        }
        
        Ok(())
    }
    
    fn set_environment(&self) -> Result<(), &'static str> {
        unsafe {
            // Set program name (best-effort)
            super::ffi::py_set_program_name(b"AI-OS\0".as_ptr());
            // Additional sys.path setup can be done from startup.py
        }
        
        Ok(())
    }
    
    /// Get module search paths for Python
    pub fn get_search_paths(&self) -> String {
        self.module_search_paths.join(":")
    }
}

/// Initialize Python runtime
pub fn init() -> Result<(), &'static str> {
    let runtime = PythonRuntime::new();
    runtime.setup()
}
