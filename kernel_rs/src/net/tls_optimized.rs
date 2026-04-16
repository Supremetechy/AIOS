//! Optimized TLS with session resumption and connection pooling
//! 
//! Provides performance-optimized TLS connections with caching

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;
use crate::println;
use super::tls::TlsSocket;
use super::socket::TcpSocket;

/// TLS session cache entry
struct SessionCacheEntry {
    session_id: Vec<u8>,
    master_secret: Vec<u8>,
    cipher_suite: u16,
    created_at: u64,
    last_used: u64,
}

/// TLS session cache for resumption
pub struct TlsSessionCache {
    cache: BTreeMap<String, SessionCacheEntry>,
    max_entries: usize,
    max_age_seconds: u64,
}

impl TlsSessionCache {
    /// Create new session cache
    pub fn new(max_entries: usize, max_age_seconds: u64) -> Self {
        TlsSessionCache {
            cache: BTreeMap::new(),
            max_entries,
            max_age_seconds,
        }
    }
    
    /// Store session for hostname
    pub fn store(&mut self, hostname: &str, session_id: Vec<u8>, master_secret: Vec<u8>, cipher_suite: u16) {
        let now = get_timestamp_seconds();
        
        // Evict old entries if cache is full
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }
        
        let entry = SessionCacheEntry {
            session_id,
            master_secret,
            cipher_suite,
            created_at: now,
            last_used: now,
        };
        
        self.cache.insert(String::from(hostname), entry);
        println!("[TLS-CACHE] Stored session for {}", hostname);
    }
    
    /// Retrieve session for hostname
    pub fn retrieve(&mut self, hostname: &str) -> Option<(Vec<u8>, Vec<u8>, u16)> {
        let now = get_timestamp_seconds();
        
        if let Some(entry) = self.cache.get_mut(hostname) {
            // Check if session expired
            if now - entry.created_at > self.max_age_seconds {
                println!("[TLS-CACHE] Session expired for {}", hostname);
                self.cache.remove(hostname);
                return None;
            }
            
            // Update last used time
            entry.last_used = now;
            
            println!("[TLS-CACHE] Retrieved session for {} ({}s old)", 
                     hostname, now - entry.created_at);
            
            Some((entry.session_id.clone(), entry.master_secret.clone(), entry.cipher_suite))
        } else {
            None
        }
    }
    
    /// Evict oldest entry
    fn evict_oldest(&mut self) {
        if let Some((hostname, _)) = self.cache.iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(k, v)| (k.clone(), v.last_used))
        {
            self.cache.remove(&hostname);
            println!("[TLS-CACHE] Evicted session for {}", hostname);
        }
    }
    
    /// Clear all sessions
    pub fn clear(&mut self) {
        self.cache.clear();
        println!("[TLS-CACHE] Cleared all sessions");
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_entries)
    }
}

/// Global TLS session cache
static TLS_SESSION_CACHE: Mutex<Option<TlsSessionCache>> = Mutex::new(None);

/// Initialize TLS session cache
pub fn init_session_cache() {
    let mut cache = TLS_SESSION_CACHE.lock();
    *cache = Some(TlsSessionCache::new(
        100,      // Max 100 cached sessions
        3600,     // Sessions valid for 1 hour
    ));
    println!("[TLS-CACHE] Session cache initialized");
}

/// Connection pool entry
struct PooledConnection {
    socket: TlsSocket,
    hostname: String,
    created_at: u64,
    last_used: u64,
    use_count: usize,
}

/// Connection pool for reusing TLS connections
pub struct ConnectionPool {
    connections: Vec<PooledConnection>,
    max_connections: usize,
    max_idle_time: u64,
    max_connection_age: u64,
}

impl ConnectionPool {
    /// Create new connection pool
    pub fn new(max_connections: usize, max_idle_time: u64, max_connection_age: u64) -> Self {
        ConnectionPool {
            connections: Vec::new(),
            max_connections,
            max_idle_time,
            max_connection_age,
        }
    }
    
