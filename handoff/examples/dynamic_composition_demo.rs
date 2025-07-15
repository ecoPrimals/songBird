//! Dynamic Plugin Composition Demo
//!
//! This example demonstrates how services can be dynamically discovered
//! and composed without requiring static TOML configuration files.
//!
//! Scenarios demonstrated:
//! - BearDog + Songbird automatic integration
//! - Multiple Toadstool instances chained together
//! - 8-project composition discovery
//! - Real-time reconfiguration

use songbird_lib::network::gaming::primal::GamingPrimal;
use songbird_lib::primals::{beardog::BearDogPrimal, nestgate::NestGatePrimal, toadstool::ToadstoolPrimal};
use songbird_registry::plugin::{DynamicPluginRegistry, CompositionConstraints, PerformanceRequirements, PluginRegistry};
use songbird_discovery::traits::{
    ComposablePlugin, PluginCapability, PluginRequirement, PluginHealth, IntegrationResult
};
use serde_json::json;
use tracing::info;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    println!("🧩 Dynamic Plugin Composition Demo");
    println!("===================================");
    println!("🎯 Goal: Show how services work together like Lego blocks");
    println!("📋 No static TOML files required!");
    println!();

    // Create dynamic registry
    let registry = DynamicPluginRegistry::new();
    
    // Demo 1: Register various plugins
    demo_plugin_registration(&registry).await?;
    
    // Demo 2: Auto-discover BearDog + Songbird composition
    demo_beardog_songbird_composition(&registry).await?;
    
    // Demo 3: Chain multiple Toadstool instances
    demo_toadstool_chaining(&registry).await?;
    
    // Demo 4: Complex 8-project composition
    demo_complex_composition(&registry).await?;
    
    // Demo 5: Real-time reconfiguration
    demo_realtime_reconfiguration(&registry).await?;
    
    println!("✅ Dynamic composition demo completed successfully!");
    println!("🎉 Services can now work together without pre-configured TOML files!");
    
    Ok(())
}

async fn demo_plugin_registration(registry: &DynamicPluginRegistry) -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣  Plugin Registration Demo");
    println!("─────────────────────────────");
    
    // Register BearDog encryption plugin
    let beardog = Box::new(BearDogPlugin::new());
    let beardog_id = registry.register_plugin(beardog).await?;
    println!("✅ Registered BearDog: {}", beardog_id);
    
    // Register Songbird orchestration plugin
    let songbird = Box::new(SongbirdPlugin::new());
    let songbird_id = registry.register_plugin(songbird).await?;
    println!("✅ Registered Songbird: {}", songbird_id);
    
    // Register multiple Toadstool compute plugins
    for i in 1..=3 {
        let toadstool = Box::new(ToadstoolPlugin::new(i));
        let toadstool_id = registry.register_plugin(toadstool).await?;
        println!("✅ Registered Toadstool-{}: {}", i, toadstool_id);
    }
    
    // Register other hypothetical plugins
    let plugins = vec![
        ("DataLake", Box::new(DataLakePlugin::new()) as Box<dyn ComposablePlugin>),
        ("MLPipeline", Box::new(MLPipelinePlugin::new()) as Box<dyn ComposablePlugin>),
        ("WebAPI", Box::new(WebAPIPlugin::new()) as Box<dyn ComposablePlugin>),
        ("Monitor", Box::new(MonitorPlugin::new()) as Box<dyn ComposablePlugin>),
        ("Backup", Box::new(BackupPlugin::new()) as Box<dyn ComposablePlugin>),
    ];
    
    for (name, plugin) in plugins {
        let plugin_id = registry.register_plugin(plugin).await?;
        println!("✅ Registered {}: {}", name, plugin_id);
    }
    
    let all_plugins = registry.list_plugins().await;
    println!("📊 Total registered plugins: {}", all_plugins.len());
    println!();
    
    Ok(())
}

