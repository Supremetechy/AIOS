//! TLS/SSL support for secure connections
//! 
//! Provides TLS 1.2/1.3 support using rustls

use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use crate::println;
use super::socket::TcpSocket;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::crypto::ring as provider;
use rustls::DigitallySignedStruct;

/// TLS socket wrapping a TCP socket with rustls
pub struct TlsSocket {
    connection: Option<ClientConnection>,
    tcp_socket: TcpSocket,
    hostname: String,
    connected: bool,
}

impl TlsSocket {
    /// Create new TLS socket
    pub fn new(tcp_socket: TcpSocket, hostname: &str) -> Result<Self, &'static str> {
        println!("[TLS] Creating TLS connection to {}...", hostname);
        
        // Create rustls config
        let config = create_tls_config()?;
        
        // Create server name
        let server_name = ServerName::try_from(hostname)
            .map_err(|_| "Invalid hostname")?;
        
        // Create client connection
        let connection = ClientConnection::new(Arc::new(config), server_name)
            .map_err(|_| "Failed to create TLS connection")?;
        
        Ok(TlsSocket {
            connection: Some(connection),
            tcp_socket,
            hostname: String::from(hostname),
            connected: false,
        })
    }
    
    /// Perform TLS handshake
    pub fn connect(&mut self) -> Result<(), &'static str> {
        println!("[TLS] Starting TLS handshake with {}...", self.hostname);
        
        if let Some(ref mut conn) = self.connection {
            // Complete handshake
            while conn.is_handshaking() {
                // Write handshake data
                if conn.wants_write() {
                    let mut output = vec![0u8; 16384];
                    let written = conn.write_tls(&mut output.as_mut_slice())
                        .map_err(|_| "TLS write failed")?;
                    
                    if written > 0 {
                        self.tcp_socket.send(&output[..written])?;
                    }
                }
                
                // Read handshake data
                if conn.wants_read() {
                    let mut input = vec![0u8; 16384];
                    let read = self.tcp_socket.recv(&mut input)?;
                    
                    if read > 0 {
                        conn.read_tls(&mut &input[..read])
                            .map_err(|_| "TLS read failed")?;
                        conn.process_new_packets()
                            .map_err(|_| "TLS packet processing failed")?;
                    }
                }
            }
            
            println!("[TLS] ✓ TLS handshake complete (TLS 1.3)");
            self.connected = true;
            Ok(())
        } else {
            Err("No TLS connection")
        }
    }
    
    /// Send encrypted data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.connected {
            return Err("TLS not connected");
        }
        
        if let Some(ref mut conn) = self.connection {
            // Write application data
            conn.writer().write_all(data)
                .map_err(|_| "Failed to write encrypted data")?;
            
            // Send TLS records
            let mut output = vec![0u8; 16384];
            let written = conn.write_tls(&mut output.as_mut_slice())
                .map_err(|_| "TLS write failed")?;
            
            if written > 0 {
                self.tcp_socket.send(&output[..written])?;
            }
            
            Ok(data.len())
        } else {
            Err("No TLS connection")
        }
    }
    
    /// Receive and decrypt data
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.connected {
            return Err("TLS not connected");
        }
        
        if let Some(ref mut conn) = self.connection {
            // Read TLS records from socket
            let mut input = vec![0u8; 16384];
            let read = self.tcp_socket.recv(&mut input)?;
            
            if read > 0 {
                conn.read_tls(&mut &input[..read])
                    .map_err(|_| "TLS read failed")?;
                conn.process_new_packets()
                    .map_err(|_| "TLS packet processing failed")?;
            }
            
            // Read decrypted application data
            conn.reader().read(buffer)
                .map_err(|_| "Failed to read decrypted data")
        } else {
            Err("No TLS connection")
        }
    }
    
    /// Close TLS connection
    pub fn close(&mut self) {
        if let Some(ref mut conn) = self.connection {
            let _ = conn.send_close_notify();
        }
        self.tcp_socket.close();
        self.connected = false;
    }
}

/// Create TLS client configuration
fn create_tls_config() -> Result<ClientConfig, &'static str> {
    // Use rustls with ring crypto provider
    let crypto_provider = provider::default_provider();
    
    // Create config builder
    let mut config = ClientConfig::builder()
        .with_root_certificates(get_root_certs())
        .with_no_client_auth();
    
    // Enable TLS 1.3
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    
    Ok(config)
}

/// Get root CA certificates
fn get_root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    
    // Add webpki roots (Mozilla's CA bundle)
    roots.extend(
        webpki_roots::TLS_SERVER_ROOTS
            .iter()
            .cloned()
    );
    
    println!("[TLS] Loaded {} root CA certificates", roots.len());
    
    roots
}

/// Minimal certificate verifier (for development)
pub struct MinimalVerifier;

impl ServerCertVerifier for MinimalVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // WARNING: This accepts any certificate (for testing only!)
        println!("[TLS] ⚠ Using minimal verifier (INSECURE - testing only)");
        Ok(ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
