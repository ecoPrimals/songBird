//! Universal capability system for ecosystem integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal service capabilities - extensible enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceCapability {
    // Compute capabilities (ToadStool)
    ContainerRuntime {
        orchestrators: Vec<String>,
    },
    ServerlessExecution {
        languages: Vec<String>,
    },
    GpuAcceleration {
        cuda_support: bool,
    },
    NativeExecution {
        architectures: Vec<String>,
    },
    WasmExecution {
        wasi_support: bool,
    },

    // Security capabilities (BearDog)
    Authentication {
        methods: Vec<String>,
    },
    Encryption {
        algorithms: Vec<String>,
    },
    KeyManagement {
        hsm_support: bool,
    },
    ThreatDetection {
        ml_enabled: bool,
    },
    Compliance {
        frameworks: Vec<String>,
    },

    // Storage capabilities (NestGate)
    FileSystem {
        supports_zfs: bool,
    },
    ObjectStorage {
        backends: Vec<String>,
    },
    DataReplication {
        consistency: String,
    },
    VolumeManagement {
        protocols: Vec<String>,
    },
    BackupRestore {
        incremental: bool,
    },

    // Network capabilities (Songbird)
    ServiceDiscovery {
        protocols: Vec<String>,
    },
    NetworkRouting {
        protocols: Vec<String>,
    },
    LoadBalancing {
        algorithms: Vec<String>,
    },
    CircuitBreaking {
        enabled: bool,
    },

    // AI capabilities (Squirrel)
    ModelInference {
        models: Vec<String>,
    },
    AgentFramework {
        mcp_support: bool,
    },
    MachineLearning {
        training_support: bool,
    },
    NaturalLanguage {
        languages: Vec<String>,
    },

    // OS capabilities (biomeOS)
    Orchestration {
        primals: Vec<String>,
    },
    Manifests {
        formats: Vec<String>,
    },
    Deployment {
        strategies: Vec<String>,
    },
    Monitoring {
        metrics: Vec<String>,
    },

    // Custom capabilities (extensible)
    Custom {
        name: String,
        version: String,
        metadata: HashMap<String, serde_json::Value>,
    },
}

impl ServiceCapability {
    /// Get the capability category for validation
    pub fn category(&self) -> &str {
        match self {
            ServiceCapability::ContainerRuntime { .. } => "compute",
            ServiceCapability::ServerlessExecution { .. } => "compute",
            ServiceCapability::GpuAcceleration { .. } => "compute",
            ServiceCapability::NativeExecution { .. } => "compute",
            ServiceCapability::WasmExecution { .. } => "compute",

            ServiceCapability::Authentication { .. } => "security",
            ServiceCapability::Encryption { .. } => "security",
            ServiceCapability::KeyManagement { .. } => "security",
            ServiceCapability::ThreatDetection { .. } => "security",
            ServiceCapability::Compliance { .. } => "security",

            ServiceCapability::FileSystem { .. } => "storage",
            ServiceCapability::ObjectStorage { .. } => "storage",
            ServiceCapability::DataReplication { .. } => "storage",
            ServiceCapability::VolumeManagement { .. } => "storage",
            ServiceCapability::BackupRestore { .. } => "storage",

            ServiceCapability::ServiceDiscovery { .. } => "network",
            ServiceCapability::NetworkRouting { .. } => "network",
            ServiceCapability::LoadBalancing { .. } => "network",
            ServiceCapability::CircuitBreaking { .. } => "network",

            ServiceCapability::ModelInference { .. } => "ai",
            ServiceCapability::AgentFramework { .. } => "ai",
            ServiceCapability::MachineLearning { .. } => "ai",
            ServiceCapability::NaturalLanguage { .. } => "ai",

            ServiceCapability::Orchestration { .. } => "orchestration",
            ServiceCapability::Manifests { .. } => "orchestration",
            ServiceCapability::Deployment { .. } => "orchestration",
            ServiceCapability::Monitoring { .. } => "orchestration",

            ServiceCapability::Custom { .. } => "custom",
        }
    }

