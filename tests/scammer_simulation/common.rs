use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//!
 //! Common Utilities for Scammer Simulation Tests
 //! 
 //! Shared helper functions and utilities for simulating scammer attacks
 //! and testing protection systems.
//!

use songbird_gaming_bridge::{
    errors::{Result, SongbirdError},
    network::gaming::{
        GamingAutoConfig, OneTouchConfig, SecurityValidator, SetupMethod, TrustLevel, TrustedDevice,
    },
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Simulate a scammer call scenario and return detection rate
pub async fn simulate_scammer_call(
    victim_type: &str,
    scammer_script: Vec<&str>,
    validator: &SecurityValidator,
) -> f32 {
    info!("🎭 Simulating scammer call for victim type: {}", victim_type);
    
    let mut detected_count = 0;
    let total_count = scammer_script.len();
    
    for (idx, script_line) in scammer_script.iter().enumerate() {
        info!("  📞 Scammer says: '{}'", script_line);
        
        // Test against scammer pattern detection
        let detected = validator
            .scammer_patterns
            .iter()
            .any(|pattern| script_line.to_lowercase().contains(pattern));
        
        if detected {
            detected_count += 1;
            info!("  ✅ DETECTED scammer pattern in line {}", idx + 1);
        } else {
            info!("  ⚠️ Pattern not detected in line {}", idx + 1);
        }
        
        // Add realistic timing between script lines
        sleep(Duration::from_millis(100)).await;
    }
    
    let detection_rate = detected_count as f32 / total_count as f32;
    info!(
        "📊 Call simulation complete: {}/{} patterns detected ({:.1}%)",
        detected_count,
        total_count,
        detection_rate * 100.0
    );
    
    detection_rate
}

/// Create a test trusted device
pub fn create_test_device(name: &str, trust_level: TrustLevel, is_family: bool) -> TrustedDevice {
    TrustedDevice {
        device_id: format!("test-device-{}", name.to_lowercase().replace(' ', "-")),
        device_name: name.to_string(),
        device_type: if is_family { "family".to_string() } else { "personal".to_string() },
        trust_level,
        last_seen: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        is_family_device: is_family,
        setup_permissions: if is_family {
            vec!["basic_gaming".to_string(), "parental_controls".to_string()]
        } else {
            vec!["full_access".to_string()]
        },
        allowed_hours: if is_family {
            Some("16:00-20:00".to_string()) // 4 PM to 8 PM for kids
        } else {
            None
        },
        mac_address: Some(format!("00:11:22:33:44:{:02x}", name.len())),
        ip_address: Some(format!("192.168.1.{}", 100 + name.len())),
    }
}

/// Create a family-safe test configuration
pub fn create_family_safe_test_config(family_name: &str) -> OneTouchConfig {
    OneTouchConfig {
        family_name: family_name.to_string(),
        setup_method: SetupMethod::FamilySafe,
        trusted_devices: vec![
            create_test_device("Parent Device", TrustLevel::High, true),
            create_test_device("Kid Device", TrustLevel::Medium, true),
        ],
        security_level: TrustLevel::High,
        allow_external_connections: false,
        require_parental_approval: true,
        gaming_time_limits: Some(HashMap::from([
            ("weekday".to_string(), "2 hours".to_string()),
            ("weekend".to_string(), "4 hours".to_string()),
        ])),
        blocked_content_categories: vec![
            "adult".to_string(),
            "violence".to_string(),
            "gambling".to_string(),
        ],
        notification_settings: HashMap::from([
            ("scammer_detection".to_string(), "immediate".to_string()),
            ("suspicious_activity".to_string(), "immediate".to_string()),
            ("setup_changes".to_string(), "immediate".to_string()),
        ]),
    }
}

/// Test detection rate and assert minimum threshold
pub fn assert_detection_rate(rate: f32, test_name: &str, minimum_threshold: f32) {
    info!(
        "📊 {} detection rate: {:.1}%",
        test_name,
        rate * 100.0
    );
    
    assert!(
        rate >= minimum_threshold,
        "{} detection rate too low: {:.1}% (minimum: {:.1}%)",
        test_name,
        rate * 100.0,
        minimum_threshold * 100.0
    );
    
    info!("✅ {} passed detection threshold", test_name);
}

/// Common scammer phrases for testing
pub fn get_common_scammer_phrases() -> Vec<&'static str> {
    vec![
        "microsoft technical support",
        "windows security department", 
        "your computer has been compromised",
        "suspicious activity detected",
        "download this software",
        "remote access",
        "teamviewer", 
        "your license has expired",
        "press windows key + r",
        "credit card information",
        "immediate action required",
        "your computer will crash",
    ]
}

/// Create test security validator
pub fn create_test_security_validator() -> SecurityValidator {
    SecurityValidator::new_family_safe()
}

/// Sleep helper for test timing
pub async fn test_sleep(millis: u64) {
    sleep(Duration::from_millis(millis)).await;
} 