//! Modernization utilities for migrating deprecated PrimalCapability variants
//!
//! This module provides utilities to systematically migrate from deprecated
//! PrimalCapability variants to their canonical modern equivalents.
//!
//! ## 🚀 CANONICAL MODERNIZATION UTILITIES
//!
//! These utilities enable automatic migration of deprecated capability variants
//! to their canonical modern equivalents, eliminating fragmentation.

use crate::traits::PrimalCapability;

/// Migrate deprecated PrimalCapability variants to canonical equivalents
#[allow(deprecated)] // This function is designed to handle deprecated variants
pub fn modernize_capability(deprecated: PrimalCapability) -> PrimalCapability {
    match deprecated {
        // Storage migrations
        PrimalCapability::FileSystem { supports_zfs } => {
            let mut types = vec!["file".to_string()];
            if supports_zfs {
                types.push("zfs".to_string());
            }
            PrimalCapability::Storage { types }
        }
        PrimalCapability::ObjectStorage { backends } => PrimalCapability::Storage {
            types: vec!["object".to_string()]
                .into_iter()
                .chain(backends)
                .collect(),
        },
        PrimalCapability::DataReplication { consistency } => PrimalCapability::Storage {
            types: vec!["replication".to_string()]
                .into_iter()
                .chain(consistency)
                .collect(),
        },
        PrimalCapability::Backup { incremental } => {
            let mut types = vec!["backup".to_string()];
            if incremental {
                types.push("incremental".to_string());
            }
            PrimalCapability::Storage { types }
        }
        PrimalCapability::DataArchiving { compression } => PrimalCapability::Storage {
            types: vec!["archive".to_string()]
                .into_iter()
                .chain(compression)
                .collect(),
        },
        PrimalCapability::BackupRestore { incremental } => {
            let mut types = vec!["backup".to_string(), "restore".to_string()];
            if incremental {
                types.push("incremental".to_string());
            }
            PrimalCapability::Storage { types }
        }

        // Compute migrations
        PrimalCapability::ContainerRuntime { orchestrators } => PrimalCapability::Compute {
            types: vec!["container".to_string()]
                .into_iter()
                .chain(orchestrators)
                .collect(),
        },
        PrimalCapability::ServerlessExecution { languages } => PrimalCapability::Compute {
            types: vec!["serverless".to_string()]
                .into_iter()
                .chain(languages)
                .collect(),
        },
        PrimalCapability::GpuAcceleration { cuda_support } => {
            let mut types = vec!["gpu".to_string()];
            if cuda_support {
                types.push("cuda".to_string());
            }
            PrimalCapability::Compute { types }
        }

        // AI migrations
        PrimalCapability::ModelInference { models } => PrimalCapability::AI { models },
        PrimalCapability::AgentFramework { mcp_support } => {
            let mut models = vec!["agent".to_string()];
            if mcp_support {
                models.push("mcp".to_string());
            }
            PrimalCapability::AI { models }
        }
        PrimalCapability::NaturalLanguage { languages } => PrimalCapability::AI {
            models: vec!["nlp".to_string()]
                .into_iter()
                .chain(languages)
                .collect(),
        },
        PrimalCapability::MachineLearning { training_support } => {
            let mut models = vec!["ml".to_string()];
            if training_support {
                models.push("training".to_string());
            }
            PrimalCapability::AI { models }
        }
        PrimalCapability::ComputerVision { models } => PrimalCapability::AI {
            models: vec!["vision".to_string()]
                .into_iter()
                .chain(models)
                .collect(),
        },

        // Orchestration migrations
        PrimalCapability::LoadBalancing { algorithms } => PrimalCapability::Orchestration {
            features: vec!["load_balancing".to_string()]
                .into_iter()
                .chain(algorithms)
                .collect(),
        },
        PrimalCapability::AutoScaling { metrics } => PrimalCapability::Orchestration {
            features: vec!["auto_scaling".to_string()]
                .into_iter()
                .chain(metrics)
                .collect(),
        },
        PrimalCapability::Manifests { formats } => PrimalCapability::Orchestration {
            features: vec!["manifests".to_string()]
                .into_iter()
                .chain(formats)
                .collect(),
        },

        // Security migrations
        PrimalCapability::KeyManagement { hsm_support } => {
            let mut protocols = vec!["key_management".to_string()];
            if hsm_support {
                protocols.push("hsm".to_string());
            }
            PrimalCapability::Security { protocols }
        }
        PrimalCapability::ThreatDetection { ml_enabled } => {
            let mut protocols = vec!["threat_detection".to_string()];
            if ml_enabled {
                protocols.push("ml_detection".to_string());
            }
            PrimalCapability::Security { protocols }
        }
        PrimalCapability::Authorization { rbac_support } => {
            let mut methods = vec!["authorization".to_string()];
            if rbac_support {
                methods.push("rbac".to_string());
            }
            PrimalCapability::Authentication { methods }
        }

        // Networking migrations
        PrimalCapability::NetworkRouting { protocols } => PrimalCapability::Networking {
            protocols: vec!["routing".to_string()]
                .into_iter()
                .chain(protocols)
                .collect(),
        },
        PrimalCapability::ProxyServices { types } => PrimalCapability::Networking {
            protocols: vec!["proxy".to_string()].into_iter().chain(types).collect(),
        },
        PrimalCapability::VpnServices { protocols } => PrimalCapability::Networking {
            protocols: vec!["vpn".to_string()]
                .into_iter()
                .chain(protocols)
                .collect(),
        },

        // Already modern variants - pass through unchanged
        PrimalCapability::Storage { types } => PrimalCapability::Storage { types },
        PrimalCapability::Compute { types } => PrimalCapability::Compute { types },
        PrimalCapability::AI { models } => PrimalCapability::AI { models },
        PrimalCapability::Authentication { methods } => {
            PrimalCapability::Authentication { methods }
        }
        PrimalCapability::Networking { protocols } => PrimalCapability::Networking { protocols },
        PrimalCapability::ServiceDiscovery { protocols } => {
            PrimalCapability::ServiceDiscovery { protocols }
        }
        PrimalCapability::Security { protocols } => PrimalCapability::Security { protocols },
        PrimalCapability::Encryption { algorithms } => PrimalCapability::Encryption { algorithms },
        PrimalCapability::Orchestration { features } => {
            PrimalCapability::Orchestration { features }
        }
        PrimalCapability::Database { types } => PrimalCapability::Database { types },
        PrimalCapability::Messaging { protocols } => PrimalCapability::Messaging { protocols },
        PrimalCapability::Custom { name, properties } => {
            PrimalCapability::Custom { name, properties }
        }
    }
}

