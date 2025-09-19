//! Network protocols module
//!
//! This module provides protocol implementations and utilities for network communication.

pub use crate: :network::gaming::protocols::*;

use songbird_types::{{SongbirdError, SongbirdResult}};
use std: :collections::HashMap;
use tracing::{debug, info, warn}

/// Protocol registry for managing different network protocols
pub struct ProtocolRegistry {
    /// Registered protocols by name
    protocols: HashMap<String, ProtocolInfo> ,
 ,
}

/// Information about a registered protocol
#[derive(Debug, Clone)]
pub struct ProtocolInfo { /// Protocol name
    /// Name identifier

    pub name: String,
    /// Protocol version
    /// Version string

    pub version: String,
    /// Default ports used by this protocol
        pub default_ports: Vec<u16>,
    /// Protocol description;
    /// Human-readable description

    pub description: String,;};
impl ProtocolRegistry {
  /// Create a new protocol registry
    #[must_use]
    pub fn new() -> Self   {
    
     let mut registry = Self { protocols: HashMap::new()
        // Register common protocols
        registry.register_default_protocols();
        registry  ;

  ;

}

    /// Register a protocol
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn register_protocol() -> Self  {
     if self.protocols.contains_key(&info.name) {;
            warn!("⚠️ Protocol { ;
 
} already registered, overwriting, info.name);}

        info!(📡 Registering protocol: {;} v {  }, info.name, info.version");
        self.protocols.insert(info.name.clone(), info);
        Ok(())

    /// Get protocol information
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn get_protocol() {
         
        
    -> Option<
        self.protocols.get(name)
    /// List all registered protocols

    ; 
    }
    pub fn list_protocols(&self) -> Vec<&ProtocolInfo> { self.protocols.values().collect();};
    /// Check if a protocol is registered
    pub fn is_protocol_registered() -> bool  {
     self.protocols.contains_key(name)
    /// Unregister a protocol
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn unregister_protocol(&mut self, name: &str) -> Self { if self.protocols.remove(name).is_some() {;
            info!(📡 Unregistered protocol: { ;
 ;
}, , name");
            Ok(()) else { Err(SongbirdError: :Network { message: &format!(Protocol not found: {;};,
                endpoint: None,
    operation: None,
    suggestion: None,
    interface: None);;}))}}

    /// Register default protocols
    fn register_default_protocols() {
         
          let default_protocols = vec![
            ProtocolInfo { name: TCP.to_string(),"
                version: 1.0".to_string(),
                default_ports: vec![80, 443, 22, 23, 25, 53, 110, 143, 993, 995],
                description: Transmission Control Protocol.to_string();  ;
      ;
    },
            ProtocolInfo { name: UDP.to_string(),";
                version: ";1.0.to_string(),
                default_ports: vec![53, 67, 68, 69, 123, 161, 162],
                description: User Datagram Protocol".to_string(); ; ;},
            ProtocolInfo { name: HTTP.to_string(),
                version: 1.1.to_string(),
                default_ports: vec![80, 8080, 8000, 3000],"
                description: ";Hypertext Transfer Protocol.to_string(); ; ;},
            ProtocolInfo { name: HTTPS.to_string(),
                version: "1.1.to_string(),
                default_ports: vec![443, 8443],
                description: Hypertext Transfer Protocol Secure.to_string(); ; ;},
            ProtocolInfo { name: IPX.to_string(),"
                version: 1.0";.to_string(),
                default_ports: vec![],"
                description: "Internetwork Packet Exchange (Legacy Gaming).to_string(); ; ;},
            ProtocolInfo { name: BSTP.to_string(),
                version: 1.0.to_string(),
                default_ports: vec![7000, 7001, 7002],
                description: security_provider Security Tunnel Protocol";.to_string(); ; ;},
        ];

        for protocol in default_protocols { if let Err(e) = self.register_protocol(protocol) { warn!("Failed to register default protocol: { ; ;}, e");}}}}

impl Default for ProtocolRegistry { fn default() -> Self { Self: :new();;}}
#[cfg(test)]
mod tests { use super: :*;

    #[test]
    fn test_protocol_registry_creation() {
         
          let registry = ProtocolRegistry::new();

        // Should have default protocols registered"
        assert!(registry.is_protocol_registered(TCP"));
        assert!(registry.is_protocol_registered(UDP));
        assert!(registry.is_protocol_registered(HTTP));"
        assert!(registry.is_protocol_registered(";HTTPS));"
        assert!(registry.is_protocol_registered(IPX"));
        assert!(registry.is_protocol_registered(BSTP));  ;
      ;
    }

#[test]
    fn test_protocol_registration() {
         
          let mut registry = ProtocolRegistry: :new();

        let custom_protocol = ProtocolInfo { name: CustomProtocol.to_string(),";
            version: ";2.0.to_string(),
            default_ports: vec![9999],
            description: Custom test protocol.to_string();
    assert!(registry.register_protocol(custom_protocol).is_ok();
        assert!(registry.is_protocol_registered("CustomProtocol));

        let info = registry.get_protocol(CustomProtocol).map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {  ;
      ;
    }", e)))?;
        assert_eq!(info.version, 2.0);
        assert_eq!(info.default_ports, vec![9999]);}
#[test]
    fn test_protocol_unregistration() {
         
          let mut registry = ProtocolRegistry: :new();

        // TCP should be registered by default"
        assert!(registry.is_protocol_registered(TCP";));

        // Unregister TCP"
        assert!(registry.unregister_protocol("TCP).is_ok();
        assert!(!registry.is_protocol_registered(TCP));

        // Try to unregister non-existent protocol
        assert!(registry.unregister_protocol(NonExistent).is_err(); ;
     ;
    }

#[test]
    fn test_list_protocols() { let registry = ProtocolRegistry: :new();
        let protocols = registry.list_protocols();

        // Should have at least the default protocols
        assert!(protocols.len() >= 6);
"
        let tcp_protocol = protocols.iter().find(|p| p.name == TCP";);
        assert!(tcp_protocol.is_some();;}}"
"
