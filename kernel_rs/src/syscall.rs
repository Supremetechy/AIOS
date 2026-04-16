//! System call interface for AI-OS
//! Provides user-space programs access to kernel services
//! Focused on AI workloads: file I/O, audio/video, network, GPU access

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

/// System call numbers (AI-focused)
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum Syscall {
    // Process management
    Exit = 0,
    Fork = 1,
    Exec = 2,
    Wait = 3,
    GetPID = 4,
    Kill = 5,
    
    // File I/O
    Open = 10,
    Close = 11,
    Read = 12,
    Write = 13,
    Seek = 14,
    Stat = 15,
    Unlink = 16,
    
    // Directory operations
    Mkdir = 20,
    Rmdir = 21,
    Chdir = 22,
    Getcwd = 23,
    Readdir = 24,
    
    // Memory
    Mmap = 30,
    Munmap = 31,
    Brk = 32,
    
    // AI-specific: Audio (STT/TTS)
    AudioOpen = 100,
    AudioClose = 101,
    AudioRead = 102,      // Capture from microphone
    AudioWrite = 103,     // Play TTS output
    AudioConfig = 104,    // Configure sample rate, channels
    
    // AI-specific: Video (Camera)
    CameraOpen = 110,
    CameraClose = 111,
    CameraCapture = 112,  // Capture frame
    CameraConfig = 113,   // Set resolution, FPS
    CameraStream = 114,   // Start video stream
    
    // AI-specific: GPU
    GPUAllocate = 120,
    GPUFree = 121,
    GPUExecute = 122,     // Execute kernel/shader
    GPUMemcpy = 123,      // Copy to/from GPU
    GPUQuery = 124,       // Query GPU info
    
    // AI-specific: Network (for autonomous browsing)
    NetConnect = 130,
    NetSend = 131,
    NetReceive = 132,
    NetClose = 133,
    HTTPGet = 134,        // High-level HTTP
    HTTPPost = 135,
    
    // AI-specific: LLM Integration
    LLMQuery = 140,       // Send query to LLM
    LLMStream = 141,      // Stream LLM response
    LLMContext = 142,     // Manage context
    
    // AI-specific: Autonomous operations
    AutoCreateFile = 150, // Autonomous file creation
    AutoBrowse = 151,     // Autonomous web browsing
    AutoExecute = 152,    // Execute autonomous task
    
    // System info
    GetTime = 200,
    Sleep = 201,
    GetSystemInfo = 202,
}

impl Syscall {
    pub fn from_number(num: u64) -> Option<Self> {
        match num {
            0 => Some(Syscall::Exit),
            1 => Some(Syscall::Fork),
            2 => Some(Syscall::Exec),
            10 => Some(Syscall::Open),
            12 => Some(Syscall::Read),
            13 => Some(Syscall::Write),
            100 => Some(Syscall::AudioOpen),
            102 => Some(Syscall::AudioRead),
            103 => Some(Syscall::AudioWrite),
            110 => Some(Syscall::CameraOpen),
            112 => Some(Syscall::CameraCapture),
            120 => Some(Syscall::GPUAllocate),
            140 => Some(Syscall::LLMQuery),
            150 => Some(Syscall::AutoCreateFile),
            _ => None,
        }
    }
}

/// System call arguments
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallArgs {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
}

/// System call return value
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallResult {
    pub value: i64,  // Return value or error code
    pub error: bool, // True if error occurred
}

impl SyscallResult {
    pub fn ok(value: i64) -> Self {
        SyscallResult { value, error: false }
    }
    
    pub fn err(errno: i64) -> Self {
        SyscallResult { value: errno, error: true }
    }
}

/// Error codes
#[repr(i64)]
pub enum ErrorCode {
    Success = 0,
    InvalidSyscall = -1,
    InvalidArgument = -2,
    PermissionDenied = -3,
    NotFound = -4,
    AlreadyExists = -5,
    NoMemory = -12,
    IOError = -5,
    NotSupported = -38,
    DeviceBusy = -16,
}