async fn demo_beardog_songbird_composition(registry: &DynamicPluginRegistry) -> Result<(), Box<dyn std::error::Error>> {
    println!("2️⃣  BearDog + Songbird Auto-Composition");
    println!("────────────────────────────────────────");
    
    // Define what we need: secure service orchestration
    let required_capabilities = vec![
        PluginCapability::Encryption { algorithms: vec!["AES-256".to_string()] },
        PluginCapability::ServiceDiscovery { protocols: vec!["HTTP".to_string()] },
        PluginCapability::LoadBalancing { strategies: vec!["round-robin".to_string()] },
    ];
    
    // Auto-discover optimal composition
    let constraints = CompositionConstraints {
        max_latency_ms: Some(50.0),
        max_memory_mb: Some(512.0),
        max_plugins: Some(5),
        required_performance: Some(PerformanceRequirements {
            min_throughput_rps: 1000.0,
            max_latency_ms: 50.0,
            max_cpu_percent: 80.0,
            max_memory_mb: 512.0,
        }),
        security_level: Some("confidential".to_string()),
    };
    
    let compositions = registry.discover_optimal_composition(
        "Secure service orchestration with encryption",
        required_capabilities,
        constraints,
    ).await?;
    
    println!("🔍 Found {} possible compositions", compositions.len());
    
    if let Some(best_plan) = compositions.first() {
        println!("🏆 Best composition plan:");
        println!("   Plugins: {:?}", best_plan.plugins);
        println!("   Integration order: {:?}", best_plan.integration_order);
        println!("   Estimated latency: {:.2}ms", best_plan.estimated_performance.latency_ms);
        println!("   Estimated throughput: {:.0} RPS", best_plan.estimated_performance.throughput_rps);
        
        // Execute the composition
        let composed_system = registry.execute_composition(best_plan.clone()).await?;
        println!("✅ Composed system created: {}", composed_system.system_id);
        println!("   Active plugins: {:?}", composed_system.active_plugins);
        println!("   System capabilities: {} total", composed_system.system_capabilities.len());
        println!("   Overall health: {}", composed_system.system_health.overall_healthy);
    }
    
    println!();
    Ok(())
}

async fn demo_toadstool_chaining(registry: &DynamicPluginRegistry) -> Result<(), Box<dyn std::error::Error>> {
    println!("3️⃣  Toadstool Chaining Demo (Toadstool on Toadstool)");
    println!("─────────────────────────────────────────────────────");
    
    // Find all Toadstool plugins
    let all_plugins = registry.list_plugins().await;
    let toadstool_plugins: Vec<_> = all_plugins.iter()
        .filter(|id| id.contains("toadstool"))
        .collect();
    
    println!("🍄 Found {} Toadstool instances", toadstool_plugins.len());
    
    // Create a compute pipeline using multiple Toadstools
    let compute_capabilities = vec![
        PluginCapability::Compute { cpu_cores: 16, memory_gb: 32 },
    ];
    
    let plan = registry.auto_compose(compute_capabilities).await?;
    println!("🔗 Toadstool chain composition:");
    println!("   Plugins: {:?}", plan.plugins);
    println!("   Total estimated compute: {:.0} CPU units", plan.estimated_performance.cpu_utilization_percent);
    
    // Show how they can be chained
    println!("🔄 Chaining pattern:");
    for (i, plugin_id) in plan.plugins.iter().enumerate() {
        if i == 0 {
            println!("   📥 Input → {}", plugin_id);
        } else if i == plan.plugins.len() - 1 {
            println!("   {} → 📤 Output", plugin_id);
        } else {
            println!("   {} → (processing)", plugin_id);
        }
    }
    
    println!();
    Ok(())
}

