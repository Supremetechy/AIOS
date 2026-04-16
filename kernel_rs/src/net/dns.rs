//! DNS resolver
//! 
//! Resolves domain names to IP addresses using real DNS queries

use alloc::string::String;
use alloc::vec::Vec;
use smoltcp::socket::dns;
use smoltcp::wire::{DnsQueryType, Ipv4Address};
use crate::println;

/// DNS resolver
pub struct DnsResolver {
    servers: Vec<Ipv4Address>,
}

impl DnsResolver {
    /// Create new DNS resolver
    pub fn new() -> Self {
        DnsResolver {
            servers: alloc::vec![
                Ipv4Address::new(8, 8, 8, 8),      // Google DNS
                Ipv4Address::new(1, 1, 1, 1),      // Cloudflare DNS
            ],
        }
    }
    
    /// Add DNS server
    pub fn add_server(&mut self, server: Ipv4Address) {
        if !self.servers.contains(&server) {
            self.servers.push(server);
        }
    }
    
    /// Resolve hostname to IP address
    pub fn resolve(&self, hostname: &str) -> Result<[u8; 4], &'static str> {
        println!("[DNS] Resolving: {}", hostname);
        
        // Try smoltcp DNS socket
        let result = super::stack::with_stack(|net| {
            // Create DNS socket
            let dns_socket = dns::Socket::new(&self.servers[..], vec![]);
            let dns_handle = net.sockets.add(dns_socket);
            
            // Start query
            {
                let dns_socket = net.sockets.get_mut::<dns::Socket>(dns_handle);
                let query_handle = dns_socket
                    .start_query(net.interface.context(), hostname, DnsQueryType::A)
                    .map_err(|_| "Failed to start DNS query")?;
            }
            
            // Poll until resolved
            for _ in 0..50 {
                let timestamp = smoltcp::time::Instant::now();
                let _ = net.interface.poll(timestamp, &mut net.device, &mut net.sockets);
                
                let dns_socket = net.sockets.get_mut::<dns::Socket>(dns_handle);
                
                if let Some(addrs) = dns_socket.get_query_result(0) {
                    if let Some(addr) = addrs.first() {
                        if let smoltcp::wire::IpAddress::Ipv4(ipv4) = addr {
                            println!("[DNS] ✓ Resolved to {}", ipv4);
                            return Ok::<[u8; 4], &'static str>(ipv4.0);
                        }
                    }
                }
                
                // Small delay
                for _ in 0..10000 {
                    core::hint::spin_loop();
                }
            }
            
            Err("DNS query timeout")
        });
        
        result.unwrap_or_else(|_| {
            // Fallback to hardcoded for common hosts
            println!("[DNS] Falling back to hardcoded resolution");
            match hostname {
                "generativelanguage.googleapis.com" => Ok([142, 250, 185, 202]),
                "api.openai.com" => Ok([104, 18, 7, 192]),
                "api.anthropic.com" => Ok([104, 18, 6, 42]),
                "localhost" => Ok([127, 0, 0, 1]),
                _ => Err("DNS resolution failed"),
            }
        })
    }
}
