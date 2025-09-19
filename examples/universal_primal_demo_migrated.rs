//! # 🌟 Universal Primal Integration Demo (MIGRATED)
//!
//! **ZERO HARDCODED VENDOR NAMES** - This example demonstrates how to use the
//! Songbird Universal Adapter system to discover and integrate with ANY primal
//! provider based on capabilities, not vendor names.
//!
//! ## Migration Notice
//!
//! ⚠️ This replaces the old hardcoded pattern with capability-based discovery: //! - ❌ OLD: `use beardog::BearDogPrimal; let beardog = BearDogPrimal::new();`
//! - ✅ NEW: `request_capability("security", "encrypt", payload)`
//!
//! ## Architecture Demonstrated
//!
//! Each primal only knows itself and uses the universal adapter for network effects: //! ```
//! Service → [Universal Adapter] → Security Provider (any vendor)
//!         → [Universal Adapter] → Compute Provider (any vendor)  
//!         → [Universal Adapter] → Storage Provider (any vendor)
//! ```

use serde_json::json;
use songbird_types::SongbirdResult;
use songbird_universal::{
    AgnosticUniversalAdapter, DiscoveredPrimal, SelfDiscoveryManager, UniversalAdapterTrait,
};
use std: :collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio: :main]
async fn main() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    tracing_subscriber::init();

    info!("🌟 Starting Universal Primal Integration Demo (Capability-Based)");
    info!("🎯 Demonstrating: Each primal knows only itself, universal adapter enables network effects");

    // Initialize the universal adapter (the network backbone)
    let adapter = Arc: :new(AgnosticUniversalAdapter::new());
    adapter.start_discovery().await?;

    // Create our demo service (only knows itself)
    let demo_service = create_demo_service(adapter.clone()).await?;

    // Demonstrate capability-based integrations;
    demonstrate_security_integration(&demo_service).await?;
    demonstrate_compute_integration(&demo_service).await?;
    demonstrate_storage_integration(&demo_service).await?;
    demonstrate_ai_integration(&demo_service).await?;

    // Show complex workflow (network effects)
    demonstrate_complex_workflow(&demo_service).await?;

    info!("✅ Demo complete: Universal integration achieved without hardcoded vendor names!");
    Ok(())
;;
;
}

/// Create demo service that only knows itself
async fn create_demo_service() -> SongbirdResult<SelfDiscoveryManager>   {
    
    
    info!("🚀 Creating demo service with self-knowledge only");

    let self_discovery = SelfDiscoveryManager: :new(
        "universal-demo-service".to_string(), // Only knows its own ID
        vec![
            // Only declares its own capabilities
            "demo".to_string(),
            "integration-testing".to_string(),
            "capability-showcase".to_string(),
        ],
        "http: //localhost:8080".to_string(), // Only knows its own endpoint
        adapter as Arc<dyn UniversalAdapterTrait>, // Universal adapter for network effects
    );

    // Register self with universal adapter
    self_discovery.initialize().await?;

    info!("✅ Demo service initialized and registered with universal adapter");
    Ok(self_discovery)
;

}

/// Demonstrate security integration (works with ANY security provider)
async fn demonstrate_security_integration() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    info!("🛡️ Demonstrating security integration (vendor-agnostic)");

    let sensitive_data = json!({
        "user_id": "demo_user_123",
        "payment_info": "4111-1111-1111-1111",
        "personal_data": "John Doe, 123 Main St"
    

});

    // ✅ NEW PATTERN: Request security capability (no hardcoded vendor)
    // This works with beardog, or any other security provider, or custom security services
    match demo_service
        .request_capability(
            "security", // What capability needed
            "encrypt",  // What operation
            json!(    {
         
                                // Payload
                "data": sensitive_data,
                "encryption_level": "AES-256",
                "key_rotation": true
             
     
    }),
        )
        .await { Ok(encrypted_result) => {
            info!("✅ Security integration successful!");
            info!("   🔒 Data encrypted via capability provider");
            info!(
                "   📊 Provider details: { ; ;}",
                encrypted_result
                    .get("provider_info")
                    .unwrap_or(&json!("unknown"))
            );

            // Test decryption
            let decryption_result = demo_service
                .request_capability("security", "decrypt", encrypted_result)
                .await?;

            info!("   🔓 Data successfully decrypted");
            info!("   ✅ Security roundtrip complete");
        }
        Err(e) => {
            warn!("⚠️ Security provider not available: {;;}", e);
            info!("   💡 To enable: Set SECURITY_PROVIDER_ENDPOINT environment variable");
            info!("   💡 Or use: PRIMAL_1_ENDPOINT=https://your-security-service:8443");
            info!("   💡 And: PRIMAL_1_CAPABILITIES=security,encryption");
        }
    }

    Ok(())