async fn demo_complex_composition(registry: &DynamicPluginRegistry) -> Result<(), Box<dyn std::error::Error>> {
    println!("4️⃣  Complex 8-Project Composition");
    println!("──────────────────────────────────");
    
    // Define a complex ML pipeline requirement
    let ml_pipeline_capabilities = vec![
        PluginCapability::Storage { capacity_gb: 1000, storage_type: "SSD".to_string() },
        PluginCapability::Compute { cpu_cores: 32, memory_gb: 64 },
        PluginCapability::Network { bandwidth_mbps: 1000, latency_ms: 10 },
        PluginCapability::Encryption { algorithms: vec!["AES-256".to_string()] },
        PluginCapability::Custom { 
            name: "MLInference".to_string(), 
            attributes: HashMap::from([("model_type".to_string(), "transformer".to_string())])
        },
    ];
    
    let constraints = CompositionConstraints {
        max_latency_ms: Some(200.0),
        max_memory_mb: Some(2048.0),
        max_plugins: Some(8),
        required_performance: Some(PerformanceRequirements {
            min_throughput_rps: 100.0,
            max_latency_ms: 200.0,
            max_cpu_percent: 90.0,
            max_memory_mb: 2048.0,
        }),
        security_level: Some("confidential".to_string()),
    };
    
    let compositions = registry.discover_optimal_composition(
        "Complex ML pipeline with encryption and monitoring",
        ml_pipeline_capabilities,
        constraints,
    ).await?;
    
    println!("🧠 Complex ML Pipeline Compositions:");
    for (i, plan) in compositions.iter().take(3).enumerate() {
        println!("   Option {}: {} plugins", i + 1, plan.plugins.len());
        println!("     Plugins: {:?}", plan.plugins);
        println!("     Performance: {:.0} RPS, {:.1}ms latency", 
                 plan.estimated_performance.throughput_rps,
                 plan.estimated_performance.latency_ms);
        println!();
    }
    
    Ok(())
}

async fn demo_realtime_reconfiguration(registry: &DynamicPluginRegistry) -> Result<(), Box<dyn std::error::Error>> {
    println!("5️⃣  Real-time Reconfiguration Demo");
    println!("───────────────────────────────────");
    
    // Subscribe to plugin events
    let mut event_receiver = registry.subscribe_events();
    
    // Simulate adding a new plugin at runtime
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        println!("🔄 Adding new HighPerformanceCompute plugin at runtime...");
        // This would trigger recomposition automatically
    });
    
    // Listen for events briefly
    tokio::select! {
        _ = sleep(Duration::from_millis(200)) => {
            println!("⏰ Event listening timeout");
        }
        event = event_receiver.recv() => {
            if let Ok(event) = event {
                println!("📢 Plugin event: {:?}", event);
            }
        }
    }
    
    println!("✅ Real-time reconfiguration capability demonstrated");
    println!();
    
    Ok(())
}

// ============================================================================
// Plugin Implementations
// ============================================================================

/// BearDog encryption plugin
struct BearDogPlugin {
    id: String,
}

impl BearDogPlugin {
    fn new() -> Self {
        Self {
            id: "beardog-encryption".to_string(),
        }
    }
}

#[async_trait]
impl ComposablePlugin for BearDogPlugin {
    fn plugin_id(&self) -> &str {
        &self.id
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability::Encryption { 
                algorithms: vec!["AES-256".to_string(), "ChaCha20".to_string()] 
            },
            PluginCapability::Custom { 
                name: "KeyManagement".to_string(),
                attributes: HashMap::from([("hsm_support".to_string(), "true".to_string())])
            },
        ]
    }
    
    fn requirements(&self) -> Vec<PluginRequirement> {
        vec![] // BearDog is self-contained
    }
    
    fn can_integrate_with(&self, other: &dyn ComposablePlugin) -> bool {
        // BearDog can integrate with any plugin that needs encryption
        other.requirements().iter().any(|req| matches!(req, PluginRequirement::RequiresEncryption { .. }))
    }
    
    async fn integrate_with(&mut self, _other: &dyn ComposablePlugin) -> Result<IntegrationResult> {
        Ok(IntegrationResult {
            success: true,
            integration_id: uuid::Uuid::new_v4().to_string(),
            shared_capabilities: vec![
                PluginCapability::Encryption { algorithms: vec!["AES-256".to_string()] }
            ],
            configuration_updates: None,
            error_message: None,
        })
    }
    
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "security_level": {"type": "string", "enum": ["standard", "confidential", "secret"]},
                "key_rotation_hours": {"type": "integer", "minimum": 1, "maximum": 8760}
            }
        })
    }
    
    fn apply_config(&mut self, _config: serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> PluginHealth {
        PluginHealth {
            healthy: true,
            status_message: "BearDog encryption services operational".to_string(),
            last_check: chrono::Utc::now(),
            performance_metrics: HashMap::from([
                ("encryption_ops_per_sec".to_string(), 10000.0),
                ("key_rotation_status".to_string(), 1.0),
            ]),
        }
    }
}