    /// Get a human-readable name for the capability
    pub fn name(&self) -> String {
        match self {
            ServiceCapability::ContainerRuntime { .. } => "Container Runtime".to_string(),
            ServiceCapability::ServerlessExecution { .. } => "Serverless Execution".to_string(),
            ServiceCapability::GpuAcceleration { .. } => "GPU Acceleration".to_string(),
            ServiceCapability::NativeExecution { .. } => "Native Execution".to_string(),
            ServiceCapability::WasmExecution { .. } => "WASM Execution".to_string(),

            ServiceCapability::Authentication { .. } => "Authentication".to_string(),
            ServiceCapability::Encryption { .. } => "Encryption".to_string(),
            ServiceCapability::KeyManagement { .. } => "Key Management".to_string(),
            ServiceCapability::ThreatDetection { .. } => "Threat Detection".to_string(),
            ServiceCapability::Compliance { .. } => "Compliance".to_string(),

            ServiceCapability::FileSystem { .. } => "File System".to_string(),
            ServiceCapability::ObjectStorage { .. } => "Object Storage".to_string(),
            ServiceCapability::DataReplication { .. } => "Data Replication".to_string(),
            ServiceCapability::VolumeManagement { .. } => "Volume Management".to_string(),
            ServiceCapability::BackupRestore { .. } => "Backup & Restore".to_string(),

            ServiceCapability::ServiceDiscovery { .. } => "Service Discovery".to_string(),
            ServiceCapability::NetworkRouting { .. } => "Network Routing".to_string(),
            ServiceCapability::LoadBalancing { .. } => "Load Balancing".to_string(),
            ServiceCapability::CircuitBreaking { .. } => "Circuit Breaking".to_string(),

            ServiceCapability::ModelInference { .. } => "Model Inference".to_string(),
            ServiceCapability::AgentFramework { .. } => "Agent Framework".to_string(),
            ServiceCapability::MachineLearning { .. } => "Machine Learning".to_string(),
            ServiceCapability::NaturalLanguage { .. } => "Natural Language".to_string(),

            ServiceCapability::Orchestration { .. } => "Orchestration".to_string(),
            ServiceCapability::Manifests { .. } => "Manifests".to_string(),
            ServiceCapability::Deployment { .. } => "Deployment".to_string(),
            ServiceCapability::Monitoring { .. } => "Monitoring".to_string(),

            ServiceCapability::Custom { name, .. } => name.clone(),
        }
    }
}

/// Universal capability requirements for service matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityRequirement {
    // Compute requirements
    ContainerRuntime {
        required_orchestrator: String,
    },
    ServerlessExecution {
        required_language: String,
    },
    GpuAcceleration {
        cuda_required: bool,
    },
    NativeExecution {
        required_architecture: String,
    },
    WasmExecution {
        wasi_required: bool,
    },

    // Security requirements
    Authentication {
        required_method: String,
    },
    Encryption {
        required_algorithm: String,
    },
    KeyManagement {
        hsm_required: bool,
    },
    ThreatDetection {
        ml_required: bool,
    },
    Compliance {
        required_framework: String,
    },

    // Storage requirements
    FileSystem {
        zfs_required: bool,
    },
    ObjectStorage {
        required_backend: String,
    },
    DataReplication {
        required_consistency: String,
    },
    VolumeManagement {
        required_protocol: String,
    },
    BackupRestore {
        incremental_required: bool,
    },

    // Network requirements
    ServiceDiscovery {
        required_protocol: String,
    },
    NetworkRouting {
        required_protocol: String,
    },
    LoadBalancing {
        required_algorithm: String,
    },
    CircuitBreaking {
        required: bool,
    },

    // AI requirements
    ModelInference {
        required_model: String,
    },
    AgentFramework {
        mcp_required: bool,
    },
    MachineLearning {
        training_required: bool,
    },
    NaturalLanguage {
        required_language: String,
    },

    // OS requirements
    Orchestration {
        required_primal: String,
    },
    Manifests {
        required_format: String,
    },
    Deployment {
        required_strategy: String,
    },
    Monitoring {
        required_metric: String,
    },

    // Custom requirements
    Custom {
        name: String,
        version: String,
        requirements: HashMap<String, serde_json::Value>,
    },
}

