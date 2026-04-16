//! WebSocket protocol implementation
//! 
//! Provides WebSocket support for real-time bidirectional communication

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::convert::TryInto;
use crate::println;
use super::tls::TlsSocket;
use super::http::HttpClient;

/// WebSocket opcode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

/// WebSocket frame
pub struct WebSocketFrame {
    pub fin: bool,
    pub opcode: OpCode,
    pub payload: Vec<u8>,
}

impl WebSocketFrame {
    /// Create text frame
    pub fn text(data: &str) -> Self {
        WebSocketFrame {
            fin: true,
            opcode: OpCode::Text,
            payload: data.as_bytes().to_vec(),
        }
    }
    
    /// Create binary frame
    pub fn binary(data: Vec<u8>) -> Self {
        WebSocketFrame {
            fin: true,
            opcode: OpCode::Binary,
            payload: data,
        }
    }
    
    /// Create close frame
    pub fn close() -> Self {
        WebSocketFrame {
            fin: true,
            opcode: OpCode::Close,
            payload: Vec::new(),
        }
    }
    
    /// Encode frame to bytes
    pub fn encode(&self, masked: bool) -> Vec<u8> {
        let mut frame = Vec::new();
        
        // Byte 0: FIN + RSV + OpCode
        let mut byte0 = self.opcode as u8;
        if self.fin {
            byte0 |= 0x80;
        }
        frame.push(byte0);
        
        // Byte 1: MASK + Payload length
        let payload_len = self.payload.len();
        let mut byte1 = 0u8;
        if masked {
            byte1 |= 0x80;
        }
        
        if payload_len < 126 {
            byte1 |= payload_len as u8;
            frame.push(byte1);
        } else if payload_len < 65536 {
            byte1 |= 126;
            frame.push(byte1);
            frame.extend_from_slice(&(payload_len as u16).to_be_bytes());
        } else {
            byte1 |= 127;
            frame.push(byte1);
            frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
        }
        
        // Masking key (if masked)
        let mask_key = if masked {
            let key = [0x12, 0x34, 0x56, 0x78]; // TODO: Use random key
            frame.extend_from_slice(&key);
            Some(key)
        } else {
            None
        };
        
        // Payload (masked if needed)
        if let Some(key) = mask_key {
            for (i, byte) in self.payload.iter().enumerate() {
                frame.push(byte ^ key[i % 4]);
            }
        } else {
            frame.extend_from_slice(&self.payload);
        }
        
        frame
    }
    
    /// Decode frame from bytes
    pub fn decode(data: &[u8]) -> Result<(Self, usize), &'static str> {
        if data.len() < 2 {
            return Err("Incomplete frame");
        }
        
        let fin = (data[0] & 0x80) != 0;
        let opcode = match data[0] & 0x0F {
            0x0 => OpCode::Continuation,
            0x1 => OpCode::Text,
            0x2 => OpCode::Binary,
            0x8 => OpCode::Close,
            0x9 => OpCode::Ping,
            0xA => OpCode::Pong,
            _ => return Err("Invalid opcode"),
        };
        
        let masked = (data[1] & 0x80) != 0;
        let mut payload_len = (data[1] & 0x7F) as usize;
        let mut offset = 2;
        
        if payload_len == 126 {
            if data.len() < 4 {
                return Err("Incomplete frame");
            }
            payload_len = u16::from_be_bytes(data[2..4].try_into().unwrap()) as usize;
            offset = 4;
        } else if payload_len == 127 {
            if data.len() < 10 {
                return Err("Incomplete frame");
            }
            payload_len = u64::from_be_bytes(data[2..10].try_into().unwrap()) as usize;
            offset = 10;
        }
        
        let mask_key = if masked {
            if data.len() < offset + 4 {
                return Err("Incomplete frame");
            }
            let key = [data[offset], data[offset+1], data[offset+2], data[offset+3]];
            offset += 4;
            Some(key)
        } else {
            None
        };
        
        if data.len() < offset + payload_len {
            return Err("Incomplete payload");
        }
        
        let mut payload = Vec::with_capacity(payload_len);
        if let Some(key) = mask_key {
            for i in 0..payload_len {
                payload.push(data[offset + i] ^ key[i % 4]);
            }
        } else {
            payload.extend_from_slice(&data[offset..offset + payload_len]);
        }
        
        Ok((WebSocketFrame { fin, opcode, payload }, offset + payload_len))
    }
}

