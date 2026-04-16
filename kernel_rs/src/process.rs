//! Process management with context switching
//! Designed for AI workloads (STT, TTS, vision, autonomous operations)

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use x86_64::structures::paging::PageTable;

/// Global process ID counter
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

/// Process states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,      // Ready to run
    Running,    // Currently executing
    Blocked,    // Waiting for I/O or event
    Sleeping,   // Timed sleep
    Zombie,     // Terminated, awaiting cleanup
}

/// Process priority levels (AI-focused)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,           // Background tasks
    Low = 1,            // File operations
    Normal = 2,         // Standard programs
    High = 3,           // Interactive (STT/TTS)
    RealTime = 4,       // Camera/audio processing
}

/// CPU register state for context switching
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RegisterState {
    // General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    // Instruction pointer
    pub rip: u64,
    
    // Flags
    pub rflags: u64,
    
    // Segment selectors
    pub cs: u64,
    pub ss: u64,
}

impl Default for RegisterState {
    fn default() -> Self {
        RegisterState {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0x202, // IF flag set
            cs: 0x08, ss: 0x10,
        }
    }
}

/// Process Control Block (PCB)
pub struct Process {
    pub pid: usize,
    pub parent_pid: Option<usize>,
    pub name: String,
    pub state: ProcessState,
    pub priority: Priority,
    pub registers: RegisterState,
    pub page_table: Option<*mut PageTable>,
    pub kernel_stack: Vec<u8>,
    pub user_stack: Vec<u8>,
    pub cpu_time: u64,          // Total CPU time used
    pub creation_time: u64,     // When created
    pub ai_context: AIContext,  // AI-specific context
}

/// AI-specific process context
#[derive(Debug, Clone)]
pub struct AIContext {
    pub is_voice_processing: bool,    // STT/TTS task
    pub is_vision_processing: bool,   // Camera/image task
    pub is_autonomous: bool,          // Autonomous operation
    pub io_priority: u8,              // I/O scheduling priority
    pub gpu_affinity: Option<usize>,  // Preferred GPU
}

impl Default for AIContext {
    fn default() -> Self {
        AIContext {
            is_voice_processing: false,
            is_vision_processing: false,
            is_autonomous: false,
            io_priority: 50,
            gpu_affinity: None,
        }
    }
}

impl Process {
    /// Create a new process
    pub fn new(name: String, entry_point: u64, priority: Priority) -> Self {
        let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);
        
        let mut registers = RegisterState::default();
        registers.rip = entry_point;
        
        // Allocate kernel stack (16KB)
        let kernel_stack = vec![0u8; 16 * 1024];
        registers.rsp = kernel_stack.as_ptr() as u64 + kernel_stack.len() as u64;
        
        // Allocate user stack (64KB for AI workloads)
        let user_stack = vec![0u8; 64 * 1024];
        
        Process {
            pid,
            parent_pid: None,
            name,
            state: ProcessState::Ready,
            priority,
            registers,
            page_table: None,
            kernel_stack,
            user_stack,
            cpu_time: 0,
            creation_time: 0,
            ai_context: AIContext::default(),
        }
    }
    
    /// Create AI voice processing task (high priority, real-time)
    pub fn new_voice_task(name: String, entry_point: u64) -> Self {
        let mut process = Self::new(name, entry_point, Priority::RealTime);
        process.ai_context.is_voice_processing = true;
        process.ai_context.io_priority = 90; // High I/O priority for audio
        process
    }
    
    /// Create AI vision processing task (camera input)
    pub fn new_vision_task(name: String, entry_point: u64, gpu_id: Option<usize>) -> Self {
        let mut process = Self::new(name, entry_point, Priority::High);
        process.ai_context.is_vision_processing = true;
        process.ai_context.gpu_affinity = gpu_id;
        process.ai_context.io_priority = 80;
        process
    }
    
    /// Create autonomous background task
    pub fn new_autonomous_task(name: String, entry_point: u64) -> Self {
        let mut process = Self::new(name, entry_point, Priority::Low);
        process.ai_context.is_autonomous = true;
        process.ai_context.io_priority = 30;
        process
    }
}

