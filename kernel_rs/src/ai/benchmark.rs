//! Performance benchmarking for AI inference
//! 
//! Provides tools to benchmark local and cloud AI performance

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::println;

/// Benchmark result
pub struct BenchmarkResult {
    pub provider: String,
    pub prompt_length: usize,
    pub response_length: usize,
    pub latency_ms: u64,
    pub tokens_per_second: f32,
    pub success: bool,
    pub error: Option<String>,
}

/// Benchmark suite
pub struct BenchmarkSuite {
    results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        BenchmarkSuite {
            results: Vec::new(),
        }
    }
    
    /// Benchmark local AI inference
    pub fn bench_local(&mut self, prompt: &str, max_tokens: usize) -> Result<(), &'static str> {
        println!("[BENCH] Benchmarking local AI...");
        
        let start = get_timestamp_ms();
        
        let result = super::llama::generate(prompt, max_tokens);
        
        let end = get_timestamp_ms();
        let latency = end - start;
        
        match result {
            Ok(response) => {
                let tokens = estimate_tokens(&response);
                let tokens_per_sec = if latency > 0 {
                    (tokens as f32 / latency as f32) * 1000.0
                } else {
                    0.0
                };
                
                self.results.push(BenchmarkResult {
                    provider: String::from("Local (Gemma 2B)"),
                    prompt_length: prompt.len(),
                    response_length: response.len(),
                    latency_ms: latency,
                    tokens_per_second: tokens_per_sec,
                    success: true,
                    error: None,
                });
                
                println!("[BENCH] ✓ Local: {} ms, {:.1} tok/s", latency, tokens_per_sec);
                Ok(())
            }
            Err(e) => {
                self.results.push(BenchmarkResult {
                    provider: String::from("Local (Gemma 2B)"),
                    prompt_length: prompt.len(),
                    response_length: 0,
                    latency_ms: latency,
                    tokens_per_second: 0.0,
                    success: false,
                    error: Some(String::from(e)),
                });
                
                println!("[BENCH] ✗ Local failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Benchmark cloud AI (Gemini)
    pub fn bench_gemini(&mut self, prompt: &str, max_tokens: usize, api_key: &str) -> Result<(), &'static str> {
        println!("[BENCH] Benchmarking Gemini API...");
        
        let config = super::cloud::CloudConfig::gemini(String::from(api_key));
        let mut client = super::cloud::CloudAI::new(config);
        
        let start = get_timestamp_ms();
        let result = client.query(prompt, max_tokens);
        let end = get_timestamp_ms();
        let latency = end - start;
        
        match result {
            Ok(response) => {
                let tokens = estimate_tokens(&response);
                let tokens_per_sec = if latency > 0 {
                    (tokens as f32 / latency as f32) * 1000.0
                } else {
                    0.0
                };
                
                self.results.push(BenchmarkResult {
                    provider: String::from("Gemini Flash"),
                    prompt_length: prompt.len(),
                    response_length: response.len(),
                    latency_ms: latency,
                    tokens_per_second: tokens_per_sec,
                    success: true,
                    error: None,
                });
                
                println!("[BENCH] ✓ Gemini: {} ms, {:.1} tok/s", latency, tokens_per_sec);
                Ok(())
            }
            Err(e) => {
                self.results.push(BenchmarkResult {
                    provider: String::from("Gemini Flash"),
                    prompt_length: prompt.len(),
                    response_length: 0,
                    latency_ms: latency,
                    tokens_per_second: 0.0,
                    success: false,
                    error: Some(String::from(e)),
                });
                
                println!("[BENCH] ✗ Gemini failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Benchmark cloud AI (OpenAI)
    pub fn bench_openai(&mut self, prompt: &str, max_tokens: usize, api_key: &str) -> Result<(), &'static str> {
        println!("[BENCH] Benchmarking OpenAI API...");
        
        let config = super::cloud::CloudConfig::openai(String::from(api_key));
        let mut client = super::cloud::CloudAI::new(config);
        
        let start = get_timestamp_ms();
        let result = client.query(prompt, max_tokens);
        let end = get_timestamp_ms();
        let latency = end - start;
        
        match result {
            Ok(response) => {
                let tokens = estimate_tokens(&response);
                let tokens_per_sec = if latency > 0 {
                    (tokens as f32 / latency as f32) * 1000.0
                } else {
                    0.0
                };
                
                self.results.push(BenchmarkResult {
                    provider: String::from("OpenAI GPT-4o"),
                    prompt_length: prompt.len(),
                    response_length: response.len(),
                    latency_ms: latency,
                    tokens_per_second: tokens_per_sec,
                    success: true,
                    error: None,
                });
                
                println!("[BENCH] ✓ OpenAI: {} ms, {:.1} tok/s", latency, tokens_per_sec);
                Ok(())
            }
            Err(e) => {
                self.results.push(BenchmarkResult {
                    provider: String::from("OpenAI GPT-4o"),
                    prompt_length: prompt.len(),
                    response_length: 0,
                    latency_ms: latency,
                    tokens_per_second: 0.0,
                    success: false,
                    error: Some(String::from(e)),
                });
                
                println!("[BENCH] ✗ OpenAI failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Benchmark cloud AI (Anthropic)
    pub fn bench_anthropic(&mut self, prompt: &str, max_tokens: usize, api_key: &str) -> Result<(), &'static str> {
        println!("[BENCH] Benchmarking Anthropic API...");
        
        let config = super::cloud::CloudConfig::anthropic(String::from(api_key));
        let mut client = super::cloud::CloudAI::new(config);
        
        let start = get_timestamp_ms();
        let result = client.query(prompt, max_tokens);
        let end = get_timestamp_ms();
        let latency = end - start;
        
        match result {
            Ok(response) => {
                let tokens = estimate_tokens(&response);
                let tokens_per_sec = if latency > 0 {
                    (tokens as f32 / latency as f32) * 1000.0
                } else {
                    0.0
                };
                
                self.results.push(BenchmarkResult {
                    provider: String::from("Anthropic Claude"),
                    prompt_length: prompt.len(),
                    response_length: response.len(),
                    latency_ms: latency,
                    tokens_per_second: tokens_per_sec,
                    success: true,
                    error: None,
                });
                
                println!("[BENCH] ✓ Anthropic: {} ms, {:.1} tok/s", latency, tokens_per_sec);
                Ok(())
            }
            Err(e) => {
                self.results.push(BenchmarkResult {
                    provider: String::from("Anthropic Claude"),
                    prompt_length: prompt.len(),
                    response_length: 0,
                    latency_ms: latency,
                    tokens_per_second: 0.0,
                    success: false,
                    error: Some(String::from(e)),
                });
                
                println!("[BENCH] ✗ Anthropic failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Print benchmark results
    pub fn print_results(&self) {
        println!("\n╔════════════════════════════════════════════════════════════════════════╗");
        println!("║                      AI Performance Benchmark                          ║");
        println!("╠════════════════════════════════════════════════════════════════════════╣");
        println!("║ Provider          │ Latency  │ Tok/s   │ Status                       ║");
        println!("╠═══════════════════╪══════════╪═════════╪══════════════════════════════╣");
        
        for result in &self.results {
            let status = if result.success { "✓ OK" } else { "✗ FAIL" };
            println!("║ {:<17} │ {:>6} ms │ {:>6.1} │ {:<28} ║",
                     truncate(&result.provider, 17),
                     result.latency_ms,
                     result.tokens_per_second,
                     status);
        }
        
        println!("╚════════════════════════════════════════════════════════════════════════╝\n");
        
        // Summary
        let successful = self.results.iter().filter(|r| r.success).count();
        let total = self.results.len();
        let avg_latency: u64 = if successful > 0 {
            self.results.iter()
                .filter(|r| r.success)
                .map(|r| r.latency_ms)
                .sum::<u64>() / successful as u64
        } else {
            0
        };
        
        println!("Summary:");
        println!("  Successful: {}/{}", successful, total);
        println!("  Average Latency: {} ms", avg_latency);
        
        if let Some(fastest) = self.results.iter().filter(|r| r.success).min_by_key(|r| r.latency_ms) {
            println!("  Fastest: {} ({} ms)", fastest.provider, fastest.latency_ms);
        }
    }
}

/// Estimate token count (rough approximation)
fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~4 characters per token
    text.len() / 4
}

/// Get current timestamp in milliseconds
fn get_timestamp_ms() -> u64 {
    // TODO: Use actual timer
    // For now, return placeholder
    0
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}
