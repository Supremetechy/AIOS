//! Network subsystem for AI-OS
//! 
//! Provides TCP/IP networking using smoltcp for cloud API access

pub mod device;
pub mod stack;
pub mod socket;
pub mod dns;
pub mod http;
pub mod tls;
pub mod tls_optimized;
pub mod websocket;
pub mod smoltcp_device;
pub mod dhcp;

use crate::println;
use alloc::string::String;

/// Network configuration
pub struct NetworkConfig {
    pub use_dhcp: bool,
    pub static_ip: Option<[u8; 4]>,
    pub gateway: Option<[u8; 4]>,
    pub dns_servers: alloc::vec::Vec<[u8; 4]>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            use_dhcp: true,
            static_ip: None,
            gateway: None,
            dns_servers: alloc::vec![
                [8, 8, 8, 8],      // Google DNS
                [1, 1, 1, 1],      // Cloudflare DNS
            ],
        }
    }
}

/// Network subsystem state
pub struct NetworkSubsystem {
    initialized: bool,
    config: NetworkConfig,
}

static mut NETWORK: Option<NetworkSubsystem> = None;

/// Initialize network subsystem
pub fn init(config: NetworkConfig) -> Result<(), &'static str> {
    println!("[NET] Initializing network subsystem...");
    
    unsafe {
        NETWORK = Some(NetworkSubsystem {
            initialized: false,
            config,
        });
    }
    
    // Initialize network device
    device::init()?;
    
    // Initialize TCP/IP stack
    stack::init()?;
    
    unsafe {
        if let Some(net) = &mut NETWORK {
            net.initialized = true;
        }
    }
    
    println!("[NET] ✓ Network subsystem initialized");
    Ok(())
}

/// Check if network is available
pub fn is_available() -> bool {
    unsafe {
        NETWORK.as_ref().map(|n| n.initialized).unwrap_or(false)
    }
}

/// Get network configuration
pub fn get_config() -> Option<&'static NetworkConfig> {
    unsafe {
        NETWORK.as_ref().map(|n| &n.config)
    }
}