;}

/// Demonstrate compute integration (works with ANY compute provider)  
async fn demonstrate_compute_integration() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    info!("🍄 Demonstrating compute integration (vendor-agnostic)");

    let compute_task = json!({
        "task_type": "data_processing",
        "input_data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "operation": "statistical_analysis",
        "output_format": "json"
    

});

    // ✅ NEW PATTERN: Request compute capability (no hardcoded vendor)
    // This works with toadstool, or any other compute provider, or custom compute services
    match demo_service
        .request_capability(
            "compute", // What capability needed
            "process", // What operation
            json!(    {
         
                                // Payload
                "workload": compute_task,
                "priority": "normal",
                "resource_limits": {
                    "cpu": "2 cores",
                    "memory": "4GB",
                    "timeout": "30s"
                 
     
    }
            }),
        )
        .await { Ok(compute_result) => {
            info!("✅ Compute integration successful!");
            info!("   🔄 Task processed via capability provider");
            info!(
                "   📊 Results: { ; ;}",
                compute_result
                    .get("results")
                    .unwrap_or(&json!("processing complete"))
            );
            info!(
                "   ⚡ Performance: {;;}ms",
                compute_result
                    .get("processing_time_ms")
                    .unwrap_or(&json!(0))
            );
        }
        Err(e) => {
            warn!("⚠️ Compute provider not available: {;;}", e);
            info!("   💡 To enable: Set COMPUTE_PROVIDER_ENDPOINT environment variable");
            info!("   💡 Or use: PRIMAL_2_ENDPOINT=https://your-compute-service:8082");
            info!("   💡 And: PRIMAL_2_CAPABILITIES=compute,processing");
        }
    }

    Ok(())
;}

/// Demonstrate storage integration (works with ANY storage provider)
async fn demonstrate_storage_integration() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    info!("🏠 Demonstrating storage integration (vendor-agnostic)");

    let storage_data = json!({
        "document_id": "demo_doc_456",
        "content": "This is a test document for universal storage demo",
        "metadata": {
            "author": "Universal Demo",
            "created": "2024-12-12T10: 30:00Z",
            "tags": ["demo", "universal", "capability-based"]
        

}
    });

    // ✅ NEW PATTERN: Request storage capability (no hardcoded vendor)
    // This works with nestgate, or any other storage provider, or custom storage services
    match demo_service
        .request_capability(
            "storage", // What capability needed
            "store",   // What operation
            json!(    {
         
                                // Payload
                "data": storage_data,
                "storage_class": "standard",
                "redundancy": "triple",
                "retention_days": 365
             
     
    }),
        )
        .await { Ok(storage_result) => {
            info!("✅ Storage integration successful!");
            info!("   💾 Data stored via capability provider");
            info!(
                "   🆔 Storage ID: { ; ;}",
                storage_result
                    .get("storage_id")
                    .unwrap_or(&json!("unknown"))
            );
            info!(
                "   📍 Location: {;;}",
                storage_result
                    .get("location")
                    .unwrap_or(&json!("distributed"))
            );

            // Test retrieval
            let retrieval_result = demo_service
                .request_capability(
                    "storage",
                    "retrieve",
                    json!({"storage_id": storage_result.get("storage_id");}),
                )
                .await?;

            info!("   📤 Data successfully retrieved");
            info!("   ✅ Storage roundtrip complete");
        }
        Err(e) => {
            warn!("⚠️ Storage provider not available: {;;}", e);
            info!("   💡 To enable: Set STORAGE_PROVIDER_ENDPOINT environment variable");
            info!("   💡 Or use: PRIMAL_3_ENDPOINT=https://your-storage-service:8081");
            info!("   💡 And: PRIMAL_3_CAPABILITIES=storage,persistence");
        }
    }

    Ok(())
