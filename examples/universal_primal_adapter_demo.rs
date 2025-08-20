/// Universal Primal Adapter Demonstration
///
/// This example shows how to use Songbird's universal primal adapter system
/// to interact with ANY primal through capability-based discovery, without
/// hardcoding specific primal names or endpoints.
use serde_json::json;
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::{
    PrimalResult, UniversalPrimalAdapter, UniversalPrimalAdapterBuilder,
};
use tracing::{error, info, warn};

#[tokio::main]
fn main(Result<(), Box<dyn std::error::Error>>) ->  {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Universal Primal Adapter Demo");
    info!("📋 This demo shows capability-based primal interaction");

    // Create universal adapter with configuration
    let adapter = UniversalPrimalAdapterBuilder::new()
        .with_timeout(30000) // 30 second timeout
        .with_retries(3) // 3 retry attempts
        .with_cache_ttl(300) // 5 minute cache TTL
        .enable_fallback(true) // Allow fallback to any available primal
        .build()
        .await?;

    info!("✅ Universal Primal Adapter created successfully");

    // Step 1: Discover all available primals
    info!("🔍 Step 1: Discovering available primals...");
    match adapter.discover_primals().await {
        Ok(discovered) => {
            info!(
                "✅ Discovered {} primals: {:?}",
                discovered.len(),
                discovered
            );

            // Show all primals with their capabilities
            let all_primals = adapter.get_all_primals().await;
            for (name, capabilities) in all_primals {
                info!("  📋 Primal '{}' provides: {:?}", name, capabilities);
            }
        }
        Err(e) => {
            warn!("⚠️ Discovery failed: {}", e);
            info!("💡 This is normal if no primals are configured");
            info!("💡 Set environment variables like:");
            info!("   PRIMAL_1_NAME=my-security-service");
            info!("   PRIMAL_1_ENDPOINT=https://localhost:8443");
            info!("   PRIMAL_1_CAPABILITIES=security,encryption");
        }
    }

    // Step 2: Demonstrate capability-based requests
    info!("🔧 Step 2: Demonstrating capability-based requests...");

    // Example 1: Security capability request
    info!("🔐 Attempting security operation...");
    match adapter
        .security_request(
            "encrypt",
            json!({
                "data": "sensitive information",
                "algorithm": "AES-256-GCM",
                "key_size": 256
            }),
        )
        .await
    {
        Ok(result) => {
            info!("✅ Security operation successful: {:?}", result);
        }
        Err(e) => {
            info!(
                "💡 Security operation failed (no security primal available): {}",
                e
            );
        }
    }

    // Example 2: AI capability request
    info!("🤖 Attempting AI operation...");
    match adapter
        .ai_request(
            "inference",
            json!({
                "model": "general",
                "input": "What is the meaning of life?",
                "max_tokens": 100
            }),
        )
        .await
    {
        Ok(result) => {
            info!("✅ AI operation successful: {:?}", result);
        }
        Err(e) => {
            info!("💡 AI operation failed (no AI primal available): {}", e);
        }
    }

    // Example 3: Storage capability request
    info!("💾 Attempting storage operation...");
    match adapter
        .storage_request(
            "store",
            json!({
                "key": "demo-data",
                "value": {"message": "Hello from Universal Adapter!"},
                "ttl": 3600
            }),
        )
        .await
    {
        Ok(result) => {
            info!("✅ Storage operation successful: {:?}", result);
        }
        Err(e) => {
            info!(
                "💡 Storage operation failed (no storage primal available): {}",
                e
            );
        }
    }

    // Example 4: Compute capability request
    info!("⚙️ Attempting compute operation...");
    match adapter
        .compute_request(
            "process",
            json!({
                "task": "fibonacci",
                "parameters": {"n": 10},
                "priority": "normal"
            }),
        )
        .await
    {
        Ok(result) => {
            info!("✅ Compute operation successful: {:?}", result);
        }
        Err(e) => {
            info!(
                "💡 Compute operation failed (no compute primal available): {}",
                e
            );
        }
    }

    // Step 3: Demonstrate dynamic capability discovery
    info!("🎯 Step 3: Finding best primal for specific capabilities...");

    let capabilities_to_test = vec!["security", "ai", "storage", "compute", "custom"];

    for capability in capabilities_to_test {
        match adapter.find_best_primal_for_capability(capability).await {
            Ok(Some(primal_name)) => {
                info!("✅ Best primal for '{}': {}", capability, primal_name);
            }
            Ok(None) => {
                info!("❌ No primal found for capability: {}", capability);
            }
            Err(e) => {
                error!("💥 Error finding primal for '{}': {}", capability, e);
            }
        }
    }

    // Step 4: Show health status of all primals
    info!("🏥 Step 4: Checking primal health status...");
    let health_status = adapter.get_primal_health_status().await;

    if health_status.is_empty() {
        info!("💡 No primals currently registered");
    } else {
        for (name, health) in health_status {
            info!("  💓 Primal '{}': {:?}", name, health);
        }
    }

    // Step 5: Demonstrate custom capability request
    info!("🔧 Step 5: Demonstrating custom capability request...");
    match adapter
        .send_capability_request(
            "custom_processing",
            "transform",
            json!({
                "input": [1, 2, 3, 4, 5],
                "operation": "square",
                "output_format": "json"
            }),
        )
        .await
    {
        Ok(result) => {
            info!("✅ Custom capability request successful: {:?}", result);
        }
        Err(e) => {
            info!(
                "💡 Custom capability request failed (no matching primal): {}",
                e
            );
        }
    }

    info!("🎉 Universal Primal Adapter Demo Complete!");
    info!("");
    info!("🔑 Key Takeaways:");
    info!("  1. Songbird doesn't know about specific primals by name");
    info!("  2. All communication is capability-based and name-agnostic");
    info!("  3. Any primal implementing the universal protocol can be used");
    info!("  4. Community primals can be added without code changes");
    info!("  5. Configuration is done via PRIMAL_{{i}}_* environment variables");
    info!("");
    info!("📖 To add a primal, set these environment variables:");
    info!("     PRIMAL_1_NAME=your-primal-name");
    info!("     PRIMAL_1_ENDPOINT=http://your-endpoint:port");
    info!("     PRIMAL_1_CAPABILITIES=capability1,capability2,capability3");
    info!("     PRIMAL_1_TYPE=your-primal-type");

    Ok(())
}

