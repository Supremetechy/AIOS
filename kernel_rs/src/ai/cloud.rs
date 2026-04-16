//! Cloud API integration for hybrid AI mode
//! 
//! Provides access to cloud AI services (Gemini, OpenAI, etc.)
//! Falls back to on-device inference if network unavailable

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::println;

/// Cloud AI provider
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloudProvider {
    Gemini,
    OpenAI,
    Anthropic,
    Local, // On-device only
}

/// Cloud AI configuration
pub struct CloudConfig {
    pub provider: CloudProvider,
    pub api_key: Option<String>,
    pub endpoint: String,
    pub model: String,
    pub fallback_to_local: bool,
}

impl Default for CloudConfig {
    fn default() -> Self {
        CloudConfig {
            provider: CloudProvider::Gemini,
            api_key: None,
            endpoint: String::from("https://generativelanguage.googleapis.com/v1beta/models/"),
            model: String::from("gemini-2.0-flash-exp"),
            fallback_to_local: true,
        }
    }
}

impl CloudConfig {
    /// Create config for OpenAI
    pub fn openai(api_key: String) -> Self {
        CloudConfig {
            provider: CloudProvider::OpenAI,
            api_key: Some(api_key),
            endpoint: String::from("https://api.openai.com/v1/"),
            model: String::from("gpt-4o"),
            fallback_to_local: true,
        }
    }
    
    /// Create config for Anthropic
    pub fn anthropic(api_key: String) -> Self {
        CloudConfig {
            provider: CloudProvider::Anthropic,
            api_key: Some(api_key),
            endpoint: String::from("https://api.anthropic.com/v1/"),
            model: String::from("claude-3-5-sonnet-20241022"),
            fallback_to_local: true,
        }
    }
    
    /// Create config for Gemini
    pub fn gemini(api_key: String) -> Self {
        CloudConfig {
            provider: CloudProvider::Gemini,
            api_key: Some(api_key),
            endpoint: String::from("https://generativelanguage.googleapis.com/v1beta/models/"),
            model: String::from("gemini-2.0-flash-exp"),
            fallback_to_local: true,
        }
    }
}

/// Cloud AI client
pub struct CloudAI {
    config: CloudConfig,
    network_available: bool,
}

impl CloudAI {
    /// Create new cloud AI client
    pub fn new(config: CloudConfig) -> Self {
        let network_available = crate::net::is_available();
        
        println!("[CLOUD] Cloud AI client initialized");
        println!("[CLOUD]   Provider: {:?}", config.provider);
        println!("[CLOUD]   Model: {}", config.model);
        println!("[CLOUD]   Network: {}", if network_available { "Available" } else { "Unavailable" });
        println!("[CLOUD]   Fallback: {}", if config.fallback_to_local { "Enabled" } else { "Disabled" });
        
        CloudAI {
            config,
            network_available,
        }
    }
    
    /// Query cloud AI
    pub fn query(&mut self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        // Check network availability
        if !self.network_available {
            if self.config.fallback_to_local {
                println!("[CLOUD] Network unavailable, falling back to local AI");
                return self.query_local(prompt, max_tokens);
            } else {
                return Err("Network unavailable and fallback disabled");
            }
        }
        
        // Try cloud query
        match self.query_cloud(prompt, max_tokens) {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("[CLOUD] Cloud query failed: {}", e);
                if self.config.fallback_to_local {
                    println!("[CLOUD] Falling back to local AI");
                    self.query_local(prompt, max_tokens)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Query cloud API
    fn query_cloud(&mut self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        match self.config.provider {
            CloudProvider::Gemini => self.query_gemini(prompt, max_tokens),
            CloudProvider::OpenAI => self.query_openai(prompt, max_tokens),
            CloudProvider::Anthropic => self.query_anthropic(prompt, max_tokens),
            CloudProvider::Local => self.query_local(prompt, max_tokens),
        }
    }
    
    /// Query Google Gemini API
    fn query_gemini(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        println!("[GEMINI] Sending request to Gemini API...");
        
        // Build API endpoint
        let url = format!("{}{}:generateContent", self.config.endpoint, self.config.model);
        
        // Build JSON request body
        let request_body = self.build_gemini_request(prompt, max_tokens)?;
        
        // Create HTTP client
        let mut http_client = crate::net::http::HttpClient::new();
        
        // Create POST request
        let mut request = crate::net::http::HttpRequest::post(&url, request_body.into_bytes())
            .header("Content-Type", "application/json");
        
        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            request = request.header("x-goog-api-key", api_key);
        }
        
        // Send request
        let response = http_client.send(request)?;
        
        // Parse response
        if response.status_code == 200 {
            let response_text = self.parse_gemini_response(&response.body)?;
            println!("[GEMINI] ✓ Response received ({} bytes)", response_text.len());
            Ok(response_text)
        } else {
            Err("Gemini API request failed")
        }
    }
    
    /// Build Gemini API request JSON
    fn build_gemini_request(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        // Simplified JSON building (in production, use serde_json)
        let json = format!(
            r#"{{
                "contents": [{{
                    "parts": [{{
                        "text": "{}"
                    }}]
                }}],
                "generationConfig": {{
                    "maxOutputTokens": {},
                    "temperature": 0.7
                }}
            }}"#,
            prompt.replace('"', "\\\""),
            max_tokens
        );
        
        Ok(json)
    }
    