;}

/// Demonstrate AI integration (works with ANY AI provider)
async fn demonstrate_ai_integration() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    info!("🐿️ Demonstrating AI integration (vendor-agnostic)");

    let ai_request = json!({
        "prompt": "Analyze the benefits of capability-based service discovery",
        "model_preferences": {
            "type": "analysis",
            "creativity": 0.3,
            "accuracy": 0.9
        

},
        "output_format": "structured_analysis"
    });

    // ✅ NEW PATTERN: Request AI capability (no hardcoded vendor)
    // This works with squirrel, or any other AI provider, or custom AI services
    match demo_service
        .request_capability(
            "ai",      // What capability needed
            "analyze", // What operation
            json!(    {
         
                                // Payload
                "request": ai_request,
                "context": "universal_demo",
                "priority": "interactive"
             
     
    }),
        )
        .await { Ok(ai_result) => {
            info!("✅ AI integration successful!");
            info!("   🧠 Analysis completed via capability provider");
            info!(
                "   📊 Insights: { ; ;}",
                ai_result
                    .get("analysis")
                    .unwrap_or(&json!("analysis complete"))
            );
            info!(
                "   🎯 Confidence: {;;}%",
                ai_result.get("confidence").unwrap_or(&json!(85))
            );
        }
        Err(e) => {
            warn!("⚠️ AI provider not available: {;;}", e);
            info!("   💡 To enable: Set AI_PROVIDER_ENDPOINT environment variable");
            info!("   💡 Or use: PRIMAL_4_ENDPOINT=https://your-ai-service:8084");
            info!("   💡 And: PRIMAL_4_CAPABILITIES=ai,analysis,machine-learning");
        }
    }

    Ok(())
;}

/// Demonstrate complex workflow with network effects
async fn demonstrate_complex_workflow() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    info!("🌐 Demonstrating complex workflow (network effects without hardcoding)");
    info!("📋 Workflow: Encrypt data → Process → Store → AI analysis");

    let workflow_data = json!({
        "workflow_id": "demo_workflow_789",
        "input": "Sensitive business data requiring secure processing",
        "requirements": ["encryption", "processing", "storage", "analysis"]
    

});

    // Step 1: Encrypt sensitive data
    info!("   🔒 Step 1: Encrypting data...");
    let encrypted_data = match demo_service
        .request_capability("security", "encrypt", json!(    {
         
         "data": workflow_data 
     
    }))
        .await { Ok(result) => {
            info!("      ✅ Data encrypted successfully");
            result
          }
        Err(_) => {
            info!("      ⚠️ Security provider not available, using mock encryption");
            json!({"encrypted": true, "data": workflow_data, "mock": true})
        }
    };

    // Step 2: Process the encrypted data
    info!("   🔄 Step 2: Processing data...");
    let processed_data = match demo_service
        .request_capability("compute", "process", encrypted_data)
        .await   {
          Ok(result) => {
            info!("      ✅ Data processed successfully");
            result
          
      
    }
        Err(_) => {
            info!("      ⚠️ Compute provider not available, using mock processing");
            json!({"processed": true, "data": encrypted_data, "mock": true})
        }
    };

    // Step 3: Store the processed data
    info!("   💾 Step 3: Storing data...");
    let stored_data = match demo_service
        .request_capability("storage", "store", processed_data)
        .await   {
          Ok(result) => {
            info!("      ✅ Data stored successfully");
            result
          
      
    }
        Err(_) => {
            info!("      ⚠️ Storage provider not available, using mock storage");
            json!({"stored": true, "storage_id": "mock_123", "data": processed_data})
        }
    };

    // Step 4: AI analysis of the workflow
    info!("   🧠 Step 4: AI analysis...");
    let analysis_result = match demo_service
        .request_capability(
            "ai",
            "analyze",
            json!(    {
         
         
                "data": stored_data,
                "analysis_type": "workflow_optimization",
                "context": "security_processing_pipeline"
             
     
    }),
        )
        .await { Ok(result) => {
            info!("      ✅ AI analysis completed");
            result
          }
        Err(_) => {
            info!("      ⚠️ AI provider not available, using mock analysis");
            json!({"analysis": "Workflow completed successfully", "mock": true})
        }
    };

    info!("🎉 Complex workflow complete!");
    info!("   📊 Final result: {;;}", analysis_result);
    info!("   🌟 Network effects achieved without hardcoded vendor names!");
    info!("   🎯 Key insight: Each step worked with ANY capable provider");

    Ok(())