/// Main system call handler
pub fn handle_syscall(syscall_num: u64, args: SyscallArgs) -> SyscallResult {
    let syscall = match Syscall::from_number(syscall_num) {
        Some(s) => s,
        None => return SyscallResult::err(ErrorCode::InvalidSyscall as i64),
    };
    
    match syscall {
        // Process management
        Syscall::Exit => sys_exit(args.arg0 as i32),
        Syscall::Fork => sys_fork(),
        Syscall::GetPID => sys_getpid(),
        
        // File I/O
        Syscall::Open => sys_open(args.arg0, args.arg1, args.arg2),
        Syscall::Close => sys_close(args.arg0 as i32),
        Syscall::Read => sys_read(args.arg0 as i32, args.arg1, args.arg2),
        Syscall::Write => sys_write(args.arg0 as i32, args.arg1, args.arg2),
        
        // AI: Audio (STT/TTS)
        Syscall::AudioOpen => sys_audio_open(args.arg0 as u32),
        Syscall::AudioRead => sys_audio_read(args.arg0 as i32, args.arg1, args.arg2),
        Syscall::AudioWrite => sys_audio_write(args.arg0 as i32, args.arg1, args.arg2),
        
        // AI: Camera
        Syscall::CameraOpen => sys_camera_open(args.arg0 as u32),
        Syscall::CameraCapture => sys_camera_capture(args.arg0 as i32, args.arg1, args.arg2),
        
        // AI: GPU
        Syscall::GPUAllocate => sys_gpu_allocate(args.arg0),
        Syscall::GPUExecute => sys_gpu_execute(args.arg0, args.arg1, args.arg2),
        
        // AI: LLM
        Syscall::LLMQuery => sys_llm_query(args.arg0, args.arg1, args.arg2),
        
        // AI: Autonomous
        Syscall::AutoCreateFile => sys_auto_create_file(args.arg0, args.arg1),
        Syscall::AutoBrowse => sys_auto_browse(args.arg0),
        
        // Network
        Syscall::HTTPGet => sys_http_get(args.arg0, args.arg1, args.arg2),
        
        _ => SyscallResult::err(ErrorCode::NotSupported as i64),
    }
}

// ============================================================================
// Process Management
// ============================================================================

fn sys_exit(code: i32) -> SyscallResult {
    println!("[SYSCALL] exit({})", code);
    // TODO: Terminate current process
    SyscallResult::ok(0)
}

fn sys_fork() -> SyscallResult {
    println!("[SYSCALL] fork()");
    // TODO: Clone current process
    SyscallResult::ok(0) // Return child PID
}

fn sys_getpid() -> SyscallResult {
    // TODO: Return current process PID
    SyscallResult::ok(1)
}

// ============================================================================
// File I/O
// ============================================================================

fn sys_open(path_ptr: u64, flags: u64, mode: u64) -> SyscallResult {
    println!("[SYSCALL] open(path={:#x}, flags={}, mode={})", path_ptr, flags, mode);
    // TODO: Open file, return file descriptor
    SyscallResult::ok(3) // Return fd
}

fn sys_close(fd: i32) -> SyscallResult {
    println!("[SYSCALL] close({})", fd);
    // TODO: Close file descriptor
    SyscallResult::ok(0)
}

fn sys_read(fd: i32, buf_ptr: u64, count: u64) -> SyscallResult {
    println!("[SYSCALL] read(fd={}, buf={:#x}, count={})", fd, buf_ptr, count);
    // TODO: Read from file
    SyscallResult::ok(0) // Return bytes read
}

fn sys_write(fd: i32, buf_ptr: u64, count: u64) -> SyscallResult {
    println!("[SYSCALL] write(fd={}, buf={:#x}, count={})", fd, buf_ptr, count);
    
    // Special case: stdout (fd=1) and stderr (fd=2)
    if fd == 1 || fd == 2 {
        unsafe {
            let buf = core::slice::from_raw_parts(buf_ptr as *const u8, count as usize);
            if let Ok(s) = core::str::from_utf8(buf) {
                print!("{}", s);
            }
        }
        return SyscallResult::ok(count as i64);
    }
    
    // TODO: Write to file
    SyscallResult::ok(count as i64)
}

// ============================================================================
// AI: Audio System (STT/TTS)
// ============================================================================

fn sys_audio_open(device_id: u32) -> SyscallResult {
    println!("[SYSCALL] audio_open(device={})", device_id);
    // TODO: Open audio device (microphone or speaker)
    // Return audio device handle
    SyscallResult::ok(100) // Audio device fd
}

fn sys_audio_read(fd: i32, buf_ptr: u64, frames: u64) -> SyscallResult {
    println!("[SYSCALL] audio_read(fd={}, buf={:#x}, frames={})", fd, buf_ptr, frames);
    // TODO: Capture audio from microphone
    // For STT: capture speech → feed to speech recognition
    SyscallResult::ok(frames as i64)
}

fn sys_audio_write(fd: i32, buf_ptr: u64, frames: u64) -> SyscallResult {
    println!("[SYSCALL] audio_write(fd={}, buf={:#x}, frames={})", fd, buf_ptr, frames);
    // TODO: Play audio through speaker
    // For TTS: receive synthesized speech → play
    SyscallResult::ok(frames as i64)
}

// ============================================================================
// AI: Camera System (Vision)
// ============================================================================

fn sys_camera_open(camera_id: u32) -> SyscallResult {
    println!("[SYSCALL] camera_open(id={})", camera_id);
    // TODO: Open camera device
    // Initialize V4L2 or similar interface
    SyscallResult::ok(110) // Camera device fd
}