impl CapabilityRequirement {
    /// Check if a capability satisfies this requirement
    pub fn is_satisfied_by(&self, capability: &ServiceCapability) -> bool {
        match (self, capability) {
            (
                CapabilityRequirement::ContainerRuntime {
                    required_orchestrator,
                },
                ServiceCapability::ContainerRuntime { orchestrators },
            ) => orchestrators.contains(required_orchestrator),
            (
                CapabilityRequirement::Authentication { required_method },
                ServiceCapability::Authentication { methods },
            ) => methods.contains(required_method),
            (
                CapabilityRequirement::Custom { name: req_name, .. },
                ServiceCapability::Custom { name: cap_name, .. },
            ) => req_name == cap_name,
            // Add more matching logic as needed
            _ => false,
        }
    }
}

/// Universal capability validator trait
#[async_trait::async_trait]
pub trait CapabilityValidator: Send + Sync {
    /// Validate a capability
    async fn validate(&self, capability: &ServiceCapability) -> Result<(), crate::CapabilityError>;

    /// Get the category this validator handles
    fn category(&self) -> &str;
}

/// Built-in capability validators
pub struct ComputeCapabilityValidator;

#[async_trait::async_trait]
impl CapabilityValidator for ComputeCapabilityValidator {
    async fn validate(&self, capability: &ServiceCapability) -> Result<(), crate::CapabilityError> {
        match capability {
            ServiceCapability::ContainerRuntime { orchestrators } => {
                if orchestrators.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Container runtime must specify at least one orchestrator".to_string(),
                    ));
                }
            }
            ServiceCapability::ServerlessExecution { languages } => {
                if languages.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Serverless execution must specify at least one language".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn category(&self) -> &str {
        "compute"
    }
}

pub struct StorageCapabilityValidator;

#[async_trait::async_trait]
impl CapabilityValidator for StorageCapabilityValidator {
    async fn validate(&self, capability: &ServiceCapability) -> Result<(), crate::CapabilityError> {
        match capability {
            ServiceCapability::ObjectStorage { backends } => {
                if backends.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Object storage must specify at least one backend".to_string(),
                    ));
                }
            }
            ServiceCapability::VolumeManagement { protocols } => {
                if protocols.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Volume management must specify at least one protocol".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn category(&self) -> &str {
        "storage"
    }
}

pub struct SecurityCapabilityValidator;

#[async_trait::async_trait]
impl CapabilityValidator for SecurityCapabilityValidator {
    async fn validate(&self, capability: &ServiceCapability) -> Result<(), crate::CapabilityError> {
        match capability {
            ServiceCapability::Authentication { methods } => {
                if methods.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Authentication must specify at least one method".to_string(),
                    ));
                }
            }
            ServiceCapability::Encryption { algorithms } => {
                if algorithms.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Encryption must specify at least one algorithm".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn category(&self) -> &str {
        "security"
    }
}

pub struct AICapabilityValidator;

#[async_trait::async_trait]
impl CapabilityValidator for AICapabilityValidator {
    async fn validate(&self, capability: &ServiceCapability) -> Result<(), crate::CapabilityError> {
        match capability {
            ServiceCapability::ModelInference { models } => {
                if models.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Model inference must specify at least one model".to_string(),
                    ));
                }
            }
            ServiceCapability::NaturalLanguage { languages } => {
                if languages.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Natural language must specify at least one language".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn category(&self) -> &str {
        "ai"
    }
}

pub struct NetworkCapabilityValidator;

#[async_trait::async_trait]
impl CapabilityValidator for NetworkCapabilityValidator {
    async fn validate(&self, capability: &ServiceCapability) -> Result<(), crate::CapabilityError> {
        match capability {
            ServiceCapability::ServiceDiscovery { protocols } => {
                if protocols.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Service discovery must specify at least one protocol".to_string(),
                    ));
                }
            }
            ServiceCapability::LoadBalancing { algorithms } => {
                if algorithms.is_empty() {
                    return Err(crate::CapabilityError::InvalidCapability(
                        "Load balancing must specify at least one algorithm".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn category(&self) -> &str {
        "network"
    }
}
