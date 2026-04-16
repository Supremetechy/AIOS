//! Python interpreter embedding for AI-OS
//! Embeds CPython into the kernel to run AI workloads

pub mod ffi;
pub mod bindings;
pub mod runtime;

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

/// Python interpreter state
pub struct PythonInterpreter {
    initialized: bool,
    main_thread_state: usize,
}

impl PythonInterpreter {
    pub const fn new() -> Self {
        PythonInterpreter {
            initialized: false,
            main_thread_state: 0,
        }
    }
    
    /// Initialize Python interpreter
    pub fn initialize(&mut self) -> Result<(), &'static str> {
        println!("[PYTHON] Initializing embedded Python interpreter...");
        
        if self.initialized {
            return Err("Python already initialized");
        }
        
        // Initialize CPython
        unsafe {
            // Set program name
            ffi::py_set_program_name(b"AI-OS\0".as_ptr());
            
            // Initialize Python
            ffi::py_initialize();
            
            if !ffi::py_is_initialized() {
                return Err("Failed to initialize Python");
            }
        }
        
        self.initialized = true;
        
        println!("[PYTHON] ✓ Python {} initialized", self.version());
        
        // Import AI-OS Python modules (best-effort)
        let _ = self.import_aios_modules();
        
        Ok(())
    }
    
    /// Get Python version
    pub fn version(&self) -> String {
        unsafe {
            let ver = ffi::py_get_version();
            // Convert C string to Rust string (best-effort)
            if !ver.is_null() {
                let mut len = 0;
                while *ver.add(len) != 0 { len += 1; }
                let slice = core::slice::from_raw_parts(ver, len);
                if let Ok(s) = core::str::from_utf8(slice) {
                    return s.to_string();
                }
            }
            String::from("unknown")
        }
    }
    
    /// Execute Python code
    pub fn execute(&self, code: &str) -> Result<String, &'static str> {
        if !self.initialized {
            return Err("Python not initialized");
        }
        
        println!("[PYTHON] Executing: {}", code);
        
        unsafe {
            let result = ffi::py_run_simple_string(code.as_ptr());
            
            if result != 0 {
                return Err("Python execution failed");
            }
        }
        
        Ok(String::from("Success"))
    }
    
    /// Execute Python file
    pub fn execute_file(&self, path: &str) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Python not initialized");
        }
        
        println!("[PYTHON] Executing file: {}", path);
        
        // TODO: Read file from VFS and execute
        
        Ok(())
    }
    
    /// Import AI-OS Python modules
    fn import_aios_modules(&self) -> Result<(), &'static str> {
        println!("[PYTHON] Importing AI-OS modules...");
        
        // Import kernel module (provides syscall access)
        self.execute("import aios_kernel")?;
        
        // Import AI modules
        self.execute("import aios_advanced")?;
        
        println!("[PYTHON] ✓ AI-OS modules imported");
        
        Ok(())
    }
    
    /// Call Python function
    pub fn call_function(&self, module: &str, function: &str, args: &[&str]) -> Result<String, &'static str> {
        if !self.initialized {
            return Err("Python not initialized");
        }
        
        // Build Python call string
        let args_str = args.join(", ");
        let code = format!("{}.{}({})", module, function, args_str);
        
        self.execute(&code)
    }
    
    /// Get Python global variable
    pub fn get_global(&self, name: &str) -> Result<String, &'static str> {
        if !self.initialized {
            return Err("Python not initialized");
        }
        
        unsafe {
            let main_module = ffi::py_import_add_module(b"__main__\0".as_ptr());
            if main_module.is_null() {
                return Err("Failed to get main module");
            }
            
            let globals = ffi::py_module_get_dict(main_module);
            let key = ffi::py_unicode_from_string(name.as_ptr());
            let value = ffi::py_dict_get_item(globals, key);
            
            if value.is_null() {
                return Err("Variable not found");
            }
            
            // Convert to string
            let str_obj = ffi::py_object_str(value);
            let c_str = ffi::py_unicode_as_utf8(str_obj);
            
            // TODO: Convert C string to Rust String
            
            Ok(String::from("value"))
        }
    }
    
    /// Finalize Python interpreter
    pub fn finalize(&mut self) {
        if !self.initialized {
            return;
        }
        
        println!("[PYTHON] Finalizing Python interpreter...");
        
        unsafe {
            ffi::py_finalize();
        }
        
        self.initialized = false;
    }
}

/// Global Python interpreter instance
static mut PYTHON: PythonInterpreter = PythonInterpreter::new();

/// Initialize Python subsystem
pub fn init() -> Result<(), &'static str> {
    println!("[PYTHON] Initializing Python subsystem...");

    // Prepare runtime (extract minimal stdlib, set paths)
    super::runtime::init().map_err(|_| "runtime init failed")?;
    
    unsafe {
        PYTHON.initialize()?;
    }

    // Register kernel bindings/module
    super::bindings::init();
    
    println!("[PYTHON] ✓ Python subsystem ready");
    println!("[PYTHON]   Kernel APIs exposed to Python");
    println!("[PYTHON]   AI-OS Python layer integrated");
    
    Ok(())
}

/// Execute Python code
pub fn execute(code: &str) -> Result<String, &'static str> {
    unsafe {
        PYTHON.execute(code)
    }
}

/// Execute Python file
pub fn execute_file(path: &str) -> Result<(), &'static str> {
    unsafe {
        PYTHON.execute_file(path)
    }
}

/// Call Python function
pub fn call_function(module: &str, function: &str, args: &[&str]) -> Result<String, &'static str> {
    unsafe {
        PYTHON.call_function(module, function, args)
    }
}

/// Start Python AI shell (interactive)
pub fn start_ai_shell() -> ! {
    println!("[PYTHON] Starting AI-OS Python shell...");
    
    loop {
        // Read input (from keyboard)
        // Execute Python code
        // Display output
        
        // Simplified for now
        unsafe {
            let _ = PYTHON.execute("print('AI-OS Python Shell Ready')");
        }
        
        // TODO: Implement REPL
        x86_64::instructions::hlt();
    }
}