fn sys_camera_capture(fd: i32, buf_ptr: u64, buf_size: u64) -> SyscallResult {
    println!("[SYSCALL] camera_capture(fd={}, buf={:#x}, size={})", fd, buf_ptr, buf_size);
    // TODO: Capture single frame from camera
    // Return frame data for vision processing
    SyscallResult::ok(buf_size as i64)
}

// ============================================================================
// AI: GPU Access
// ============================================================================

fn sys_gpu_allocate(size: u64) -> SyscallResult {
    println!("[SYSCALL] gpu_allocate(size={})", size);
    // TODO: Allocate GPU memory
    // Return GPU memory pointer
    SyscallResult::ok(0x1000_0000) // GPU memory address
}

fn sys_gpu_execute(kernel_ptr: u64, args_ptr: u64, grid_size: u64) -> SyscallResult {
    println!("[SYSCALL] gpu_execute(kernel={:#x}, args={:#x}, grid={})", 
             kernel_ptr, args_ptr, grid_size);
    // TODO: Execute GPU kernel (CUDA/OpenCL)
    SyscallResult::ok(0)
}

// ============================================================================
// AI: LLM Integration
// ============================================================================

fn sys_llm_query(prompt_ptr: u64, response_buf: u64, max_tokens: u64) -> SyscallResult {
    println!("[SYSCALL] llm_query(prompt={:#x}, buf={:#x}, tokens={})", 
             prompt_ptr, response_buf, max_tokens);
    // TODO: Send query to LLM
    // This interfaces with the Python AI layer or embedded model
    SyscallResult::ok(0) // Return response length
}

// ============================================================================
// AI: Autonomous Operations
// ============================================================================

fn sys_auto_create_file(path_ptr: u64, content_ptr: u64) -> SyscallResult {
    println!("[SYSCALL] auto_create_file(path={:#x}, content={:#x})", path_ptr, content_ptr);
    // TODO: Autonomous file creation
    // AI decides file name and content, creates it
    SyscallResult::ok(0)
}

fn sys_auto_browse(url_ptr: u64) -> SyscallResult {
    println!("[SYSCALL] auto_browse(url={:#x})", url_ptr);
    // TODO: Autonomous web browsing
    // AI browses web, extracts information
    SyscallResult::ok(0)
}

// ============================================================================
// Network (for autonomous browsing)
// ============================================================================

fn sys_http_get(url_ptr: u64, response_buf: u64, buf_size: u64) -> SyscallResult {
    println!("[SYSCALL] http_get(url={:#x}, buf={:#x}, size={})", url_ptr, response_buf, buf_size);
    // TODO: HTTP GET request
    // For autonomous web browsing
    SyscallResult::ok(0) // Return response length
}

// ============================================================================
// System call entry point (called from interrupt handler)
// ============================================================================

use x86_64::structures::idt::InterruptStackFrame;

#[no_mangle]
pub extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    unsafe {
        // Get syscall number and arguments from registers
        let syscall_num: u64;
        let args = SyscallArgs {
            arg0: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            arg4: 0,
            arg5: 0,
        };
        
        core::arch::asm!(
            "mov {}, rax",
            out(reg) syscall_num,
        );
        
        // Handle the syscall
        let result = handle_syscall(syscall_num, args);
        
        // Return result in RAX
        core::arch::asm!(
            "mov rax, {}",
            in(reg) result.value,
        );
    }
}

/// Initialize system call interface
pub fn init() {
    println!("[SYSCALL] Initializing system call interface...");
    
    // Register syscall handler in IDT (interrupt 0x80)
    // This will be done in interrupts.rs
    
    println!("[SYSCALL] ✓ System calls initialized");
    println!("[SYSCALL]   Process: exit, fork, exec, getpid");
    println!("[SYSCALL]   File I/O: open, close, read, write");
    println!("[SYSCALL]   Audio: audio_open, audio_read, audio_write (STT/TTS)");
    println!("[SYSCALL]   Camera: camera_open, camera_capture (Vision)");
    println!("[SYSCALL]   GPU: gpu_allocate, gpu_execute (AI Acceleration)");
    println!("[SYSCALL]   LLM: llm_query (AI Integration)");
    println!("[SYSCALL]   Autonomous: auto_create_file, auto_browse");
}

// ============================================================================
// User-space syscall wrappers (to be used by programs)
// ============================================================================

/// User-space syscall wrapper macro
#[macro_export]
macro_rules! syscall {
    ($num:expr) => {{
        let result: i64;
        unsafe {
            core::arch::asm!(
                "mov rax, {}",
                "int 0x80",
                "mov {}, rax",
                in(reg) $num,
                out(reg) result,
            );
        }
        result
    }};
    
    ($num:expr, $arg0:expr) => {{
        let result: i64;
        unsafe {
            core::arch::asm!(
                "mov rax, {}",
                "mov rdi, {}",
                "int 0x80",
                "mov {}, rax",
                in(reg) $num,
                in(reg) $arg0,
                out(reg) result,
            );
        }
        result
    }};
}
