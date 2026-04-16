//! Native Python integration (compiled Python, no interpreter)
//! Calls Python code that has been compiled to C/machine code

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

// External functions from compiled Python code
extern "C" {
    // Main entry point from compiled Python
    fn run_compiled_python();
    
    // Individual compiled Python functions
    fn python_native_main() -> i32;
    fn python_voice_interaction();
    fn python_vision_processing();
    fn python_autonomous_operations();
}

// Kernel functions exported to compiled Python
#[no_mangle]
pub extern "C" fn kernel_print(text: *const u8) {
    unsafe {
        if text.is_null() {
            return;
        }
        
        // Convert C string to Rust string
        let mut len = 0;
        while *text.add(len) != 0 {
            len += 1;
        }
        
        let slice = core::slice::from_raw_parts(text, len);
        if let Ok(s) = core::str::from_utf8(slice) {
            println!("{}", s);
        }
    }
}

#[no_mangle]
pub extern "C" fn kernel_syscall(num: u64, args: *const u64, arg_count: usize) -> i64 {
    // Call kernel syscall interface
    use crate::syscall::{Syscall, SyscallArgs, handle_syscall};
    
    let syscall_args = unsafe {
        let args_slice = core::slice::from_raw_parts(args, arg_count);
        SyscallArgs {
            arg0: args_slice.get(0).copied().unwrap_or(0),
            arg1: args_slice.get(1).copied().unwrap_or(0),
            arg2: args_slice.get(2).copied().unwrap_or(0),
            arg3: args_slice.get(3).copied().unwrap_or(0),
            arg4: args_slice.get(4).copied().unwrap_or(0),
            arg5: args_slice.get(5).copied().unwrap_or(0),
        }
    };
    
    let result = handle_syscall(num, syscall_args);
    result.value
}

#[no_mangle]
pub extern "C" fn kernel_read_file(path: *const u8, buffer: *mut u8, size: usize) -> isize {
    // Read file via VFS
    unsafe {
        // Convert path
        let mut len = 0;
        while *path.add(len) != 0 {
            len += 1;
        }
        
        let path_slice = core::slice::from_raw_parts(path, len);
        if let Ok(path_str) = core::str::from_utf8(path_slice) {
            use crate::fs::vfs;
            
            if let Ok(fd) = vfs::open(path_str, 0x01) {
                let buf_slice = core::slice::from_raw_parts_mut(buffer, size);
                if let Ok(bytes) = vfs::read(fd, buf_slice) {
                    vfs::close(fd).ok();
                    return bytes as isize;
                }
                vfs::close(fd).ok();
            }
        }
    }
    
    -1
}

#[no_mangle]
pub extern "C" fn kernel_capture_audio(duration_ms: u32, buffer: *mut u8, size: usize) -> isize {
    // Capture audio from microphone
    use crate::drivers::audio;
    
    unsafe {
        let buf_slice = core::slice::from_raw_parts_mut(buffer, size);
        if let Ok(bytes) = audio::capture_audio(buf_slice) {
            return bytes as isize;
        }
    }
    
    -1
}

#[no_mangle]
pub extern "C" fn kernel_capture_frame(buffer: *mut u8, size: usize) -> isize {
    // Capture video frame from camera
    use crate::drivers::camera;
    
    unsafe {
        if let Ok(frame) = camera::capture_frame() {
            let copy_size = core::cmp::min(frame.data.len(), size);
            let buf_slice = core::slice::from_raw_parts_mut(buffer, copy_size);
            buf_slice.copy_from_slice(&frame.data[..copy_size]);
            return copy_size as isize;
        }
    }
    
    -1
}

#[no_mangle]
pub extern "C" fn kernel_llm_query(
    prompt: *const u8,
    response: *mut u8,
    max_size: usize
) -> isize {
    // Query LLM
    unsafe {
        // Parse prompt
        let mut len = 0;
        while *prompt.add(len) != 0 {
            len += 1;
        }
        
        let prompt_slice = core::slice::from_raw_parts(prompt, len);
        if let Ok(prompt_str) = core::str::from_utf8(prompt_slice) {
            // TODO: Call LLM via syscall
            
            // For now, return placeholder
            let placeholder = b"LLM response here";
            let copy_size = core::cmp::min(placeholder.len(), max_size);
            let resp_slice = core::slice::from_raw_parts_mut(response, copy_size);
            resp_slice.copy_from_slice(&placeholder[..copy_size]);
            return copy_size as isize;
        }
    }
    
    -1
}

/// Initialize native Python subsystem
pub fn init() {
    println!("[NATIVE-PY] Initializing compiled Python code...");
    println!("[NATIVE-PY] No interpreter needed - pure machine code!");
}

/// Run compiled Python main
pub fn run_native_python() {
    println!("[NATIVE-PY] Executing compiled Python code...");
    
    unsafe {
        run_compiled_python();
    }
    
    println!("[NATIVE-PY] ✓ Python code executed natively");
}

/// Start native Python AI-OS
pub fn start_native_aios() -> i32 {
    println!("[NATIVE-PY] Starting native AI-OS...");
    
    unsafe {
        python_native_main()
    }
}