/// Example function showing how to integrate the universal adapter
/// into an existing application
fn integrate_with_existing_app(PrimalResult<()>) ->  {
    info!("🔌 Integrating Universal Adapter with existing application...");

    // Create adapter with custom configuration
    let _adapter = UniversalPrimalAdapterBuilder::new()
        .with_timeout(15000)
        .with_retries(2)
        .enable_fallback(false) // Strict mode - don't fallback
        .build()
        .await?;

    // Store adapter in your application state
    // (In real app, this would be in an Arc<Mutex<>> or similar)

    // Example integration patterns:

    // Example integration patterns (would use _adapter in real implementation):

    // 1. Request handler that needs security
    fn _handle_secure_request(PrimalResult<serde_json::Value>) ->  {
        // Find and use any primal with security capability
        adapter.security_request("validate", user_data).await
    }

    // 2. Background job that needs compute
    fn _process_background_job(PrimalResult<serde_json::Value>) ->  {
        // Find and use any primal with compute capability
        adapter.compute_request("batch_process", job_data).await
    }

    // 3. Data persistence that needs storage
    fn _save_user_preferences(PrimalResult<serde_json::Value>) ->  {
        // Find and use any primal with storage capability
        adapter
            .storage_request(
                "save",
                json!({
                    "collection": "user_preferences",
                    "key": user_id,
                    "data": preferences
                }),
            )
            .await
    }

    info!("✅ Application integration examples complete");
    Ok(())
}
