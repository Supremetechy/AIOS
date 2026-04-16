//! TCP/IP stack implementation
//! 
//! Uses smoltcp for lightweight, no_std TCP/IP networking

use crate::println;
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
use smoltcp::time::Instant;
use spin::Mutex;
use alloc::vec::Vec;

/// TCP/IP stack state
pub struct NetworkStack {
    interface: Interface,
    sockets: SocketSet<'static>,
    device: super::smoltcp_device::E1000Adapter,
    ip_addr: Ipv4Address,
}

static STACK: Mutex<Option<NetworkStack>> = Mutex::new(None);

/// Initialize TCP/IP stack
pub fn init() -> Result<(), &'static str> {
    println!("[NET] Initializing TCP/IP stack with smoltcp...");
    
    // Get MAC address from device
    let mac = super::device::get_mac_address()
        .ok_or("No MAC address available")?;
    let ethernet_addr = EthernetAddress(mac);
    
    println!("[NET]   Ethernet: {}", ethernet_addr);
    
    // Create device adapter
    let device = super::smoltcp_device::E1000Adapter::new();
    
    // Create interface configuration
    let config = Config::new(ethernet_addr.into());
    let mut interface = Interface::new(config, &mut super::smoltcp_device::E1000Adapter::new(), Instant::now());
    
    // Set IP address (will be replaced by DHCP)
    let ip_addr = Ipv4Address::new(10, 0, 2, 15);
    let ip_cidr = IpCidr::new(IpAddress::from(ip_addr), 24);
    interface.update_ip_addrs(|addrs| {
        addrs.push(ip_cidr).ok();
    });
    
    // Add default route
    interface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(10, 0, 2, 2))
        .map_err(|_| "Failed to add default route")?;
    
    println!("[NET] ✓ TCP/IP stack initialized");
    println!("[NET]   IP: {}", ip_addr);
    println!("[NET]   Netmask: 255.255.255.0");
    println!("[NET]   Gateway: 10.0.2.2");
    
    // Create socket set
    let sockets = SocketSet::new(Vec::new());
    
    let mut stack_guard = STACK.lock();
    *stack_guard = Some(NetworkStack {
        interface,
        sockets,
        device,
        ip_addr,
    });
    
    Ok(())
}

/// Poll network stack (call from timer interrupt)
pub fn poll() {
    let mut stack = STACK.lock();
    if let Some(net) = stack.as_mut() {
        let timestamp = Instant::now();
        let _ = net.interface.poll(timestamp, &mut net.device, &mut net.sockets);
    }
}

/// Get current IP address
pub fn get_ip_address() -> Option<[u8; 4]> {
    let stack = STACK.lock();
    stack.as_ref().map(|net| net.ip_addr.0)
}

/// Get interface for socket operations
pub fn with_stack<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut NetworkStack) -> R,
{
    let mut stack = STACK.lock();
    stack.as_mut().map(f)
}
