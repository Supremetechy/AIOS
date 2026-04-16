//! DHCP client for automatic IP configuration
//! 
//! Implements DHCP protocol for acquiring network configuration

use smoltcp::socket::dhcpv4;
use smoltcp::wire::Ipv4Address;
use crate::println;

/// DHCP client state
pub struct DhcpClient {
    configured: bool,
    ip_addr: Option<Ipv4Address>,
    gateway: Option<Ipv4Address>,
    dns_servers: alloc::vec::Vec<Ipv4Address>,
}

impl DhcpClient {
    pub fn new() -> Self {
        DhcpClient {
            configured: false,
            ip_addr: None,
            gateway: None,
            dns_servers: alloc::vec::Vec::new(),
        }
    }
    
    /// Start DHCP discovery
    pub fn discover(&mut self) -> Result<(), &'static str> {
        println!("[DHCP] Starting DHCP discovery...");
        
        // Use smoltcp's DHCP socket
        super::stack::with_stack(|net| {
            // Create DHCP socket
            let dhcp_socket = dhcpv4::Socket::new();
            let dhcp_handle = net.sockets.add(dhcp_socket);
            
            // Poll until configured
            for _ in 0..100 {
                let timestamp = smoltcp::time::Instant::now();
                let _ = net.interface.poll(timestamp, &mut net.device, &mut net.sockets);
                
                let dhcp_socket = net.sockets.get_mut::<dhcpv4::Socket>(dhcp_handle);
                
                if let Some(config) = dhcp_socket.config() {
                    self.ip_addr = Some(config.address.address());
                    self.gateway = config.router;
                    
                    if let Some(dns) = config.dns_servers.first() {
                        self.dns_servers.push(*dns);
                    }
                    
                    self.configured = true;
                    
                    println!("[DHCP] ✓ Configuration received");
                    println!("[DHCP]   IP: {}", config.address);
                    if let Some(gw) = config.router {
                        println!("[DHCP]   Gateway: {}", gw);
                    }
                    for dns in &config.dns_servers {
                        if !dns.is_unspecified() {
                            println!("[DHCP]   DNS: {}", dns);
                        }
                    }
                    
                    return Ok(());
                }
                
                // Small delay
                for _ in 0..100000 {
                    core::hint::spin_loop();
                }
            }
            
            Err("DHCP timeout")
        }).unwrap_or(Err("Network stack not initialized"))
    }
    
    /// Get assigned IP address
    pub fn ip_address(&self) -> Option<Ipv4Address> {
        self.ip_addr
    }
    
    /// Get gateway
    pub fn gateway(&self) -> Option<Ipv4Address> {
        self.gateway
    }
    
    /// Get DNS servers
    pub fn dns_servers(&self) -> &[Ipv4Address] {
        &self.dns_servers
    }
    
    /// Is configured
    pub fn is_configured(&self) -> bool {
        self.configured
    }
}