/// Songbird orchestration plugin
struct SongbirdPlugin {
    id: String,
}

impl SongbirdPlugin {
    fn new() -> Self {
        Self {
            id: "songbird-orchestrator".to_string(),
        }
    }
}

#[async_trait]
impl ComposablePlugin for SongbirdPlugin {
    fn plugin_id(&self) -> &str {
        &self.id
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability::ServiceDiscovery { protocols: vec!["HTTP".to_string(), "gRPC".to_string()] },
            PluginCapability::LoadBalancing { strategies: vec!["round-robin".to_string(), "least-connections".to_string()] },
            PluginCapability::GamingBridge { protocols: vec!["IPX".to_string(), "DirectPlay".to_string()] },
        ]
    }
    
    fn requirements(&self) -> Vec<PluginRequirement> {
        vec![
            PluginRequirement::RequiresEncryption { min_key_size: Some(256) },
        ]
    }
    
    fn can_integrate_with(&self, other: &dyn ComposablePlugin) -> bool {
        other.capabilities().iter().any(|cap| matches!(cap, PluginCapability::Encryption { .. }))
    }
    
    async fn integrate_with(&mut self, _other: &dyn ComposablePlugin) -> Result<IntegrationResult> {
        Ok(IntegrationResult {
            success: true,
            integration_id: uuid::Uuid::new_v4().to_string(),
            shared_capabilities: vec![
                PluginCapability::ServiceDiscovery { protocols: vec!["HTTPS".to_string()] }
            ],
            configuration_updates: Some(serde_json::json!({"tls_enabled": true})),
            error_message: None,
        })
    }
    
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "load_balancing_strategy": {"type": "string", "enum": ["round-robin", "least-connections", "health-aware"]},
                "gaming_protocols": {"type": "array", "items": {"type": "string"}}
            }
        })
    }
    
    fn apply_config(&mut self, _config: serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> PluginHealth {
        PluginHealth {
            healthy: true,
            status_message: "Songbird orchestration services operational".to_string(),
            last_check: chrono::Utc::now(),
            performance_metrics: HashMap::from([
                ("active_services".to_string(), 15.0),
                ("requests_per_second".to_string(), 1250.0),
            ]),
        }
    }
}

/// Toadstool compute plugin
struct ToadstoolPlugin {
    id: String,
    instance: u32,
}

impl ToadstoolPlugin {
    fn new(instance: u32) -> Self {
        Self {
            id: format!("toadstool-compute-{}", instance),
            instance,
        }
    }
}

