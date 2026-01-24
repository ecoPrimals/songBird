// Songbird TLS Server Mode - Foundation
// Reuses ALL client logic, just reverses message flow

use crate::beardog_client::BearDogClient;
use crate::error::{Error, Result};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, debug};

/// TLS Server Mode
/// 
/// CRITICAL: Reuses ALL client logic from TlsHandshake!
/// - Same update_transcript()
/// - Same compute_transcript_hash()
/// - Same key derivation
/// - Just reverses message flow (receive ClientHello, send ServerHello)
pub struct TlsServer {
    /// Shared BearDog client for crypto operations
    #[allow(dead_code)]
    beardog: Arc<BearDogClient>,
    
    /// Transcript tracking (SAME as client!)
    transcript: Vec<u8>,
    
    /// Server certificate and private key
    /// TODO: Load from config or BearDog
    #[allow(dead_code)]
    cert_chain: Vec<u8>,
    #[allow(dead_code)]
    private_key: Vec<u8>,
}

impl TlsServer {
    /// Create new TLS server
    pub fn new(beardog: Arc<BearDogClient>) -> Self {
        Self {
            beardog,
            transcript: Vec::new(),
            cert_chain: Vec::new(),
            private_key: Vec::new(),
        }
    }
    
    /// Accept a TLS 1.3 connection
    /// 
    /// This is the server equivalent of TlsHandshake::handshake()
    /// CRITICAL: Uses EXACT same transcript logic as client!
    pub async fn accept_connection(&mut self, stream: &mut TcpStream) -> Result<()> {
        info!("🔒 TLS Server: Accepting connection");
        
        // 1. Read ClientHello
        info!("Step 1: Reading ClientHello from client");
        let client_hello = self.read_client_hello(stream).await?;
        
        // CRITICAL: Add to transcript (SAME as client!)
        self.update_transcript(&client_hello, "ClientHello (received)", false);
        info!("✅ ClientHello received and added to transcript: {} bytes", client_hello.len());
        
        // TODO: Parse ClientHello
        // - Extract client random
        // - Extract supported cipher suites
        // - Extract key share extension
        // - Extract supported groups
        // - Extract signature algorithms
        
        // 2. Send ServerHello
        info!("Step 2: Building and sending ServerHello");
        let server_hello = self.build_server_hello()?;
        
        // CRITICAL: Add to transcript BEFORE sending (SAME as client!)
        self.update_transcript(&server_hello, "ServerHello (sending)", false);
        stream.write_all(&server_hello).await.map_err(Error::Io)?;
        info!("✅ ServerHello sent and added to transcript: {} bytes", server_hello.len());
        
        // TODO: Continue with encrypted handshake messages
        // 3. Derive handshake traffic keys (SAME as client!)
        // 4. Send EncryptedExtensions (encrypted)
        // 5. Send Certificate (encrypted)
        // 6. Send CertificateVerify (encrypted)
        // 7. Send Server Finished (encrypted)
        // 8. Compute transcript hash (SAME as client!)
        // 9. Derive application traffic keys (SAME as client!)
        // 10. Receive client Finished
        // 11. Receive and decrypt HTTP request
        // 12. Send HTTP response
        
        info!("🎉 TLS Server: Connection accepted (handshake incomplete - TODO)");
        Ok(())
    }
    
    /// Read ClientHello from client
    async fn read_client_hello(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        // Read TLS record (5-byte header + payload)
        let mut header = [0u8; 5];
        stream.read_exact(&mut header).await.map_err(Error::Io)?;
        
        let record_type = header[0];
        let tls_version = u16::from_be_bytes([header[1], header[2]]);
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;
        
        debug!("TLS record: type=0x{:02x}, version=0x{:04x}, length={}", 
               record_type, tls_version, length);
        
        // Read payload
        let mut payload = vec![0u8; length];
        stream.read_exact(&mut payload).await.map_err(Error::Io)?;
        
        // For ClientHello, return the handshake message (strip TLS record header)
        // CRITICAL: Return ONLY the handshake message, not the TLS record header!
        Ok(payload)
    }
    
    /// Build ServerHello
    /// 
    /// TODO: Complete implementation
    /// - Generate server random
    /// - Select cipher suite
    /// - Generate x25519 key pair
    /// - Build key share extension
    fn build_server_hello(&self) -> Result<Vec<u8>> {
        // Placeholder - TODO: Complete implementation
        info!("TODO: Building ServerHello");
        
        // For now, return empty Vec (will be implemented in Phase 2)
        Ok(Vec::new())
    }
    
    /// Update transcript
    /// 
    /// CRITICAL: This is EXACTLY the same as TlsHandshake::update_transcript()!
    /// We must use the SAME logic to ensure transcript hashes match!
    fn update_transcript(&mut self, message: &[u8], label: &str, was_decrypted: bool) {
        let before = self.transcript.len();
        self.transcript.extend_from_slice(message);
        let after = self.transcript.len();
        
        info!("📝 SERVER Transcript Update: {}", label);
        info!("   Message length: {} bytes", message.len());
        info!("   Was decrypted: {}", was_decrypted);
        info!("   Cumulative: {} → {} bytes (+{} bytes)", before, after, message.len());
        
        if !message.is_empty() {
            debug!("   First byte: 0x{:02x}", message[0]);
            debug!("   First 16 bytes: {}", hex::encode(&message[..std::cmp::min(16, message.len())]));
        }
    }
    
    /// Compute transcript hash
    /// 
    /// CRITICAL: This is EXACTLY the same as TlsHandshake::compute_transcript_hash()!
    /// We must use the SAME logic to ensure transcript hashes match!
    #[allow(dead_code)]
    fn compute_transcript_hash(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.transcript);
        let hash = hasher.finalize().to_vec();
        
        info!("🔐 SERVER Transcript Hash: {}", hex::encode(&hash));
        info!("   Transcript length: {} bytes", self.transcript.len());
        
        hash
    }
    
    /// Log complete transcript hex dump
    /// 
    /// CRITICAL: Same format as client for easy comparison!
    #[allow(dead_code)]
    fn log_transcript_hex_dump(&self) {
        info!("════════════════════════════════════════════════════════════");
        info!("🔬 SERVER COMPLETE TRANSCRIPT HEX DUMP");
        info!("════════════════════════════════════════════════════════════");
        info!("Total transcript length: {} bytes", self.transcript.len());
        info!("");
        info!("📝 Full transcript (hex):");
        for (i, chunk) in self.transcript.chunks(64).enumerate() {
            info!("{:04x}: {}", i * 64, hex::encode(chunk));
        }
        info!("════════════════════════════════════════════════════════════");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transcript_tracking() {
        let beardog = Arc::new(BearDogClient::new("http://localhost:3000".to_string()));
        let mut server = TlsServer::new(beardog);
        
        // Test that transcript tracking works
        let test_message = b"test message";
        server.update_transcript(test_message, "Test", false);
        
        assert_eq!(server.transcript.len(), test_message.len());
        assert_eq!(&server.transcript[..], test_message);
    }
    
    #[test]
    fn test_transcript_hash() {
        let beardog = Arc::new(BearDogClient::new("http://localhost:3000".to_string()));
        let mut server = TlsServer::new(beardog);
        
        // Add test data
        server.update_transcript(b"ClientHello", "ClientHello", false);
        server.update_transcript(b"ServerHello", "ServerHello", false);
        
        // Compute hash
        let hash = server.compute_transcript_hash();
        
        // Hash should be 32 bytes (SHA-256)
        assert_eq!(hash.len(), 32);
    }
}