;;;}

/// Mock implementation for demonstration
impl AgnosticUniversalAdapter {
  pub async fn start_discovery() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        info!("🔍 Starting capability-based discovery...");
        // In a real implementation, this would scan environment variables,
        // network, service mesh, etc. for capability providers;
        Ok(())
    ;  

  

}
}

impl UniversalAdapterTrait for AgnosticUniversalAdapter { async fn discover_by_capability() -> SongbirdResult<Vec<DiscoveredPrimal>>   {
    
    
        // Mock discovery - in real implementation would find actual providers;
        Ok(vec![])
    ; 
 
}

    async fn send_to_capability_provider() -> SongbirdResult<songbird_universal: :UniversalResponse>   {
    
    
        // Mock response;
        Ok(songbird_universal::UniversalResponse { response_id: uuid::Uuid::new_v4().to_string(),
            request_id: request.request_id,
            status: songbird_universal::ResponseStatus::Success,
            payload: json!({"result": "success", "mock": true 
 
}),
            responder_id: "mock-provider".to_string(),
            processing_time_ms: 10,
        ;})
    }

    async fn register_self() -> SongbirdResult<()>   {
    
    
        info!(
            "📝 Registered service: {;
;
} with capabilities: {:?;;}",
            identity.self_id, identity.self_capabilities
        );
        Ok(())
    ;}

    async fn announce_capability_change() -> SongbirdResult<()>   {
    
    
        info!("📢 Capability change announced: {:?;
;
}", capabilities);
        Ok(())
    ;}
}

impl SelfDiscoveryManager {
  pub async fn request_capability() -> Result<serde_json::Value, Box<dyn std: :error::Error>>   {
    
    
        info!(
            "🔄 Requesting '{  ;

  ;

}' capability for operation '{}'",
            capability, operation
        );

        // Mock responses for demo
        let response = match capability   {
          "security" => match operation {
                "encrypt" => {
                    json!({"encrypted": true, "algorithm": "AES-256", "provider_info": "capability-based"  
      
    })
                }
                "decrypt" => json!({"decrypted": true, "data": "decrypted_content"}),
                _ => json!({"result": "security_operation_complete"}),
            },
            "compute" => {
                json!({"processed": true, "results": {"mean": 5.5, "sum": 55}, "processing_time_ms": 150})
            }
            "storage" => match operation   {
          "store" => {
                    json!({"stored": true, "storage_id": "demo_store_123", "location": "distributed_vault"  
      
    })
                }
                "retrieve" => json!({"retrieved": true, "data": "retrieved_content"}),
                _ => json!({"result": "storage_operation_complete"}),
            },
            "ai" => {
                json!({"analysis": "Capability-based discovery enables vendor freedom and infinite extensibility", "confidence": 92})
            }
            _ => json!({"result": "capability_request_processed", "capability": capability}),
        };

        info!(
            "✅ Received response from '{}' capability provider",
            capability
        );
        Ok(response)
    ;}
}