    /// Parse Gemini API response
    fn parse_gemini_response(&self, body: &[u8]) -> Result<String, &'static str> {
        // TODO: Proper JSON parsing
        // For now, extract text from response
        
        let body_str = core::str::from_utf8(body).map_err(|_| "Invalid UTF-8")?;
        
        // Simple extraction (in production, use serde_json)
        // Look for "text": "..." pattern
        if let Some(start) = body_str.find(r#""text":"#) {
            let start = start + 8; // Skip past "text":"
            if let Some(end) = body_str[start..].find('"') {
                return Ok(String::from(&body_str[start..start + end]));
            }
        }
        
        Err("Failed to parse Gemini response")
    }
    
    /// Query OpenAI API
    fn query_openai(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        println!("[OPENAI] Sending request to OpenAI API...");
        
        // Build API endpoint
        let url = format!("https://api.openai.com/v1/chat/completions");
        
        // Build JSON request body
        let request_body = self.build_openai_request(prompt, max_tokens)?;
        
        // Create HTTP client
        let mut http_client = crate::net::http::HttpClient::new();
        
        // Create POST request
        let mut request = crate::net::http::HttpRequest::post(&url, request_body.into_bytes())
            .header("Content-Type", "application/json");
        
        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", &format!("Bearer {}", api_key));
        }
        
        // Send request
        let response = http_client.send(request)?;
        
        // Parse response
        if response.status_code == 200 {
            let response_text = self.parse_openai_response(&response.body)?;
            println!("[OPENAI] ✓ Response received ({} bytes)", response_text.len());
            Ok(response_text)
        } else {
            Err("OpenAI API request failed")
        }
    }
    
    /// Build OpenAI API request JSON
    fn build_openai_request(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        let json = format!(
            r#"{{
                "model": "gpt-4o",
                "messages": [{{
                    "role": "user",
                    "content": "{}"
                }}],
                "max_tokens": {},
                "temperature": 0.7
            }}"#,
            prompt.replace('"', "\\\""),
            max_tokens
        );
        Ok(json)
    }
    
    /// Parse OpenAI API response
    fn parse_openai_response(&self, body: &[u8]) -> Result<String, &'static str> {
        let body_str = core::str::from_utf8(body).map_err(|_| "Invalid UTF-8")?;
        
        // Simple extraction: look for "content": "..." pattern
        if let Some(start) = body_str.find(r#""content":"#) {
            let start = start + 11; // Skip past "content":"
            if let Some(end) = body_str[start..].find('"') {
                return Ok(String::from(&body_str[start..start + end]));
            }
        }
        
        Err("Failed to parse OpenAI response")
    }
    
    /// Query Anthropic API
    fn query_anthropic(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        println!("[ANTHROPIC] Sending request to Anthropic API...");
        
        // Build API endpoint
        let url = format!("https://api.anthropic.com/v1/messages");
        
        // Build JSON request body
        let request_body = self.build_anthropic_request(prompt, max_tokens)?;
        
        // Create HTTP client
        let mut http_client = crate::net::http::HttpClient::new();
        
        // Create POST request with Anthropic-specific headers
        let mut request = crate::net::http::HttpRequest::post(&url, request_body.into_bytes())
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01");
        
        // Add API key if available
        if let Some(api_key) = &self.config.api_key {
            request = request.header("x-api-key", api_key);
        }
        
        // Send request
        let response = http_client.send(request)?;
        
        // Parse response
        if response.status_code == 200 {
            let response_text = self.parse_anthropic_response(&response.body)?;
            println!("[ANTHROPIC] ✓ Response received ({} bytes)", response_text.len());
            Ok(response_text)
        } else {
            Err("Anthropic API request failed")
        }
    }
    
    /// Build Anthropic API request JSON
    fn build_anthropic_request(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        let json = format!(
            r#"{{
                "model": "claude-3-5-sonnet-20241022",
                "messages": [{{
                    "role": "user",
                    "content": "{}"
                }}],
                "max_tokens": {}
            }}"#,
            prompt.replace('"', "\\\""),
            max_tokens
        );
        Ok(json)
    }
    
    /// Parse Anthropic API response
    fn parse_anthropic_response(&self, body: &[u8]) -> Result<String, &'static str> {
        let body_str = core::str::from_utf8(body).map_err(|_| "Invalid UTF-8")?;
        
        // Simple extraction: look for "text": "..." pattern
        if let Some(start) = body_str.find(r#""text":"#) {
            let start = start + 8; // Skip past "text":"
            if let Some(end) = body_str[start..].find('"') {
                return Ok(String::from(&body_str[start..start + end]));
            }
        }
        
        Err("Failed to parse Anthropic response")
    }
    
    /// Query local AI (fallback)
    fn query_local(&self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        println!("[CLOUD] Using local AI inference");
        super::llama::generate(prompt, max_tokens)
    }
}