#[async_trait]
impl ComposablePlugin for ToadstoolPlugin {
    fn plugin_id(&self) -> &str {
        &self.id
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability::Compute { cpu_cores: 8 * self.instance, memory_gb: 16 * self.instance },
            PluginCapability::Custom { 
                name: "DistributedCompute".to_string(),
                attributes: HashMap::from([("can_chain".to_string(), "true".to_string())])
            },
        ]
    }
    
    fn requirements(&self) -> Vec<PluginRequirement> {
        vec![
            PluginRequirement::RequiresNetwork { min_bandwidth_mbps: 100, max_latency_ms: 50 },
        ]
    }
    
    fn can_integrate_with(&self, other: &dyn ComposablePlugin) -> bool {
        // Toadstool can chain with other Toadstools or work with any service
        other.plugin_id().contains("toadstool") || 
        other.capabilities().iter().any(|cap| matches!(cap, PluginCapability::Network { .. }))
    }
    
    async fn integrate_with(&mut self, _other: &dyn ComposablePlugin) -> Result<IntegrationResult> {
        Ok(IntegrationResult {
            success: true,
            integration_id: uuid::Uuid::new_v4().to_string(),
            shared_capabilities: vec![
                PluginCapability::Compute { cpu_cores: 8 * self.instance, memory_gb: 16 * self.instance }
            ],
            configuration_updates: None,
            error_message: None,
        })
    }
    
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "compute_mode": {"type": "string", "enum": ["batch", "realtime", "hybrid"]},
                "resource_limits": {
                    "type": "object",
                    "properties": {
                        "max_cpu_percent": {"type": "number", "minimum": 0, "maximum": 100},
                        "max_memory_gb": {"type": "number", "minimum": 1}
                    }
                }
            }
        })
    }
    
    fn apply_config(&mut self, _config: serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> PluginHealth {
        PluginHealth {
            healthy: true,
            status_message: format!("Toadstool compute instance {} operational", self.instance),
            last_check: chrono::Utc::now(),
            performance_metrics: HashMap::from([
                ("cpu_utilization".to_string(), 45.0),
                ("memory_usage_gb".to_string(), 8.0 * self.instance as f64),
                ("active_tasks".to_string(), 12.0),
            ]),
        }
    }
}

// Simplified implementations for other plugins
macro_rules! simple_plugin {
    ($name:ident, $id:expr, $capabilities:expr) => {
        struct $name { id: String }
        impl $name { fn new() -> Self { Self { id: $id.to_string() } } }
        
        #[async_trait]
        impl ComposablePlugin for $name {
            fn plugin_id(&self) -> &str { &self.id }
            fn capabilities(&self) -> Vec<PluginCapability> { $capabilities }
            fn requirements(&self) -> Vec<PluginRequirement> { vec![] }
            fn can_integrate_with(&self, _other: &dyn ComposablePlugin) -> bool { true }
            async fn integrate_with(&mut self, _other: &dyn ComposablePlugin) -> Result<IntegrationResult> {
                Ok(IntegrationResult {
                    success: true,
                    integration_id: uuid::Uuid::new_v4().to_string(),
                    shared_capabilities: vec![],
                    configuration_updates: None,
                    error_message: None,
                })
            }
            fn config_schema(&self) -> serde_json::Value { serde_json::json!({}) }
            fn apply_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
            async fn health_check(&self) -> PluginHealth {
                PluginHealth {
                    healthy: true,
                    status_message: format!("{} operational", self.id),
                    last_check: chrono::Utc::now(),
                    performance_metrics: HashMap::new(),
                }
            }
        }
    };
}

simple_plugin!(DataLakePlugin, "datalake-storage", vec![
    PluginCapability::Storage { capacity_gb: 10000, storage_type: "Object".to_string() }
]);

simple_plugin!(MLPipelinePlugin, "ml-pipeline", vec![
    PluginCapability::Custom { 
        name: "MLInference".to_string(),
        attributes: HashMap::from([("model_type".to_string(), "transformer".to_string())])
    }
]);

simple_plugin!(WebAPIPlugin, "web-api", vec![
    PluginCapability::Network { bandwidth_mbps: 1000, latency_ms: 5 }
]);

simple_plugin!(MonitorPlugin, "monitoring", vec![
    PluginCapability::Custom { 
        name: "Monitoring".to_string(),
        attributes: HashMap::from([("metrics".to_string(), "prometheus".to_string())])
    }
]);

simple_plugin!(BackupPlugin, "backup-service", vec![
    PluginCapability::Storage { capacity_gb: 5000, storage_type: "Archive".to_string() }
]); 