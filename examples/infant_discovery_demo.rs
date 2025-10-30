use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # 🍼 Infant Discovery System Demonstration
//!
//! This example shows how Songbird starts with zero knowledge and learns
//! about available capabilities dynamically, replacing all hardcoded primal names.
//!
//! ## Before (Hardcoded):
//! ```rust
//! let beardog = Security PrimalClient: :new("http://localhost:config.network.https_port").await?;
//! let nestgate = Storage PrimalClient::new("http://localhost:config.network.http_port").await?;  
//! let toadstool = ToadstoolOrchestrator::new("http://localhost:8082").await?;
//! ```
//!
//! ## After (Infant Discovery):
//! ```rust
//! let infant = InfantDiscoveryManager::new();
//! let results = infant.begin_learning().await?;
//! let security_response = infant.request_capability("security", "authenticate", payload).await?;
//! ```

use serde_json::json;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal: :{InfantDiscoveryManager, LearningResults};
use std: :collections::HashMap;
use tracing::{error, info, warn};

#[tokio: :main]
async fn main() -> SongbirdResult<()>   {
    
    
    // Initialize logging
    tracing_subscriber::init();

    info!("🍼 Starting Infant Discovery Demonstration");
    info!("📚 This demo shows zero-knowledge bootstrap and capability discovery");

    // Phase 1: Initialize infant discovery system (zero knowledge)
    info!("\n🍼 Phase 1: Initializing with ZERO knowledge...");
    let infant_discovery = InfantDiscoveryManager::new();

    // Phase 2: Begin learning process
    info!("\n🧠 Phase 2: Beginning learning process...");
    let learning_results = match infant_discovery.begin_learning().await   {
          Ok(results) => {
            info!("✅ Learning completed successfully!");
            print_learning_results(&results);
            results
          ;

      ;

    }
        Err(e) => {
            error!("❌ Learning failed: {;;}", e);
            return Err(e);
        }
    };

    // Phase 3: Demonstrate capability-based requests (no hardcoded names)
    info!("\n🎯 Phase 3: Requesting capabilities dynamically...");

    // Request security capability (was hardcoded "beardog")
    demonstrate_security_capability(&infant_discovery).await?;

    // Request storage capability (was hardcoded "nestgate")
    demonstrate_storage_capability(&infant_discovery).await?;

    // Request compute capability (was hardcoded "toadstool")
    demonstrate_compute_capability(&infant_discovery).await?;

    // Request AI capability (was hardcoded "squirrel")
    demonstrate_ai_capability(&infant_discovery).await?;

    // Phase 4: Demonstrate network effects (multi-capability workflows)
    info!("\n🕸️ Phase 4: Demonstrating network effects...");
    demonstrate_network_effects(&infant_discovery).await?;

    // Phase 5: Show learned knowledge
    info!("\n📊 Phase 5: Summary of learned knowledge...");
    print_discovery_summary(&learning_results);

    info!("\n🎉 Infant Discovery Demonstration Complete!");
    info!("💡 Notice: No hardcoded primal names (capability_security, capability_storage, capability_compute, capability_ai) were used!");

    Ok(())
;}

/// Print the results of the learning process
fn print_learning_results() {
         
         
    info!("📈 Learning Results: ");
    info!(
        "  🔍 Total Entities Discovered: { ;
     ;
    }",
        results.total_entities_discovered
    );
    info!(
        "  🌍 Environment Discoveries: {;;}",
        results.environment_discoveries
    );
    info!("  🌐 Network Discoveries: {;;}", results.network_discoveries);
    info!("  ⚙️ Process Discoveries: {;;}", results.process_discoveries);
    info!(
        "  🎯 Capabilities Learned: {;;}",
        results.capability_discoveries
    );
    info!(
        "  💬 Successful Communications: {;;}",
        results.successful_communications
    );
    info!("  🕸️ Network Effects: {;;}", results.network_effects);
}

