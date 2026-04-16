//! Interactive conversation manager for AI chat
//! 
//! Manages the conversation loop between user and AI, handling
//! keyboard input, context management, and response display.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::println;
use crate::ai::{query, get_system_context};

const MAX_HISTORY: usize = 10;
const MAX_RESPONSE_TOKENS: usize = 512;

/// Conversation history entry
struct Message {
    role: MessageRole,
    content: String,
}

enum MessageRole {
    System,
    User,
    Assistant,
}

/// Conversation state
pub struct Conversation {
    history: Vec<Message>,
    system_context: String,
}

impl Conversation {
    pub fn new() -> Self {
        let system_context = get_system_context();
        
        Conversation {
            history: Vec::new(),
            system_context,
        }
    }
    
    /// Add system message (context)
    pub fn add_system(&mut self, content: String) {
        self.history.push(Message {
            role: MessageRole::System,
            content,
        });
        
        // Keep history bounded
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
    }
    
    /// Add user message
    pub fn add_user(&mut self, content: String) {
        self.history.push(Message {
            role: MessageRole::User,
            content,
        });
        
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
    }
    
    /// Add assistant response
    pub fn add_assistant(&mut self, content: String) {
        self.history.push(Message {
            role: MessageRole::Assistant,
            content,
        });
        
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
    }
    
    /// Build prompt from conversation history
    pub fn build_prompt(&self, user_input: &str) -> String {
        let mut prompt = String::new();
        
        // Add system context
        prompt.push_str(&self.system_context);
        prompt.push_str("\n\n");
        
        // Add conversation history
        for msg in &self.history {
            match msg.role {
                MessageRole::System => {
                    prompt.push_str("System: ");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n\n");
                }
                MessageRole::User => {
                    prompt.push_str("User: ");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n\n");
                }
                MessageRole::Assistant => {
                    prompt.push_str("Assistant: ");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n\n");
                }
            }
        }
        
        // Add current user input
        prompt.push_str("User: ");
        prompt.push_str(user_input);
        prompt.push_str("\n\nAssistant:");
        
        prompt
    }
}

/// Main conversation loop
pub fn run_loop() -> ! {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              AI-OS Interactive Assistant                      ║");
    println!("║            Native On-Device AI (Gemma 2B)                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let mut conversation = Conversation::new();
    
    // Initial greeting
    println!("[AI] Initializing conversation...\n");
    
    let welcome_prompt = super::context::create_onboarding_prompt(&conversation.system_context);
    
    match query(&welcome_prompt, MAX_RESPONSE_TOKENS) {
        Ok(response) => {
            println!("Assistant: {}\n", response);
            conversation.add_assistant(response);
        }
        Err(e) => {
            println!("[ERROR] Failed to get AI response: {}", e);
        }
    }
    
    // Main interaction loop
    println!("Type your questions or commands. Type 'help' for available commands.\n");
    
    loop {
        // Display prompt
        print!("aios> ");
        
        // Read user input
        let user_input = read_line();
        
        if user_input.is_empty() {
            continue;
        }
        
        // Handle special commands
        match user_input.trim() {
            "exit" | "quit" => {
                println!("\nShutting down AI-OS. Goodbye!");
                shutdown();
            }
            "help" => {
                show_help();
                continue;
            }
            "clear" => {
                clear_screen();
                continue;
            }
            "status" => {
                show_status(&conversation);
                continue;
            }
            "sysinfo" => {
                println!("\n{}\n", conversation.system_context);
                continue;
            }
            _ => {}
        }
        
        // Get AI response
        let prompt = conversation.build_prompt(&user_input);
        
        println!("\n[Thinking...]");
        
        match query(&prompt, MAX_RESPONSE_TOKENS) {
            Ok(response) => {
                println!("\nAssistant: {}\n", response);
                conversation.add_user(user_input);
                conversation.add_assistant(response);
            }
            Err(e) => {
                println!("\n[ERROR] AI inference failed: {}\n", e);
            }
        }
    }
}

/// Read a line of input from keyboard
fn read_line() -> String {
    // TODO: Implement actual keyboard input buffering
    // For now, return placeholder
    // This will be replaced with proper keyboard interrupt handler integration
    
    use crate::println;
    
    // Placeholder: simulate user typing a question
    static mut DEMO_COUNTER: usize = 0;
    
    let demo_inputs = [
        "What hardware did you detect?",
        "How can I use the GPU for AI training?",
        "Show me system resources",
        "help",
    ];
    
    unsafe {
        let input = if DEMO_COUNTER < demo_inputs.len() {
            demo_inputs[DEMO_COUNTER]
        } else {
            "status"
        };
        
        println!("{}", input);
        DEMO_COUNTER += 1;
        
        // Simulate user delay
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
        
        String::from(input)
    }
}

/// Display help information
fn show_help() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                      AI-OS Commands                            ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  help       - Show this help message                          ║");
    println!("║  status     - Show conversation status                        ║");
    println!("║  sysinfo    - Display system information                      ║");
    println!("║  clear      - Clear the screen                                ║");
    println!("║  exit       - Shutdown AI-OS                                  ║");
    println!("║                                                                ║");
    println!("║  Or just type naturally to chat with the AI assistant!        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}

/// Show conversation status
fn show_status(conversation: &Conversation) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                   Conversation Status                          ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  Messages in history: {}                                      ║", 
             format!("{:>2}", conversation.history.len()));
    println!("║  Context size: {} bytes                                    ║", 
             format!("{:>6}", conversation.system_context.len()));
    println!("║  Model: Gemma 2B Instruction-Tuned                            ║");
    println!("║  Status: Active                                               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}

/// Clear screen
fn clear_screen() {
    use crate::vga_buffer::WRITER;
    WRITER.lock().clear_screen();
}

/// Shutdown system
fn shutdown() -> ! {
    use crate::ai::llama;
    
    println!("\n[SHUTDOWN] Cleaning up AI resources...");
    llama::shutdown();
    
    println!("[SHUTDOWN] Halting system...");
    
    loop {
        x86_64::instructions::hlt();
    }
}
