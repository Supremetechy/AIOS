//! Socket abstraction for TCP/UDP
//! 
//! Provides high-level socket API over smoltcp

use alloc::vec::Vec;
use alloc::string::String;

/// Socket type
pub enum SocketType {
    TCP,
    UDP,
}

/// TCP socket
pub struct TcpSocket {
    // TODO: Wrap smoltcp TCP socket
    connected: bool,
}

impl TcpSocket {
    /// Create new TCP socket
    pub fn new() -> Result<Self, &'static str> {
        Ok(TcpSocket {
            connected: false,
        })
    }
    
    /// Connect to remote host
    pub fn connect(&mut self, ip: [u8; 4], port: u16) -> Result<(), &'static str> {
        crate::println!("[SOCKET] Connecting to {}.{}.{}.{}:{}...",
                       ip[0], ip[1], ip[2], ip[3], port);
        
        // TODO: Use smoltcp to establish connection
        
        self.connected = true;
        crate::println!("[SOCKET] ✓ Connected");
        Ok(())
    }
    
    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.connected {
            return Err("Socket not connected");
        }
        
        // TODO: Use smoltcp to send data
        
        Ok(data.len())
    }
    
    /// Receive data
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.connected {
            return Err("Socket not connected");
        }
        
        // TODO: Use smoltcp to receive data
        
        Ok(0)
    }
    
    /// Close socket
    pub fn close(&mut self) {
        self.connected = false;
    }
}

/// UDP socket
pub struct UdpSocket {
    // TODO: Wrap smoltcp UDP socket
}

impl UdpSocket {
    /// Create new UDP socket
    pub fn new() -> Result<Self, &'static str> {
        Ok(UdpSocket {})
    }
    
    /// Send datagram
    pub fn send_to(&mut self, _data: &[u8], _ip: [u8; 4], _port: u16) -> Result<usize, &'static str> {
        // TODO: Use smoltcp to send datagram
        Ok(0)
    }
    
    /// Receive datagram
    pub fn recv_from(&mut self, _buffer: &mut [u8]) -> Result<(usize, [u8; 4], u16), &'static str> {
        // TODO: Use smoltcp to receive datagram
        Err("Not implemented")
    }
}