/// Process scheduler with AI-aware scheduling
pub struct Scheduler {
    ready_queue: VecDeque<Process>,
    blocked_queue: Vec<Process>,
    current_process: Option<Process>,
    total_processes: usize,
    
    // AI-specific scheduling
    voice_queue: VecDeque<Process>,    // Real-time voice tasks
    vision_queue: VecDeque<Process>,   // Camera/vision tasks
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            blocked_queue: Vec::new(),
            current_process: None,
            total_processes: 0,
            voice_queue: VecDeque::new(),
            vision_queue: VecDeque::new(),
        }
    }
    
    /// Add a process to the scheduler
    pub fn add_process(&mut self, process: Process) {
        // Route to appropriate queue based on AI context
        if process.ai_context.is_voice_processing {
            self.voice_queue.push_back(process);
        } else if process.ai_context.is_vision_processing {
            self.vision_queue.push_back(process);
        } else {
            self.ready_queue.push_back(process);
        }
        self.total_processes += 1;
    }
    
    /// Select next process to run (AI-aware scheduling)
    pub fn schedule(&mut self) -> Option<&mut Process> {
        // Priority order:
        // 1. Voice processing (real-time, interactive)
        // 2. Vision processing (camera input)
        // 3. Regular processes by priority
        
        if let Some(process) = self.voice_queue.front_mut() {
            process.state = ProcessState::Running;
            return Some(process);
        }
        
        if let Some(process) = self.vision_queue.front_mut() {
            process.state = ProcessState::Running;
            return Some(process);
        }
        
        if let Some(process) = self.ready_queue.front_mut() {
            process.state = ProcessState::Running;
            return Some(process);
        }
        
        None
    }
    
    /// Context switch to next process
    pub fn context_switch(&mut self) {
        if let Some(current) = self.current_process.take() {
            // Save current process
            if current.state == ProcessState::Running {
                let mut process = current;
                process.state = ProcessState::Ready;
                
                // Return to appropriate queue
                if process.ai_context.is_voice_processing {
                    self.voice_queue.push_back(process);
                } else if process.ai_context.is_vision_processing {
                    self.vision_queue.push_back(process);
                } else {
                    self.ready_queue.push_back(process);
                }
            }
        }
        
        // Get next process
        if let Some(next) = self.schedule() {
            unsafe {
                switch_to_process(next);
            }
        }
    }
    
    /// Block current process (waiting for I/O, camera frame, etc.)
    pub fn block_current(&mut self, reason: &str) {
        if let Some(mut current) = self.current_process.take() {
            current.state = ProcessState::Blocked;
            self.blocked_queue.push(current);
        }
    }
    
    /// Unblock a process (I/O completed, data ready, etc.)
    pub fn unblock_process(&mut self, pid: usize) {
        if let Some(index) = self.blocked_queue.iter().position(|p| p.pid == pid) {
            let mut process = self.blocked_queue.remove(index);
            process.state = ProcessState::Ready;
            
            if process.ai_context.is_voice_processing {
                self.voice_queue.push_back(process);
            } else if process.ai_context.is_vision_processing {
                self.vision_queue.push_back(process);
            } else {
                self.ready_queue.push_back(process);
            }
        }
    }
    
    /// Get process by PID
    pub fn get_process(&self, pid: usize) -> Option<&Process> {
        if let Some(ref current) = self.current_process {
            if current.pid == pid {
                return Some(current);
            }
        }
        
        self.ready_queue.iter().find(|p| p.pid == pid)
            .or_else(|| self.blocked_queue.iter().find(|p| p.pid == pid))
            .or_else(|| self.voice_queue.iter().find(|p| p.pid == pid))
            .or_else(|| self.vision_queue.iter().find(|p| p.pid == pid))
    }
    
    /// Kill a process
    pub fn kill_process(&mut self, pid: usize) {
        // Remove from all queues
        self.ready_queue.retain(|p| p.pid != pid);
        self.blocked_queue.retain(|p| p.pid != pid);
        self.voice_queue.retain(|p| p.pid != pid);
        self.vision_queue.retain(|p| p.pid != pid);
        
        if let Some(ref current) = self.current_process {
            if current.pid == pid {
                self.current_process = None;
            }
        }
    }
    
    /// Statistics
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            total: self.total_processes,
            ready: self.ready_queue.len(),
            blocked: self.blocked_queue.len(),
            voice: self.voice_queue.len(),
            vision: self.vision_queue.len(),
        }
    }
}