    /// Get connection from pool or create new
    pub fn get_connection(&mut self, hostname: &str, port: u16, ip: [u8; 4]) 
        -> Result<TlsSocket, &'static str> 
    {
        let now = get_timestamp_seconds();
        
        // Try to find existing connection
        if let Some(idx) = self.find_connection(hostname, now) {
            let mut conn = self.connections.remove(idx);
            conn.last_used = now;
            conn.use_count += 1;
            
            println!("[POOL] Reusing connection to {} (#{} uses)", hostname, conn.use_count);
            
            return Ok(conn.socket);
        }
        
        // Create new connection
        println!("[POOL] Creating new connection to {}:{}", hostname, port);
        
        let tcp_socket = TcpSocket::new()?;
        tcp_socket.connect(ip, port)?;
        
        let tls_socket = TlsSocket::new(tcp_socket, hostname)?;
        tls_socket.connect()?;
        
        Ok(tls_socket)
    }
    
    /// Return connection to pool
    pub fn return_connection(&mut self, socket: TlsSocket, hostname: &str) {
        let now = get_timestamp_seconds();
        
        // Check if pool is full
        if self.connections.len() >= self.max_connections {
            // Close oldest connection
            if let Some(idx) = self.connections.iter()
                .enumerate()
                .min_by_key(|(_, c)| c.last_used)
                .map(|(i, _)| i)
            {
                self.connections.remove(idx);
            }
        }
        
        let conn = PooledConnection {
            socket,
            hostname: String::from(hostname),
            created_at: now,
            last_used: now,
            use_count: 1,
        };
        
        self.connections.push(conn);
        println!("[POOL] Returned connection to {} (pool size: {})", hostname, self.connections.len());
    }
    
    /// Find usable connection
    fn find_connection(&mut self, hostname: &str, now: u64) -> Option<usize> {
        // Clean up expired connections first
        self.cleanup_expired(now);
        
        // Find matching connection
        self.connections.iter()
            .enumerate()
            .find(|(_, conn)| {
                conn.hostname == hostname &&
                now - conn.last_used < self.max_idle_time &&
                now - conn.created_at < self.max_connection_age
            })
            .map(|(idx, _)| idx)
    }
    
    /// Clean up expired connections
    fn cleanup_expired(&mut self, now: u64) {
        let initial_len = self.connections.len();
        
        self.connections.retain(|conn| {
            let idle_time = now - conn.last_used;
            let age = now - conn.created_at;
            
            idle_time < self.max_idle_time && age < self.max_connection_age
        });
        
        let removed = initial_len - self.connections.len();
        if removed > 0 {
            println!("[POOL] Cleaned up {} expired connections", removed);
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.connections.len(), self.max_connections)
    }
    
    /// Clear all connections
    pub fn clear(&mut self) {
        self.connections.clear();
        println!("[POOL] Cleared all connections");
    }
}

/// Global connection pool
static CONNECTION_POOL: Mutex<Option<ConnectionPool>> = Mutex::new(None);

/// Initialize connection pool
pub fn init_connection_pool() {
    let mut pool = CONNECTION_POOL.lock();
    *pool = Some(ConnectionPool::new(
        20,       // Max 20 pooled connections
        300,      // Idle timeout: 5 minutes
        3600,     // Max connection age: 1 hour
    ));
    println!("[POOL] Connection pool initialized");
}

/// Get connection from global pool
pub fn get_pooled_connection(hostname: &str, port: u16, ip: [u8; 4]) 
    -> Result<TlsSocket, &'static str> 
{
    let mut pool = CONNECTION_POOL.lock();
    if let Some(p) = pool.as_mut() {
        p.get_connection(hostname, port, ip)
    } else {
        Err("Connection pool not initialized")
    }
}

/// Return connection to global pool
pub fn return_pooled_connection(socket: TlsSocket, hostname: &str) {
    let mut pool = CONNECTION_POOL.lock();
    if let Some(p) = pool.as_mut() {
        p.return_connection(socket, hostname);
    }
}

/// Get timestamp in seconds
fn get_timestamp_seconds() -> u64 {
    // TODO: Use actual system timer
    // For now, return placeholder
    0
}

/// Print optimization statistics
pub fn print_stats() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              TLS Optimization Statistics                      ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    
    // Session cache stats
    let cache = TLS_SESSION_CACHE.lock();
    if let Some(c) = cache.as_ref() {
        let (cached, max_cache) = c.stats();
        println!("║  Session Cache: {}/{} entries                           ║", cached, max_cache);
    }
    
    // Connection pool stats
    let pool = CONNECTION_POOL.lock();
    if let Some(p) = pool.as_ref() {
        let (pooled, max_pool) = p.stats();
        println!("║  Connection Pool: {}/{} connections                     ║", pooled, max_pool);
    }
    
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}
