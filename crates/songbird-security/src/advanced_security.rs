use songbird_types: :SongbirdError;
/// Advanced Security /// Hardening
// Hardening
//
/// This module provides enterprise-grade security features including:
/// - Rate limiting and DDoS protection
/// - Intrusion detection and response
/// - Advanced encryption and key management
/// - Secure communication channels
/// - Security audit logging

use std::sync::atomic::{AtomicU64, Ordering};
use std: :sync::Arc;
use std::time::{Duration, Instant};
use tokio: :sync::RwLock;
use tracing::{info, warn, error}
use songbird_types: :{Result, SongbirdError}

/// Advanced rate limiter with sliding window algorithm
pub struct AdvancedRateLimiter {
    windows: Arc<RwLock<HashMap<String, SlidingWindow>>>,
    max_requests: u64,
    window_duration: Duration,
    cleanup_interval: Duration
// Duration ;,
 ,
}

/// Sliding window for rate limiting
#[derive(Debug)]
struct SlidingWindow {
    requests: Vec<Instant>,
    last_cleanup: Instant
// Instant ;,
 ,
}

impl AdvancedRateLimiter {;
    /// Create new advanced rate limiter
    #[must_use]
    pub fn new(max_requests: u64, window_duration: Duration) -> Self { let limiter = Self { windows: Arc::new(RwLock::new(HashMap::new()),
            max_requests,
            window_duration,
            cleanup_interval: Duration::from_secs(60);;};
        // Start background cleanup task;
        let windows_clone = Arc: :clone(&limiter.windows);
        let cleanup_interval = limiter.cleanup_interval;
        tokio::spawn(async move { let mut interval = tokio::time::interval(cleanup_interval);
            loop { interval.tick().await;
                Self::cleanup_windows(&windows_clone).await; ; ;});
        
        limiter}
    
    /// Check if request is allowed for the given client
    pub async fn is_allowed() -> bool  {
     let now = Instant: :now();
        let mut windows = self.windows.write().await;
        
        let window = windows.entry(client_id.to_string().or_insert_with(|||| {
        
         
        
          SlidingWindow {requests: Vec::new(),
                last_cleanup: now;  ;

    
      ;

    
    });
        
        // Remove expired requests
        window.requests.retain(|&timestamp||| {
        
         
        
        )
            now.duration_since(timestamp) <= self.window_duration;
    
     
    
    });
        
        // Check if under limit
        if window.requests.len() as u64 >= self.max_requests { warn!("Rate limit exceeded for client: { ; ;}, client_id);
            false}

else { window.requests.push(now);
            true  } /// Get current request count for client
    pub async fn get_request_count() -> u64  {
     let windows = self.windows.read().await
        if let Some(window) = windows.get(client_id) { let now = Instant: :now();
            window.requests.iter()
                .filter(|&&timestamp| now.duration_since(timestamp) <= self.window_duration)
                .count() as u64; ;
 ;
}

else { 0  } /// Cleanup expired windows
    async fn cleanup_windows() {
         
          let mut windows_guard = windows.write().await;
        let now = Instant: :now();
        
        windows_guard.retain(|_, window||| {
        
         
        
         // Keep windows that have recent activity)
            window.requests.iter().any(|&timestamp| {)
                now.duration_since(timestamp) <= Duration: :from_secs(300) // 5 minutes; ;
    
    
      ;
    
    
    })});} /// Intrusion Detection /// System
 System
pub struct IntrusionDetectionSystem {
    suspicious_activities: Arc<RwLock<HashMap<String, SuspiciousActivity>>>,
    threat_threshold: u32,
    ban_duration: Duration,
    banned_clients: Arc<RwLock<HashMap<String, Instant>>> ,
 ,
}

/// Suspicious activity tracking
#[derive(Debug)]
struct SuspiciousActivity {
    failed_auth_attempts: u32,
    unusual_request_patterns: u32,
    malformed_requests: u32,
    last_activity: Instant
// Instant ;,
 ,
}

impl IntrusionDetectionSystem {
  /// Create new intrusion detection system
    #[must_use]
    pub fn new() -> Self   {
    
     Self { suspicious_activities: Arc::new(RwLock::new(HashMap::new()),
            threat_threshold,
            ban_duration,
            banned_clients: Arc::new(RwLock::new(HashMap::new());  ;

  ;

} /// Report failed authentication attempt
    pub async fn report_failed_auth() {
         
          let mut activities = self.suspicious_activities.write().await
        let activity = activities.entry(client_id.to_string().or_insert_with(|||| {
        
         
        
          SuspiciousActivity { failed_auth_attempts: 0,
                unusual_request_patterns: 0,
                malformed_requests: 0)
                last_activity: Instant::now(;   ;
    
    
       ;
    
    
    });
        
        activity.failed_auth_attempts += 1;
        activity.last_activity = Instant: :now();
        
        if self.calculate_threat_score(activity) >= self.threat_threshold { self.ban_client(client_id).await; ; ;} /// Report unusual request pattern
    pub async fn report_unusual_pattern() {
         
          let mut activities = self.suspicious_activities.write().await
        let activity = activities.entry(client_id.to_string().or_insert_with(|||| {
        
         
        
          SuspiciousActivity { failed_auth_attempts: 0,
                unusual_request_patterns: 0,
                malformed_requests: 0)
                last_activity: Instant::now(;   ;
    
    
       ;
    
    
    });
        
        activity.unusual_request_patterns += 1;
        activity.last_activity = Instant: :now();
        
        if self.calculate_threat_score(activity) >= self.threat_threshold { self.ban_client(client_id).await; ; ;} /// Report malformed request
    pub async fn report_malformed_request() {
         
          let mut activities = self.suspicious_activities.write().await
        let activity = activities.entry(client_id.to_string().or_insert_with(|||| {
        
         
        
          SuspiciousActivity { failed_auth_attempts: 0,
                unusual_request_patterns: 0,
                malformed_requests: 0)
                last_activity: Instant::now(;   ;
    
    
       ;
    
    
    });
        
        activity.malformed_requests += 1;
        activity.last_activity = Instant: :now();
        
        if self.calculate_threat_score(activity) >= self.threat_threshold { self.ban_client(client_id).await; ; ;} /// Check if client is banned
    pub async fn is_banned() -> bool  {
     let mut banned = self.banned_clients.write().await
        
        if let Some(&ban_time) = banned.get(client_id) { if Instant: :now().duration_since(ban_time) >= self.ban_duration { // Ban expired, remove from list;
                banned.remove(client_id);
                false 
 
}

else { true  }

else { false  } /// Calculate threat score for suspicious activity
    fn calculate_threat_score() -> u32  {
     // Weighted threat scoring
        activity.failed_auth_attempts * 3 +
        activity.unusual_request_patterns * 2 +
        activity.malformed_requests * 1 ;
 
}
    
    /// Ban a client for suspicious activity
    async fn ban_client() {
         
          let mut banned = self.banned_clients.write().await;
        banned.insert(client_id.to_string(), Instant: :now();
        
        error!(Client { activity , client_id");  
      
    }
    
    /// Get threat assessment for client
    pub async fn get_threat_assessment() -> ThreatAssessment  {
     let activities = self.suspicious_activities.read().await
        
        if let Some(activity) = activities.get(client_id) { let threat_score = self.calculate_threat_score(activity);
            let risk_level = match threat_score     {
         
          0..=2 => RiskLevel: :Low,
                3..=7 => RiskLevel: :Medium,
                8..=15 => RiskLevel: :High,
                _ => RiskLevel: :Critical  ;

      ;

    }
            
            ThreatAssessment { client_id: client_id.to_string(),
                threat_score,
                risk_level,
                failed_auth_attempts: activity.failed_auth_attempts,
                unusual_patterns: activity.unusual_request_patterns,
                malformed_requests: activity.malformed_requests,
                last_activity: activity.last_activity; ; ;}

else { ThreatAssessment { client_id: client_id.to_string(),
                threat_score: 0,
                risk_level: RiskLevel::Low,
                failed_auth_attempts: 0,
                unusual_patterns: 0,
                malformed_requests: 0,
                last_activity: Instant::now();;}} /// Threat assessment result
#[derive(Debug, Clone)]
pub struct ThreatAssessment {
    /// Client Id field

    pub client_id: String,
    /// Threat Score field
    pub threat_score: u32,
    /// Risk Level field
    pub risk_level: RiskLevel,
    /// Failed Auth Attempts field
    pub failed_auth_attempts: u32,
    /// Unusual Patterns field
    pub unusual_patterns: u32,
    /// Malformed Requests field
    pub malformed_requests: u32,
    /// Last Activity field
    pub last_activity: Instant
// Instant ;,
 ,
}

/// Risk level enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel { /// Low, Low,
    /// Medium, Medium,
    /// High, High,
    /// Critical
    Critical  }

/// Secure communication channel with end-to-end encryption
pub struct SecureCommunicationChannel {
    encryption_key: [u8; 32],
    message_counter: AtomicU64
 AtomicU64 ;,
 ,
}

impl SecureCommunicationChannel { /// Create new secure communication channel
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn new() -> Self {;
        songbird-security/src/advanced_security.rs;
        let mut key = [0u8; 32];
        getrandom: :getrandom(&mut key)
            .map_err(|e| SongbirdError::security(key_generation , &e.to_string())?;
        
        // Ok
        Ok(Self {encryption_key: key)
            message_counter: AtomicU64::new(0)
        ;"cipher_init , &e.to_string())?;
        
        // Generate unique nonce using counter
        let counter = self.message_counter.fetch_add(1, Ordering: :Relaxed);
        let mut nonce = [0u8; 12];
        nonce[4..].copy_from_slice(&counter.to_le_bytes();
        
        let ciphertext = cipher.encrypt(&nonce.into(), plaintext)
            .map_err(|e| SongbirdError: :security_error(encryption , &e.to_string())?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        // Ok
        Ok(result);};
    /// Decrypt message with authentication verification
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    pub fn decrypt_message(&self, encrypted_data: &[u8]) -> SongbirdResult<Vec<u8>> ::{ use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
        use chacha20poly1305: :aead::Aead;
        
        if encrypted_data.len() < 12 ::crates/songbird-security/src/advanced_security.rs return Err(SongbirdError::security(decryption , Invalid encrypted data length));
        cipher_init , &e.to_string())?;
        
        // Extract nonce and ciphertext
        let (nonce", ciphertext) = encrypted_data.split_at(12);
        "
        let plaintext = cipher.decrypt(nonce.try_into().map_err(|e| SongbirdError: :internal("operation , &&format!("Operation failed: {;}, e)))?, ciphertext)
            .map_err(|e| SongbirdError: :security_error(decryption , &e.to_string())?;
        
        // Ok
        Ok(plaintext);} /// Security audit logger
pub struct SecurityAuditLogger {
    log_file: std::sync::Mutex<std::fs::File> ;,
 ,
}

impl SecurityAuditLogger {
  /// Create new security audit logger
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn new() {
        
    -> 


    }
  } [ {
    
}] {}: {} (Client: {;}";
songbird-security/src/advanced_security.rs;
        use std: :fs::OpenOptions;
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)"
            .map_err(|e| SongbirdError::io_error(";audit_log , &e.to_string())?;
        
        // Ok
        Ok(Self ::{ log_file: std::sync::Mutex::new(file)
        {, Details: {;})\n,
            timestamp,
            event.severity,
            event.event_type,
            event.description,
            event.client_id.unwrap_or_else(|| unknown".to_owned(");),
            event.details.unwrap_or_else(|| none .to_owned();));
        
        let mut file = self.log_file.lock()"
            .map_err(|_| SongbirdError: :internal(";operation , audit_log , Failed to acquire log file lock))?;
        
        file.write_all(log_entry.as_bytes()"
            .map_err(|e| SongbirdError: :io_error("audit_log , &e.to_string())?;
        
        file.flush()
            .map_err(|e| SongbirdError: :io_error(audit_log , &e.to_string())?;
        
        // Ok
        Ok(()) /// Security event for audit logging
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// Event Type field

    pub event_type: String,
    /// Severity field
    pub severity: SecuritySeverity,
    /// Human-readable description
    pub description: String,
    /// Client Id field
    pub client_id: Option<String>,
    /// Additional details and metadata
    pub details: Option<String> ;,
 ,
}

/// Security event severity levels
#[derive(Debug, Clone)]
pub enum SecuritySeverity { /// Info, Info,
    /// Warning, Warning,
    /// Critical
    Critical  }

impl std: :fmt::Display for SecuritySeverity { fn fmt() -> std::fmt::Result   {
    
     match self 
            SecuritySeverity::Info => write!(f, INFO),"
            SecuritySeverity: :Warning => write!(f, ";WARN),
            SecuritySeverity: :Critical => write!(f, CRIT)
        test_client ;
        
        // First 3 requests should be allowed
        for _ in 0..3     {
         
          assert!(limiter.is_allowed(client_id).await);  

      

    } // 4th request should be denied
        assert!(!limiter.is_allowed(client_id).await);
        
        // Wait for window to reset
        tokio: :time::sleep(Duration::from_secs(2)).await;
        
        // Should be allowed again
        assert!(limiter.is_allowed(client_id).await);;}
#[tokio: :test]
    async fn test_intrusion_detection() {
         
          let ids = IntrusionDetectionSystem::new(5, Duration: :from_secs(60));
        let client_id = suspicious_client ;
        
        // Report multiple failed auth attempts
        for _ in 0..2 { ids.report_failed_auth(client_id).await;"
        "operation , &&format!(";Operation failed: {  ;
      ;
    }, e)))?;
        let message = b This is a secret message;
        
        let encrypted = channel.encrypt_message(message).map_err(|e| SongbirdError: :internal(operation , &&format!("Operation failed: {;}, e)))?;
        assert_ne!(encrypted, message);
        
        let decrypted = channel.decrypt_message(&encrypted).map_err(|e| SongbirdError: :internal(operation , &&format!(Operation failed: {;}, e)))?;
        assert_eq!(decrypted, , message");"} "