/// WebSocket connection
pub struct WebSocket {
    socket: TlsSocket,
    url: String,
    connected: bool,
}

impl WebSocket {
    /// Create new WebSocket connection
    pub fn connect(url: &str) -> Result<Self, &'static str> {
        println!("[WS] Connecting to {}...", url);
        
        // Parse URL
        let (host, port, path, use_tls) = Self::parse_url(url)?;
        
        if !use_tls {
            return Err("Only WSS (WebSocket Secure) is supported");
        }
        
        // Resolve hostname
        let dns = super::dns::DnsResolver::new();
        let ip = dns.resolve(&host)?;
        
        // Create TCP connection
        let tcp_socket = super::socket::TcpSocket::new()?;
        tcp_socket.connect(ip, port)?;
        
        // Wrap in TLS
        let mut tls_socket = TlsSocket::new(tcp_socket, &host)?;
        tls_socket.connect()?;
        
        // Perform WebSocket handshake
        Self::handshake(&mut tls_socket, &host, &path)?;
        
        println!("[WS] ✓ WebSocket connected to {}", url);
        
        Ok(WebSocket {
            socket: tls_socket,
            url: String::from(url),
            connected: true,
        })
    }
    
    /// Perform WebSocket handshake
    fn handshake(socket: &mut TlsSocket, host: &str, path: &str) -> Result<(), &'static str> {
        // Generate WebSocket key (should be random)
        let ws_key = "dGhlIHNhbXBsZSBub25jZQ=="; // Base64 encoded
        
        // Build handshake request
        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {}\r\n\
             Sec-WebSocket-Version: 13\r\n\
             \r\n",
            path, host, ws_key
        );
        
        // Send handshake
        socket.send(request.as_bytes())?;
        
        // Receive response
        let mut response = vec![0u8; 4096];
        let received = socket.recv(&mut response)?;
        
        let response_str = core::str::from_utf8(&response[..received])
            .map_err(|_| "Invalid handshake response")?;
        
        // Verify handshake response
        if !response_str.contains("HTTP/1.1 101") ||
           !response_str.contains("Upgrade: websocket") {
            return Err("WebSocket handshake failed");
        }
        
        println!("[WS] ✓ Handshake complete");
        Ok(())
    }
    
    /// Send text message
    pub fn send_text(&mut self, text: &str) -> Result<(), &'static str> {
        if !self.connected {
            return Err("WebSocket not connected");
        }
        
        let frame = WebSocketFrame::text(text);
        let data = frame.encode(true); // Client must mask
        
        self.socket.send(&data)?;
        Ok(())
    }
    
    /// Send binary message
    pub fn send_binary(&mut self, data: Vec<u8>) -> Result<(), &'static str> {
        if !self.connected {
            return Err("WebSocket not connected");
        }
        
        let frame = WebSocketFrame::binary(data);
        let encoded = frame.encode(true);
        
        self.socket.send(&encoded)?;
        Ok(())
    }
    
    /// Receive message
    pub fn recv_message(&mut self) -> Result<WebSocketFrame, &'static str> {
        if !self.connected {
            return Err("WebSocket not connected");
        }
        
        let mut buffer = vec![0u8; 65536];
        let received = self.socket.recv(&mut buffer)?;
        
        let (frame, _) = WebSocketFrame::decode(&buffer[..received])?;
        
        // Handle control frames
        match frame.opcode {
            OpCode::Close => {
                self.connected = false;
                println!("[WS] Connection closed by server");
            }
            OpCode::Ping => {
                // Respond with pong
                let pong = WebSocketFrame {
                    fin: true,
                    opcode: OpCode::Pong,
                    payload: frame.payload.clone(),
                };
                let _ = self.socket.send(&pong.encode(true));
            }
            _ => {}
        }
        
        Ok(frame)
    }
    
    /// Close connection
    pub fn close(&mut self) -> Result<(), &'static str> {
        if self.connected {
            let frame = WebSocketFrame::close();
            let _ = self.socket.send(&frame.encode(true));
            self.connected = false;
            println!("[WS] Connection closed");
        }
        Ok(())
    }
    
    /// Parse WebSocket URL
    fn parse_url(url: &str) -> Result<(String, u16, String, bool), &'static str> {
        let use_tls = url.starts_with("wss://");
        let url = url.trim_start_matches("wss://").trim_start_matches("ws://");
        
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
}
