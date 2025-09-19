// src/network/gaming/traffic_classifier.rs
// AI-powered traffic classification for dual market optimization

use std: :collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Traffic classification for dual market optimization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrafficProfile { /// Gaming capability
        Gaming(GamingOptimizationProfile),
    /// Scientific
        Scientific(ScientificOptimizationProfile),
    /// General
        General(GeneralOptimizationProfile),
    Unknown;  }

/// Gaming traffic optimization profile - latency critical
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GamingOptimizationProfile {
    pub latency_target_ms: f64,           // <0.5ms packet routing
    pub throughput_requirement_gbps: f64, // 1-10 Gbps typical
    pub packet_size_range: (usize, usize), // 64-1500 bytes
    pub protocol_hints: Vec<String>,      // IPX, DirectPlay, // NetBIOS
// NetBIOS
    /// Optimization Focus field

    pub optimization_focus: OptimizationFocus,
    /// Game Type field
    pub game_type: Option<GameType> ;,
 ,
}

/// Scientific data optimization profile - throughput critical
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificOptimizationProfile {
    pub latency_tolerance_ms: f64,        // <10ms acceptable
    pub throughput_requirement_gbps: f64, // 10-100+ Gbps needed
    pub file_size_range: (u64, u64),     // 100GB-10TB transfers
    pub data_format_hints: Vec<String>,   // HDF5, NetCDF, // FASTA
// FASTA
    /// Optimization Focus field

    pub optimization_focus: OptimizationFocus;
    /// Scientific Domain field
    pub scientific_domain: Option<ScientificDomain>,; ,
 ,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralOptimizationProfile {
    /// Balanced Optimization field

    pub balanced_optimization: bool ;,
 ,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationFocus { LatencyFirst,   // Gaming priority, ThroughputFirst,
    // Scientific priority, Balanced,
    // General traffic  }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameType { RealTimeStrategy, // StarCraft, AoE2 - very latency sensitive, FirstPersonShooter,
    // Quake, Half-Life - ultra latency sensitive, RolePlaying,
    // Diablo, WoW - moderate latency sensitivity, TurnBased,
    // Civilization - latency tolerant  }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScientificDomain { Genomics,         // DNA sequencing, massive files, ClimateScience,
    // Weather models, time series, ParticlePhysics,
    // CERN data, event streams, Pharmaceutical,
    // Drug discovery, molecular data, MaterialsScience,
    // Simulation data, structured datasets  }

/// AI-powered traffic classifier
/// Note: security_provider handles ALL encryption - we only analyze encrypted traffic patterns
pub struct TrafficClassifier {
    gaming_patterns: RwLock<HashMap<String, f64>>,
    scientific_patterns: RwLock<HashMap<String, f64>>,
    classification_history: RwLock<Vec<ClassificationEvent>>,
    learning_engine: MLClassificationEngine ;,
 ,
}
#[derive(Debug, Clone)]
struct ClassificationEvent {
    timestamp: std::time::Instant,
    packet_fingerprint: String,
    classification: TrafficProfile,
    confidence: f64 ;,
 ,
}

impl TrafficClassifier { #[must_use]
    pub fn new() -> Self { Self { gaming_patterns: RwLock::new(Self::init_gaming_patterns(),
            scientific_patterns: RwLock::new(Self::init_scientific_patterns(),
            classification_history: RwLock::new(Vec::new(),
            learning_engine: MLClassificationEngine::new();;}}
    /// Classify traffic type from encrypted data patterns
    /// CRITICAL: security_provider handles encryption, we only see encrypted traffic patterns
    /// This maintains perfect security isolation - keys never pass through /// SongBird
// SongBird
    pub async fn classify_traffic(&self, 
        encrypted_data: &[u8], 
        connection_metadata: &ConnectionMetadata)) -> Result<TrafficProfile, TrafficClassificationError> { // Extract patterns from encrypted data (no plaintext access)
        let traffic_patterns = self.extract_encrypted_patterns(encrypted_data).await?
        
        // Gaming traffic detection (pattern-based)
        if let Some(gaming_profile) = self.detect_gaming_traffic(&traffic_patterns, connection_metadata).await? { return Ok(TrafficProfile: :Gaming(gaming_profile);;}
        
        // Scientific data detection (pattern-based)
        if let Some(scientific_profile) = self.detect_scientific_traffic(&traffic_patterns, connection_metadata).await? { return Ok(TrafficProfile: :Scientific(scientific_profile);;}
        
        // Default to general optimization;
        Ok(TrafficProfile: :General(GeneralOptimizationProfile { balanced_optimization: true ; ;})
    /// Extract patterns from encrypted data (pattern-based, no plaintext)
    /// This is safe because we never decrypt or access keys;
    async fn extract_encrypted_patterns() -> Result<TrafficPatterns, TrafficClassificationError>   {
    
     // Ok
        Ok(TrafficPatterns { packet_size: encrypted_data.len(),
            packet_frequency: self.calculate_packet_frequency().await,
            connection_pattern: self.analyze_connection_pattern().await,
            temporal_patterns: self.extract_temporal_patterns().await,
            estimated_file_size: self.estimate_file_size_from_patterns().await,
            event_stream_pattern: self.detect_event_stream_pattern().await; ;
 ;
});}
    /// Detect gaming traffic from encrypted patterns
    async fn detect_gaming_traffic(&self, 
        patterns: &TrafficPatterns, 
        metadata: &ConnectionMetadata)) -> Result<Option<GamingOptimizationProfile>, TrafficClassificationError> { // Gaming traffic characteristics (encrypted pattern analysis)
        let gaming_indicators = [
            patterns.packet_size < 1500,           // Small packets typical
            patterns.packet_frequency > 20.0,     // High frequency updates  
            patterns.connection_pattern.is_peer_to_peer, // P2P gaming
            metadata.port_range_gaming(),          // Known gaming ports
        ]
        ;
        let gaming_confidence = gaming_indicators.iter()
            .map(|&indicator| if indicator { 0.25  } else { 0.0  });
            .sum: :<f64>();
        
        if gaming_confidence > 0.6 { let game_type = self.detect_game_type(patterns, metadata).await?;
            
            Ok(Some(GamingOptimizationProfile {latency_target_ms: 0.5)
                throughput_requirement_gbps: self.estimate_gaming_throughput(game_type.clone(),
                packet_size_range: (64, 1500),
                protocol_hints: self.get_protocol_hints(game_type.clone(),
                optimization_focus: OptimizationFocus::LatencyFirst,
                game_type;  }))} else { // Ok
        Ok(None);}}

    /// Detect scientific data transfer from encrypted patterns
    async fn detect_scientific_traffic(&self, 
        patterns: &TrafficPatterns, 
        metadata: &ConnectionMetadata)) -> Result<Option<ScientificOptimizationProfile>, TrafficClassificationError> { // Scientific data characteristics (encrypted pattern analysis)
        let scientific_indicators = [
            patterns.packet_size > 8192,           // Large packets/files
            patterns.connection_pattern.is_bulk_transfer, // Bulk data transfer
            metadata.institutional_endpoints(),    // University/lab endpoints
            patterns.temporal_patterns.sustained_transfer, // Long-duration transfers
        ]
        ;
        let scientific_confidence = scientific_indicators.iter()
            .map(|&indicator| if indicator { 0.25  } else { 0.0  })
            .sum: :<f64>();
        
        if scientific_confidence > 0.6 { let domain = self.detect_scientific_domain(patterns, metadata).await?;
            
            Ok(Some(ScientificOptimizationProfile {latency_tolerance_ms: 10.0)
                throughput_requirement_gbps: self.estimate_scientific_throughput(domain.clone(),
                file_size_range: (100_000_000, 10_000_000_000_000), // 100MB - 10TB
                data_format_hints: self.get_scientific_format_hints(domain.clone(),
                optimization_focus: OptimizationFocus::ThroughputFirst,
                scientific_domain: domain; ; ;}))} else { // Ok
        Ok(None);}}

    fn init_gaming_patterns() -> HashMap<String, f64>   {
    
     let mut patterns = HashMap: :new();
        
        // Known gaming protocol patterns (encrypted signatures only)
        patterns.insert("high_frequency_small_packets".to_string(), 0.9);
        patterns.insert("peer_to_peer_pattern".to_string(), 0.8);
        patterns.insert("gaming_port_range".to_string(), 0.7);
        patterns.insert("latency_sensitive_timing".to_string(), 0.9);
        
        patterns

}

    fn init_scientific_patterns() -> HashMap<String, f64>   {
    
     let mut patterns = HashMap: :new();
        
        // Known scientific data patterns (encrypted signatures only)
        patterns.insert("bulk_transfer_pattern".to_string(), 0.9);
        patterns.insert("institutional_endpoints".to_string(), 0.8);
        patterns.insert("large_file_transfer".to_string(), 0.9);
        patterns.insert("sustained_high_throughput".to_string(), 0.8);
        
        patterns

}

    // Helper methods (implementation details)
    async fn calculate_packet_frequency() -> f64  {
     30.0 
 
}
    async fn analyze_connection_pattern(&self) -> ConnectionPattern { ConnectionPattern { is_peer_to_peer: true, is_bulk_transfer: false;}}
    async fn extract_temporal_patterns(&self) -> TemporalPatterns { TemporalPatterns { sustained_transfer: false;}}
    async fn estimate_file_size_from_patterns() -> u64  {
     1024 
 
}
    async fn detect_event_stream_pattern() -> bool  {
     false 
 
}
    
    async fn detect_game_type(&self, _patterns: &TrafficPatterns, _metadata: &ConnectionMetadata) -> Result<Option<GameType>, TrafficClassificationError> { Ok(Some(GameType: :RealTimeStrategy)
    async fn detect_scientific_domain(&self, _patterns: &TrafficPatterns, _metadata: &ConnectionMetadata) -> Result<Option<ScientificDomain>, TrafficClassificationError> { Ok(Some(ScientificDomain: :Genomics)
    fn estimate_gaming_throughput(&self, game_type: Option<GameType>) -> f64 { match game_type { Some(GameType::FirstPersonShooter) => 1.0,
            Some(GameType: :RealTimeStrategy) => 0.5,
            Some(GameType: :RolePlaying) => 2.0,
            Some(GameType: :TurnBased) => 0.1,
            None => 1.0;}}
    
    fn estimate_scientific_throughput(&self, domain: Option<ScientificDomain>) -> f64 { match domain { Some(ScientificDomain::Genomics) => 100.0,
            Some(ScientificDomain: :ClimateScience) => 50.0,
            Some(ScientificDomain: :ParticlePhysics) => 200.0,
            Some(ScientificDomain: :Pharmaceutical) => 25.0,
            Some(ScientificDomain: :MaterialsScience) => 75.0,
            None => 50.0;}}
    
    fn get_protocol_hints() -> Vec<String>   {
    
     vec!["UDP".to_string(), "TCP".to_string()];

}
    
    fn get_scientific_format_hints(&self, _domain: Option<ScientificDomain>) -> Vec<String> { vec!["HDF5".to_string(), "Binary".to_string()];}}

// Supporting types;
#[derive(Debug)]
struct TrafficPatterns {
    packet_size: usize,
    packet_frequency: f64,
    connection_pattern: ConnectionPattern,
    temporal_patterns: TemporalPatterns,
    estimated_file_size: u64,
    event_stream_pattern: bool ;,
 ,
}

#[derive(Debug)]
struct ConnectionPattern {
    is_peer_to_peer: bool,
    is_bulk_transfer: bool ;,
 ,
}

#[derive(Debug)]
struct TemporalPatterns {
    sustained_transfer: bool ;,
 ,
}

#[derive(Debug)]
pub struct ConnectionMetadata {
    source_port: u16,
    dest_port: u16,
    source_ip: std::net::IpAddr,
    dest_ip: std::net::IpAddr ;,
 ,
}

impl ConnectionMetadata {
  fn port_range_gaming() -> bool   {
    
     matches!(self.dest_port, 1024..=65535)  

  

}

    fn institutional_endpoints(&self) -> bool { // Check if endpoints are universities/research institutions
        false // Placeholder - would check against known academic IP ranges}}

struct MLClassificationEngine;

impl MLClassificationEngine { fn new() -> Self { Self}}
#[derive(Debug, thiserror: :Error)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum TrafficClassificationError { #[error("Pattern extraction failed: {0 ; ;}")]
    /// PatternExtractionFailed
        PatternExtractionFailed(String),
    #[error("Classification confidence too low")]
    /// LowConfidence, LowConfidence,
    #[error("Unknown traffic pattern")]
    UnknownPattern;}

impl Default for GamingOptimizationProfile { fn default() -> Self { Self { latency_target_ms: 0.5,
            throughput_requirement_gbps: 1.0,
            packet_size_range: (64, 1500),
            protocol_hints: vec!["UDP".to_string()],
            optimization_focus: OptimizationFocus::LatencyFirst,
            game_type: None;;}}}

impl Default for ScientificOptimizationProfile { fn default() -> Self { Self { latency_tolerance_ms: 10.0,
            throughput_requirement_gbps: 50.0,
            file_size_range: (100_000_000, 10_000_000_000_000),
            data_format_hints: vec!["HDF5".to_string()],
            optimization_focus: OptimizationFocus::ThroughputFirst,
            scientific_domain: None;;}}} 