/// Batch modernize multiple capabilities
pub fn modernize_capabilities(deprecated_caps: Vec<PrimalCapability>) -> Vec<PrimalCapability> {
    deprecated_caps
        .into_iter()
        .map(modernize_capability)
        .collect()
}

/// Check if a capability is deprecated and needs modernization
#[allow(deprecated)]
pub fn is_deprecated_capability(cap: &PrimalCapability) -> bool {
    matches!(
        cap,
        PrimalCapability::FileSystem { .. }
            | PrimalCapability::ContainerRuntime { .. }
            | PrimalCapability::ServerlessExecution { .. }
            | PrimalCapability::ModelInference { .. }
            | PrimalCapability::AgentFramework { .. }
            | PrimalCapability::NaturalLanguage { .. }
            | PrimalCapability::ObjectStorage { .. }
            | PrimalCapability::LoadBalancing { .. }
            | PrimalCapability::AutoScaling { .. }
            | PrimalCapability::DataReplication { .. }
            | PrimalCapability::Backup { .. }
            | PrimalCapability::DataArchiving { .. }
            | PrimalCapability::KeyManagement { .. }
            | PrimalCapability::ThreatDetection { .. }
            | PrimalCapability::Authorization { .. }
            | PrimalCapability::GpuAcceleration { .. }
            | PrimalCapability::MachineLearning { .. }
            | PrimalCapability::ComputerVision { .. }
            | PrimalCapability::NetworkRouting { .. }
            | PrimalCapability::ProxyServices { .. }
            | PrimalCapability::VpnServices { .. }
            | PrimalCapability::Manifests { .. }
            | PrimalCapability::BackupRestore { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)] // Test function needs to use deprecated variants
    fn test_file_system_migration() {
        let deprecated = PrimalCapability::FileSystem { supports_zfs: true };
        let modern = modernize_capability(deprecated);

        match modern {
            PrimalCapability::Storage { types } => {
                assert!(types.contains(&"file".to_string()));
                assert!(types.contains(&"zfs".to_string()));
            }
            _ => panic!("Expected Storage capability"),
        }
    }

    #[test]
    #[allow(deprecated)] // Test function needs to use deprecated variants
    fn test_container_runtime_migration() {
        let deprecated = PrimalCapability::ContainerRuntime {
            orchestrators: vec!["kubernetes".to_string(), "docker".to_string()],
        };
        let modern = modernize_capability(deprecated);

        match modern {
            PrimalCapability::Compute { types } => {
                assert!(types.contains(&"container".to_string()));
                assert!(types.contains(&"kubernetes".to_string()));
                assert!(types.contains(&"docker".to_string()));
            }
            _ => panic!("Expected Compute capability"),
        }
    }

    #[test]
    #[allow(deprecated)] // Test function needs to use deprecated variants
    fn test_model_inference_migration() {
        let deprecated = PrimalCapability::ModelInference {
            models: vec!["gpt".to_string(), "bert".to_string()],
        };
        let modern = modernize_capability(deprecated);

        match modern {
            PrimalCapability::AI { models } => {
                assert!(models.contains(&"gpt".to_string()));
                assert!(models.contains(&"bert".to_string()));
            }
            _ => panic!("Expected AI capability"),
        }
    }

    #[test]
    #[allow(deprecated)] // Test function needs to use deprecated variants
    fn test_batch_modernization() {
        let deprecated_caps = vec![
            PrimalCapability::FileSystem {
                supports_zfs: false,
            },
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["k8s".to_string()],
            },
            PrimalCapability::ModelInference {
                models: vec!["llama".to_string()],
            },
        ];

        let modern_caps = modernize_capabilities(deprecated_caps);
        assert_eq!(modern_caps.len(), 3);

        // All should be modern variants
        for cap in modern_caps {
            match cap {
                PrimalCapability::Storage { .. }
                | PrimalCapability::Compute { .. }
                | PrimalCapability::AI { .. } => {
                    // Good - these are modern
                }
                _ => panic!("Expected modern capability variant"),
            }
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_is_deprecated_capability() {
        let modern = PrimalCapability::Storage {
            types: vec!["file".to_string()],
        };
        let deprecated = PrimalCapability::FileSystem {
            supports_zfs: false,
        };

        assert!(!is_deprecated_capability(&modern));
        assert!(is_deprecated_capability(&deprecated));
    }
}
