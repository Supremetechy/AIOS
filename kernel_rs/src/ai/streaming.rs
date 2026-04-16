//! Streaming response support for AI APIs
//! 
//! Provides real-time token-by-token streaming for AI responses

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::println;

/// Streaming response parser
pub struct StreamParser {
    buffer: Vec<u8>,
    partial_token: String,
    tokens_received: usize,
}

impl StreamParser {
    /// Create new stream parser
    pub fn new() -> Self {
        StreamParser {
            buffer: Vec::new(),
            partial_token: String::new(),
            tokens_received: 0,
        }
    }
    
    /// Parse incoming chunk
    pub fn parse_chunk(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(chunk);
        
        let mut tokens = Vec::new();
        
        // Try to parse complete tokens from buffer
        while let Some(token) = self.extract_next_token() {
            tokens.push(token);
            self.tokens_received += 1;
        }
        
        tokens
    }
    
    /// Extract next complete token from buffer
    fn extract_next_token(&mut self) -> Option<String> {
        // Look for SSE (Server-Sent Events) format: "data: {...}\n\n"
        let buffer_str = core::str::from_utf8(&self.buffer).ok()?;
        
        if let Some(data_start) = buffer_str.find("data: ") {
            if let Some(data_end) = buffer_str[data_start..].find("\n\n") {
                let data_line = &buffer_str[data_start + 6..data_start + data_end];
                
                // Remove processed data from buffer
                let bytes_to_remove = data_start + data_end + 2;
                self.buffer.drain(..bytes_to_remove);
                
                // Parse JSON token
                if let Some(token) = self.extract_token_from_json(data_line) {
                    return Some(token);
                }
            }
        }
        
        None
    }
    
    /// Extract token text from JSON data
    fn extract_token_from_json(&self, json: &str) -> Option<String> {
        // Simple JSON parsing for streaming responses
        // Look for "text": "..." or "content": "..."
        
        if let Some(start) = json.find(r#""text":"#).or_else(|| json.find(r#""content":"#)) {
            let start = start + if json.contains(r#""text":"#) { 8 } else { 11 };
            
            if let Some(end) = json[start..].find('"') {
                let token = &json[start..start + end];
                // Unescape JSON string
                return Some(token.replace("\\n", "\n").replace("\\\"", "\""));
            }
        }
        
        None
    }
    
    /// Get total tokens received
    pub fn tokens_received(&self) -> usize {
        self.tokens_received
    }
    
    /// Check if stream is complete
    pub fn is_complete(&self) -> bool {
        let buffer_str = core::str::from_utf8(&self.buffer).unwrap_or("");
        buffer_str.contains("[DONE]") || buffer_str.contains("\"finish_reason\"")
    }
}

/// Streaming callback trait
pub trait StreamCallback {
    fn on_token(&mut self, token: &str);
    fn on_complete(&mut self);
    fn on_error(&mut self, error: &str);
}

/// Simple console streaming callback
pub struct ConsoleStreamCallback {
    total_tokens: usize,
}

impl ConsoleStreamCallback {
    pub fn new() -> Self {
        ConsoleStreamCallback {
            total_tokens: 0,
        }
    }
}

impl StreamCallback for ConsoleStreamCallback {
    fn on_token(&mut self, token: &str) {
        print!("{}", token);
        self.total_tokens += 1;
    }
    
    fn on_complete(&mut self) {
        println!("\n[STREAM] Complete ({} tokens)", self.total_tokens);
    }
    
    fn on_error(&mut self, error: &str) {
        println!("\n[STREAM] Error: {}", error);
    }
}

/// Streaming AI client
pub struct StreamingAI {
    provider: super::cloud::CloudProvider,
}

impl StreamingAI {
    /// Create new streaming AI client
    pub fn new(provider: super::cloud::CloudProvider) -> Self {
        StreamingAI { provider }
    }
    
    /// Stream query with callback
    pub fn stream_query<F>(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        api_key: &str,
        mut callback: F,
    ) -> Result<(), &'static str>
    where
        F: StreamCallback,
    {
        match self.provider {
            super::cloud::CloudProvider::Gemini => {
                self.stream_gemini(prompt, max_tokens, api_key, callback)
            }
            super::cloud::CloudProvider::OpenAI => {
                self.stream_openai(prompt, max_tokens, api_key, callback)
            }
            super::cloud::CloudProvider::Anthropic => {
                self.stream_anthropic(prompt, max_tokens, api_key, callback)
            }
            _ => Err("Provider does not support streaming"),
        }
    }
    
