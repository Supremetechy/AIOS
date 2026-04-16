//! HTTP client for REST API calls
//! 
//! Provides HTTP/1.1 client with TLS support

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::println;
use super::socket::TcpSocket;
use super::dns::DnsResolver;

/// HTTP method
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

/// HTTP request
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Create new GET request
    pub fn get(url: &str) -> Self {
        HttpRequest {
            method: HttpMethod::GET,
            url: String::from(url),
            headers: Vec::new(),
            body: None,
        }
    }
    
    /// Create new POST request
    pub fn post(url: &str, body: Vec<u8>) -> Self {
        HttpRequest {
            method: HttpMethod::POST,
            url: String::from(url),
            headers: Vec::new(),
            body: Some(body),
        }
    }
    
    /// Add header
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((String::from(key), String::from(value)));
        self
    }
}

/// HTTP response
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP client
pub struct HttpClient {
    dns: DnsResolver,
}

impl HttpClient {
    /// Create new HTTP client
    pub fn new() -> Self {
        HttpClient {
            dns: DnsResolver::new(),
        }
    }
    
    /// Send HTTP request
    pub fn send(&mut self, request: HttpRequest) -> Result<HttpResponse, &'static str> {
        println!("[HTTP] Sending request: {}", request.url);
        
        // Parse URL
        let (host, port, path, use_tls) = self.parse_url(&request.url)?;
        
        println!("[HTTP]   Host: {}", host);
        println!("[HTTP]   Port: {}", port);
        println!("[HTTP]   Path: {}", path);
        println!("[HTTP]   TLS: {}", use_tls);
        
        // Resolve hostname
        let ip = self.dns.resolve(&host)?;
        
        // Connect to server
        let mut socket = TcpSocket::new()?;
        socket.connect(ip, port)?;
        
        // Wrap in TLS if needed
        if use_tls {
            println!("[HTTP] Establishing TLS connection...");
            let mut tls_socket = super::tls::TlsSocket::new(socket, &host)?;
            tls_socket.connect()?;
            
            // Build and send request over TLS
            let http_request = self.build_request(&request, &host, &path)?;
            tls_socket.send(http_request.as_bytes())?;
            
            // Receive response
            let mut response_buffer = vec![0u8; 65536];
            let received = tls_socket.recv(&mut response_buffer)?;
            
            // Parse response
            self.parse_response(&response_buffer[..received])
        } else {
            // Plain HTTP
            let http_request = self.build_request(&request, &host, &path)?;
            socket.send(http_request.as_bytes())?;
            
            // Receive response
            let mut response_buffer = vec![0u8; 65536];
            let received = socket.recv(&mut response_buffer)?;
            
            // Parse response
            self.parse_response(&response_buffer[..received])
        }
    }
    
    /// Parse HTTP response
    fn parse_response(&self, data: &[u8]) -> Result<HttpResponse, &'static str> {
        let response_str = core::str::from_utf8(data)
            .map_err(|_| "Invalid UTF-8 in response")?;
        
        // Split headers and body
        let parts: Vec<&str> = response_str.splitn(2, "\r\n\r\n").collect();
        if parts.len() < 1 {
            return Err("Invalid HTTP response");
        }
        
        // Parse status line
        let lines: Vec<&str> = parts[0].lines().collect();
        if lines.is_empty() {
            return Err("Empty HTTP response");
        }
        
        let status_parts: Vec<&str> = lines[0].split_whitespace().collect();
        let status_code = if status_parts.len() >= 2 {
            status_parts[1].parse().unwrap_or(0)
        } else {
            0
        };
        
        // Parse headers
        let mut headers = Vec::new();
        for line in &lines[1..] {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();
                headers.push((String::from(key), String::from(value)));
            }
        }
        
        // Get body
        let body = if parts.len() > 1 {
            parts[1].as_bytes().to_vec()
        } else {
            Vec::new()
        };
        
        println!("[HTTP] ✓ Response: {} ({} bytes)", status_code, body.len());
        
        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }
    
    /// Parse URL into components
    fn parse_url(&self, url: &str) -> Result<(String, u16, String, bool), &'static str> {
        // Simple URL parser
        // Format: https://host:port/path or http://host/path
        
        let use_tls = url.starts_with("https://");
        let url = url.trim_start_matches("https://").trim_start_matches("http://");
        
        let parts: Vec<&str> = url.splitn(2, '/').collect();
        let host_port = parts[0];
        let path = if parts.len() > 1 {
            format!("/{}", parts[1])
        } else {
            String::from("/")
        };
        
        let (host, port) = if host_port.contains(':') {
            let hp: Vec<&str> = host_port.splitn(2, ':').collect();
            (String::from(hp[0]), hp[1].parse().unwrap_or(if use_tls { 443 } else { 80 }))
        } else {
            (String::from(host_port), if use_tls { 443 } else { 80 })
        };
        
        Ok((host, port, path, use_tls))
    }
    
    /// Build HTTP request string
    fn build_request(&self, request: &HttpRequest, host: &str, path: &str) -> Result<String, &'static str> {
        let method_str = match request.method {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
        };
        
        let mut req = format!("{} {} HTTP/1.1\r\n", method_str, path);
        req.push_str(&format!("Host: {}\r\n", host));
        req.push_str("User-Agent: AI-OS/1.0\r\n");
        req.push_str("Connection: close\r\n");
        
        // Add custom headers
        for (key, value) in &request.headers {
            req.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        // Add body if present
        if let Some(body) = &request.body {
            req.push_str(&format!("Content-Length: {}\r\n", body.len()));
            req.push_str("\r\n");
            // TODO: Append binary body (not UTF-8 safe)
        } else {
            req.push_str("\r\n");
        }
        
        Ok(req)
    }
}