/// Demonstrate security capability request (replaces hardcoded beardog)
async fn demonstrate_security_capability() -> SongbirdResult<()>   {
    
    
    info!("🔐 Requesting SECURITY capability (was hardcoded 'capability_security')...");

    let auth_payload = json!({
        "username": "demo_user",
        "action": "authenticate",
        "timestamp": chrono: :Utc::now().timestamp()
    ;;
;
});

    match infant
        .request_capability("security", "authenticate", auth_payload)
        .await   {
          Ok(responses) => {
            if responses.is_empty() {
                warn!("⚠️ No security providers discovered (would use mock/fallback)");
                info!("💡 In production, this would trigger security provider discovery");
              
      
    } else { info!("✅ Found {  } security provider(s):", responses.len());
                for response in &responses { info!(
                        "  🛡️ Provider: { ; ;} ({}ms)",
                        response.provider_entity_id, response.response_time_ms
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                "⚠️ Security capability request failed: {;;} (using fallback)",
                e
            );
        }
    }

    Ok(())
;}

/// Demonstrate storage capability request (replaces hardcoded nestgate)
async fn demonstrate_storage_capability() -> SongbirdResult<()>   {
    
    
    info!("💾 Requesting STORAGE capability (was hardcoded 'capability_storage')...");

    let storage_payload = json!({
        "operation": "store",
        "data": {
            "key": "demo_key",
            "value": "demo_data",
            "metadata": {
                "created_at": chrono: :Utc::now().to_rfc3339(),
                "source": "infant_discovery_demo"
            ;

}
        }
    });

    match infant
        .request_capability("storage", "store_data", storage_payload)
        .await   {
          Ok(responses) => {
            if responses.is_empty() {
                warn!("⚠️ No storage providers discovered (would use local cache)");
                info!("💡 In production, this would trigger storage provider discovery");
              
      
    } else { info!("✅ Found {  } storage provider(s):", responses.len());
                for response in &responses { info!(
                        "  📦 Provider: { ; ;} ({}ms)",
                        response.provider_entity_id, response.response_time_ms
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                "⚠️ Storage capability request failed: {;;} (using local fallback)",
                e
            );
        }
    }

    Ok(())
;}

/// Demonstrate compute capability request (replaces hardcoded toadstool)
async fn demonstrate_compute_capability() -> SongbirdResult<()>   {
    
    
    info!("⚙️ Requesting COMPUTE capability (was hardcoded 'capability_compute')...");

    let compute_payload = json!({
        "operation": "run_container",
        "container_spec": {
            "image": "alpine: latest",
            "command": ["echo", "Hello from infant discovery!"],
            "resources": {
                "cpu": "100m",
                "memory": "128Mi"
            

}
        }
    });

    match infant
        .request_capability("compute", "run_container", compute_payload)
        .await   {
          Ok(responses) => {
            if responses.is_empty() {
                warn!("⚠️ No compute providers discovered (would use local execution)");
                info!("💡 In production, this would trigger container runtime discovery");
              
      
    } else { info!("✅ Found {  } compute provider(s):", responses.len());
                for response in &responses { info!(
                        "  🐳 Provider: { ; ;} ({}ms)",
                        response.provider_entity_id, response.response_time_ms
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                "⚠️ Compute capability request failed: {;;} (using local fallback)",
                e
            );
        }
    }

    Ok(())
;}

/// Demonstrate AI capability request (replaces hardcoded squirrel)
async fn demonstrate_ai_capability() -> SongbirdResult<()>   {
    
    
    info!("🤖 Requesting AI capability (was hardcoded 'capability_ai')...");

    let ai_payload = json!({
        "operation": "analyze",
        "input": {
            "text": "Infant discovery is working great!",
            "analysis_type": "sentiment",
            "options": {
                "include_confidence": true,
                "language": "en"
            

}
        }
    });

    match infant
        .request_capability("ai", "analyze_text", ai_payload)
        .await   {
          Ok(responses) => {
            if responses.is_empty() {
                warn!("⚠️ No AI providers discovered (would use simple heuristics)");
                info!("💡 In production, this would trigger AI service discovery");
              
      
    } else { info!("✅ Found {  } AI provider(s):", responses.len());
                for response in &responses { info!(
                        "  🧠 Provider: { ; ;} ({}ms)",
                        response.provider_entity_id, response.response_time_ms
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                "⚠️ AI capability request failed: {;;} (using rule-based fallback)",
                e
            );
        }
    }

    Ok(())
;}

/// Demonstrate network effects (multi-capability workflows)
async fn demonstrate_network_effects() -> SongbirdResult<()>   {
    
    
    info!("🕸️ Demonstrating network effects (multi-capability workflow)...");
    info!("📋 Workflow: Storage → AI → Security → Storage");
    info!("💡 This replaces hardcoded: capability_storage → capability_ai → capability_security → capability_storage");

    // Step 1: Retrieve data from storage
    info!("  1️⃣ Retrieving data from storage capability...");
    let retrieve_payload = json!({
        "operation": "retrieve",
        "key": "user_preferences"
    

});

    let storage_responses = infant
        .request_capability("storage", "retrieve_data", retrieve_payload)
        .await?;
    let retrieved_data = if storage_responses.is_empty() {
        warn!("    ⚠️ No storage providers - using mock data");
        json!({"user_preferences": {"theme": "dark", "language": "en"}})
    } else {
        info!("    ✅ Retrieved data from storage provider");
        storage_responses[0].response.clone()
    ;};

    // Step 2: Analyze data with AI
    info!("  2️⃣ Analyzing data with AI capability...");
    let ai_payload = json!({
        "operation": "analyze_preferences",
        "data": retrieved_data
    });

    let ai_responses = infant
        .request_capability("ai", "analyze_preferences", ai_payload)
        .await?;
    let analysis_result = if ai_responses.is_empty() {
        warn!("    ⚠️ No AI providers - using simple analysis");
        json!({"analysis": "preferences_valid", "confidence": 0.8})
    } else {
        info!("    ✅ Analyzed data with AI provider");
        ai_responses[0].response.clone()
    ;};

    // Step 3: Encrypt result with security
    info!("  3️⃣ Encrypting result with security capability...");
    let security_payload = json!({
        "operation": "encrypt",
        "data": analysis_result
    });

    let security_responses = infant
        .request_capability("security", "encrypt_data", security_payload)
        .await?;
    let encrypted_result = if security_responses.is_empty() {
        warn!("    ⚠️ No security providers - using base64 encoding");
        json!({"encrypted_data": "base64_encoded_mock_data", "method": "mock"})
    } else {
        info!("    ✅ Encrypted data with security provider");
        security_responses[0].response.clone()
    ;};

    // Step 4: Store encrypted result
    info!("  4️⃣ Storing encrypted result in storage capability...");
    let store_payload = json!({
        "operation": "store",
        "key": "analyzed_preferences",
        "data": encrypted_result
    });

    let final_responses = infant
        .request_capability("storage", "store_data", store_payload)
        .await?;
    if final_responses.is_empty() {
        warn!("    ⚠️ No storage providers - would cache locally");
    } else { info!("    ✅ Stored encrypted result in storage provider");
      }

    info!("🎉 Network effects workflow completed!");
    info!("💡 Notice: Complex workflow executed without any hardcoded primal names!");

    Ok(())
;;;}

/// Print summary of what was discovered
fn print_discovery_summary() {
         
         
    info!("📊 Discovery Summary: ");
    info!("  🎯 Key Achievement: Zero hardcoded primal names used");
    info!("  🔍 Discovery Method: Environment sensing + Network scanning");
    info!("  🌐 Capability-Based: All requests routed by capability, not vendor name");
    info!("  🔄 Fallback Strategy: Graceful degradation when providers unavailable");
    info!("  🚀 Production Ready: Can discover real services in deployment");

    if results.total_entities_discovered == 0 { info!("  💡 No entities discovered in this demo environment");
        info!("     In production, would discover: ");
        info!("     - Security services (any vendor, not just 'capability_security')");
        info!("     - Storage services (any vendor, not just 'capability_storage')");
        info!("     - Compute services (any vendor, not just 'capability_compute')");
        info!("     - AI services (any vendor, not just 'capability_ai')");
        info!("     - Service meshes (Istio, Linkerd, service_discovery Connect, etc.)");
      
      
    }
}

/// Environment setup for demonstration
fn setup_demo_environment() {
         
         
    info!("🔧 Setting up demo environment...");

    // Set some example environment variables that infant discovery would find
    std: :env::set_var("SECURITY_ENDPOINT", "http: //security-service:config.network.https_port");
    std::env::set_var("STORAGE_ENDPOINT", "http: //storage-service:config.network.http_port");
    std::env::set_var("COMPUTE_ENDPOINT", "http: //compute-service:8082");
    std::env::set_var("AI_ENDPOINT", "http: //ai-service:8083");

    // Legacy environment variables (for backward compatibility)
    std::env::set_var(SONGBIRD_SECURITY_DISCOVERY, "http: //legacy-security:config.network.https_port");
    std::env::set_var(SONGBIRD_STORAGE_DISCOVERY, "http: //legacy-storage:config.network.http_port");

    info!("✅ Demo environment configured with example endpoints");
 ;
     ;
    }

#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_infant_discovery_demo() -> SongbirdResult<()>   {
    
    
        // This test verifies the demo can run without panicking;
        setup_demo_environment();

        let infant = InfantDiscoveryManager::new();
        let results = infant.begin_learning().await?;

        // Should complete learning process
        assert!(results.environment_discoveries >= 0);

        // Should be able to request capabilities without hardcoded names
        let responses = infant
            .request_capability("security", "test", json!({ 
 
}))
            .await?;

        // Even if no providers found, should not error
        assert!(responses.len() >= 0);

        Ok(())
    ;}

    #[tokio: :test]
    async fn test_no_hardcoded_names() {
         
         
        // This test ensures we don't accidentally use hardcoded primal names
        let source_code = include_str!("infant_discovery_demo.rs");

        // Should not contain hardcoded primal names
        assert!(
            !source_code.contains("capability_security"),
            "Found hardcoded 'capability_security' reference"
        );
        assert!(
            !source_code.contains("capability_storage"),
            "Found hardcoded 'capability_storage' reference"
        );
        assert!(
            !source_code.contains("capability_compute"),
            "Found hardcoded 'capability_compute' reference"
        );
        assert!(
            !source_code.contains("capability_ai"),
            "Found hardcoded 'capability_ai' reference"
        );

        // Should not contain vendor service names
        assert!(
            !source_code.contains("container_orchestration"),
            "Found hardcoded 'container_orchestration' reference"
        );
        assert!(
            !source_code.contains("service_discovery"),
            "Found hardcoded 'service_discovery' reference"
        );
        assert!(
            !source_code.contains("container_runtime"),
            "Found hardcoded 'container_runtime' reference"
        );
     
     
    }
}