    /// Stream from Gemini API
    fn stream_gemini<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        api_key: &str,
        mut callback: F,
    ) -> Result<(), &'static str>
    where
        F: StreamCallback,
    {
        println!("[STREAM] Starting Gemini stream...");
        
        // Build streaming request URL
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:streamGenerateContent?key={}",
            api_key
        );
        
        // Build request body
        let body = format!(
            r#"{{
                "contents": [{{
                    "parts": [{{ "text": "{}" }}]
                }}],
                "generationConfig": {{
                    "maxOutputTokens": {},
                    "temperature": 0.7
                }}
            }}"#,
            prompt.replace('"', "\\\""),
            max_tokens
        );
        
        // TODO: Use WebSocket or SSE for real streaming
        // For now, simulate streaming by chunking response
        
        callback.on_complete();
        Ok(())
    }
    
    /// Stream from OpenAI API
    fn stream_openai<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        api_key: &str,
        mut callback: F,
    ) -> Result<(), &'static str>
    where
        F: StreamCallback,
    {
        println!("[STREAM] Starting OpenAI stream...");
        
        // OpenAI uses SSE for streaming
        let url = "https://api.openai.com/v1/chat/completions";
        
        let body = format!(
            r#"{{
                "model": "gpt-4o",
                "messages": [{{ "role": "user", "content": "{}" }}],
                "max_tokens": {},
                "stream": true
            }}"#,
            prompt.replace('"', "\\\""),
            max_tokens
        );
        
        // TODO: Use SSE parser for streaming
        
        callback.on_complete();
        Ok(())
    }
    
    /// Stream from Anthropic API
    fn stream_anthropic<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        api_key: &str,
        mut callback: F,
    ) -> Result<(), &'static str>
    where
        F: StreamCallback,
    {
        println!("[STREAM] Starting Anthropic stream...");
        
        // Anthropic also uses SSE for streaming
        let url = "https://api.anthropic.com/v1/messages";
        
        let body = format!(
            r#"{{
                "model": "claude-3-5-sonnet-20241022",
                "messages": [{{ "role": "user", "content": "{}" }}],
                "max_tokens": {},
                "stream": true
            }}"#,
            prompt.replace('"', "\\\""),
            max_tokens
        );
        
        // TODO: Use SSE parser for streaming
        
        callback.on_complete();
        Ok(())
    }
}

/// WebSocket-based streaming for real-time AI
pub struct WebSocketStreaming {
    ws: Option<crate::net::websocket::WebSocket>,
}

impl WebSocketStreaming {
    /// Create new WebSocket streaming client
    pub fn new() -> Self {
        WebSocketStreaming { ws: None }
    }
    
    /// Connect to streaming endpoint
    pub fn connect(&mut self, url: &str) -> Result<(), &'static str> {
        println!("[WS-STREAM] Connecting to {}...", url);
        
        let ws = crate::net::websocket::WebSocket::connect(url)?;
        self.ws = Some(ws);
        
        println!("[WS-STREAM] ✓ Connected");
        Ok(())
    }
    
    /// Send prompt and stream responses
    pub fn stream_prompt<F>(
        &mut self,
        prompt: &str,
        mut callback: F,
    ) -> Result<(), &'static str>
    where
        F: StreamCallback,
    {
        if let Some(ws) = &mut self.ws {
            // Send prompt
            ws.send_text(prompt)?;
            
            // Receive streaming responses
            let mut parser = StreamParser::new();
            
            loop {
                let frame = ws.recv_message()?;
                
                match frame.opcode {
                    crate::net::websocket::OpCode::Text => {
                        let text = core::str::from_utf8(&frame.payload)
                            .map_err(|_| "Invalid UTF-8")?;
                        
                        // Parse tokens from chunk
                        let tokens = parser.parse_chunk(text.as_bytes());
                        
                        for token in tokens {
                            callback.on_token(&token);
                        }
                        
                        // Check if stream is complete
                        if parser.is_complete() {
                            callback.on_complete();
                            break;
                        }
                    }
                    crate::net::websocket::OpCode::Close => {
                        callback.on_complete();
                        break;
                    }
                    _ => {}
                }
            }
            
            Ok(())
        } else {
            Err("WebSocket not connected")
        }
    }
    
    /// Close connection
    pub fn close(&mut self) -> Result<(), &'static str> {
        if let Some(ws) = &mut self.ws {
            ws.close()?;
        }
        self.ws = None;
        Ok(())
    }
}