/// Hybrid AI mode configuration
pub enum AIMode {
    /// Always use cloud API
    CloudOnly,
    /// Always use local inference
    LocalOnly,
    /// Try cloud first, fallback to local
    Hybrid,
    /// Use local for simple queries, cloud for complex
    Adaptive,
}

/// Hybrid AI manager with multi-provider support
pub struct HybridAI {
    mode: AIMode,
    providers: alloc::vec::Vec<CloudAI>,
    primary_provider: usize,
    local_available: bool,
}

impl HybridAI {
    /// Create new hybrid AI manager
    pub fn new(mode: AIMode, cloud_config: Option<CloudConfig>) -> Self {
        let mut providers = alloc::vec::Vec::new();
        
        if let Some(config) = cloud_config {
            providers.push(CloudAI::new(config));
        }
        
        let local_available = true; // Assume local model is loaded
        
        println!("[HYBRID] Hybrid AI mode initialized");
        println!("[HYBRID]   Mode: {:?}", mode);
        println!("[HYBRID]   Providers: {}", providers.len());
        
        HybridAI {
            mode,
            providers,
            primary_provider: 0,
            local_available,
        }
    }
    
    /// Add additional cloud provider
    pub fn add_provider(&mut self, config: CloudConfig) {
        println!("[HYBRID] Adding provider: {:?}", config.provider);
        self.providers.push(CloudAI::new(config));
    }
    
    /// Set primary provider
    pub fn set_primary(&mut self, index: usize) {
        if index < self.providers.len() {
            self.primary_provider = index;
            println!("[HYBRID] Primary provider set to index {}", index);
        }
    }
    
    /// Get provider by type
    pub fn get_provider_mut(&mut self, provider: CloudProvider) -> Option<&mut CloudAI> {
        self.providers.iter_mut().find(|p| p.config.provider == provider)
    }
    
    /// Query with automatic mode selection and provider failover
    pub fn query(&mut self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        match self.mode {
            AIMode::CloudOnly => {
                self.query_with_failover(prompt, max_tokens)
            }
            AIMode::LocalOnly => {
                super::llama::generate(prompt, max_tokens)
            }
            AIMode::Hybrid => {
                // Try cloud with failover, fallback to local
                self.query_with_failover(prompt, max_tokens)
                    .or_else(|_| super::llama::generate(prompt, max_tokens))
            }
            AIMode::Adaptive => {
                // Decide based on query complexity
                if self.is_complex_query(prompt) {
                    // Complex: try cloud with failover, fallback to local
                    self.query_with_failover(prompt, max_tokens)
                        .or_else(|_| super::llama::generate(prompt, max_tokens))
                } else {
                    // Simple: use local for speed
                    super::llama::generate(prompt, max_tokens)
                }
            }
        }
    }
    
    /// Query with provider failover
    fn query_with_failover(&mut self, prompt: &str, max_tokens: usize) -> Result<String, &'static str> {
        if self.providers.is_empty() {
            return Err("No cloud providers configured");
        }
        
        // Try primary provider first
        if let Some(provider) = self.providers.get_mut(self.primary_provider) {
            match provider.query_cloud(prompt, max_tokens) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    println!("[HYBRID] Primary provider failed: {}", e);
                }
            }
        }
        
        // Try other providers
        for (i, provider) in self.providers.iter_mut().enumerate() {
            if i == self.primary_provider {
                continue; // Already tried
            }
            
            println!("[HYBRID] Trying failover provider {}", i);
            match provider.query_cloud(prompt, max_tokens) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    println!("[HYBRID] Failover provider {} failed: {}", i, e);
                }
            }
        }
        
        Err("All cloud providers failed")
    }
    
    /// Determine if query is complex enough for cloud
    fn is_complex_query(&self, prompt: &str) -> bool {
        // Simple heuristics
        prompt.len() > 200 || 
        prompt.contains("analyze") ||
        prompt.contains("explain in detail") ||
        prompt.contains("compare")
    }
}

impl core::fmt::Debug for AIMode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            AIMode::CloudOnly => write!(f, "CloudOnly"),
            AIMode::LocalOnly => write!(f, "LocalOnly"),
            AIMode::Hybrid => write!(f, "Hybrid"),
            AIMode::Adaptive => write!(f, "Adaptive"),
        }
    }
}
