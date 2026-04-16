//! Task scheduler and process management

use crate::println;
use alloc::vec::Vec;

pub fn init() {
    println!("[TASK] Initializing task scheduler...");
    
    // Initialize scheduler
    let scheduler = Scheduler::new();
    
    println!("[TASK] ✓ Task scheduler initialized");
}

pub struct Scheduler {
    tasks: Vec<Task>,
    current_task: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::new(),
            current_task: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn schedule(&mut self) {
        // Simple round-robin scheduling
        self.current_task = (self.current_task + 1) % self.tasks.len();
    }
}

pub struct Task {
    pub id: usize,
    pub name: &'static str,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