pub struct SchedulerStats {
    pub total: usize,
    pub ready: usize,
    pub blocked: usize,
    pub voice: usize,
    pub vision: usize,
}

/// Global scheduler instance
static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Initialize process scheduler
pub fn init() {
    use crate::println;
    println!("[PROCESS] Initializing process scheduler...");
    
    // Create init process (PID 1)
    let init_process = Process::new(
        String::from("init"),
        0, // Will be set later
        Priority::High
    );
    
    SCHEDULER.lock().add_process(init_process);
    
    println!("[PROCESS] ✓ Process scheduler initialized");
}

/// Context switch (low-level, called from assembly)
#[no_mangle]
pub unsafe extern "C" fn switch_to_process(process: *mut Process) {
    if process.is_null() {
        return;
    }
    
    let proc = &mut *process;
    
    // Switch page tables if needed
    if let Some(page_table) = proc.page_table {
        let page_table_addr = page_table as u64;
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) page_table_addr,
            options(nostack, preserves_flags)
        );
    }
    
    // Restore registers and jump to process
    restore_context(&proc.registers);
}

/// Restore CPU context (assembly stub)
#[naked]
unsafe extern "C" fn restore_context(_regs: &RegisterState) {
    core::arch::asm!(
        // Restore general purpose registers
        "mov rax, [rdi + 0]",
        "mov rbx, [rdi + 8]",
        "mov rcx, [rdi + 16]",
        "mov rdx, [rdi + 24]",
        "mov rsi, [rdi + 32]",
        // ... (full register restore)
        
        // Restore stack pointer
        "mov rsp, [rdi + 56]",
        
        // Jump to saved instruction pointer
        "jmp [rdi + 128]",
        options(noreturn)
    );
}

/// Public API
pub fn spawn(name: String, entry: u64, priority: Priority) -> usize {
    let process = Process::new(name, entry, priority);
    let pid = process.pid;
    SCHEDULER.lock().add_process(process);
    pid
}

pub fn spawn_voice_task(name: String, entry: u64) -> usize {
    let process = Process::new_voice_task(name, entry);
    let pid = process.pid;
    SCHEDULER.lock().add_process(process);
    pid
}

pub fn spawn_vision_task(name: String, entry: u64, gpu: Option<usize>) -> usize {
    let process = Process::new_vision_task(name, entry, gpu);
    let pid = process.pid;
    SCHEDULER.lock().add_process(process);
    pid
}

pub fn spawn_autonomous_task(name: String, entry: u64) -> usize {
    let process = Process::new_autonomous_task(name, entry);
    let pid = process.pid;
    SCHEDULER.lock().add_process(process);
    pid
}

pub fn yield_cpu() {
    SCHEDULER.lock().context_switch();
}

pub fn block_on_io() {
    SCHEDULER.lock().block_current("I/O");
}

pub fn unblock(pid: usize) {
    SCHEDULER.lock().unblock_process(pid);
}

pub fn kill(pid: usize) {
    SCHEDULER.lock().kill_process(pid);
}

pub fn get_stats() -> SchedulerStats {
    SCHEDULER.lock().stats()
}
