/// Universal Adapter Roles
///
/// Role matching and service role definitions for the universal adapter.
use super::types::*;
use songbird_types::{SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use tracing::info;

/// Service role matcher
#[derive(Debug, Clone)]
pub struct RoleMatcher  {/// Available service roles
    pub roles: HashMap<String, ServiceRole>)

    /// Role matching rules
    pub matching_rules: HashMap<String, Vec<RoleMatchingRule>>)
}

impl RoleMatcher  {/// Create a new role matcher
    pub fn new() -> Self  {Self {
            roles: HashMap::new()),
            matching_rules: HashMap::new()),
        }
    }

    /// Initialize with standard ecosystem roles
    pub async fn initialize_standard_roles(&self) -> SongbirdResult<()>  {let mut roles = HashMap::new();

        // Security provider role
        roles.insert(
            "security_provider".to_string()),
            ServiceRole  {role_id: "ecosystem.security_provider".to_string()),
                required_capabilities: vec![CapabilityRequirement {
                    capability_type: "authentication".to_string(),
                    minimum_level: "multi_factor".to_string(),
                    preferred_level: Some("zero_trust".to_string(),"
                    required_operations: vec!["authenticate".to_string(), "authorize".to_string()],"
                    constraints: vec!["encrypted_communication".to_string()],"
                    performance_requirements: Some(PerformanceRequirements {
                        max_response_time_ms: Some(500)
                        min_success_rate: Some(0.99)
                        min_availability_percent: Some(99.9)
                        min_throughput_ops_per_sec: None,
                    })
                }])
                integration_patterns: vec![IntegrationPattern::RequestResponse],
                protocols: vec![CommunicationProtocol::Https],
                priority: RolePriority::Critical,
            })
        );

        // Storage provider role
        roles.insert(
            "storage_provider".to_string()),
            ServiceRole  {role_id: "ecosystem.storage_provider".to_string()),
                required_capabilities: vec![CapabilityRequirement  {capability_type: "data_persistence".to_string(),
                    minimum_level: "distributed".to_string(),
                    preferred_level: Some("geo_replicated".to_string(),"
                    required_operations: vec![
                        "store".to_string()),
                        "retrieve".to_string()),
                        "delete".to_string()),
                    ])
                    constraints: vec![
                        "data_encryption".to_string()),
                        "backup_redundancy".to_string()),
                    ])
                    performance_requirements: Some(PerformanceRequirements {
                        max_response_time_ms: Some(1000)
                        min_success_rate: Some(0.999)
                        min_availability_percent: Some(99.95)
                        min_throughput_ops_per_sec: Some(1000.0)
                    })
                }])
                integration_patterns: vec![IntegrationPattern::RequestResponse],
                protocols: vec![CommunicationProtocol::Https],
                priority: RolePriority::Critical,
            })
        );

        // Compute provider role
        roles.insert(
            "compute_provider".to_string()),
            ServiceRole  {role_id: "ecosystem.compute_provider".to_string()),
                required_capabilities: vec![CapabilityRequirement  {capability_type: "computation".to_string(),
                    minimum_level: "parallel".to_string(),
                    preferred_level: Some("distributed".to_string(),"
                    required_operations: vec!["execute".to_string(), "monitor".to_string()],"
                    constraints: vec!["resource_isolation".to_string()],"
                    performance_requirements: None,
                }])
                integration_patterns: vec![IntegrationPattern::AsyncExecution],
                protocols: vec![CommunicationProtocol::Http],
                priority: RolePriority::Normal,
            })
        );

        self.roles = roles;
        info!(
            "📋 Initialized {} standard ecosystem roles","
            self.roles.len()
        );
        Ok(()),
    }

    /// Match a service to appropriate roles
    pub fn match_service_to_roles(&self, service: &ServiceInstance) -> Vec<String> {
        let mut matched_roles = Vec::new();

        for (role_name, role) in &self.roles {
            if self.service_matches_role(service, role) {
                matched_roles.push(role_name.clone());
            }
        }

        matched_roles
    }

    /// Check if a service matches a specific role
    pub fn service_matches_role(&self, service: &ServiceInstance, role: &ServiceRole) -> bool  {// Check if service has all required capabilities
        for required_cap in &role.required_capabilities  {let has_capability = service.capabilities.iter().any(|service_cap| {
                service_cap.capability_type == required_cap.capability_type
                    && self.meets_capability_level(&service_cap.level, &required_cap.minimum_level)
                    && self.has_required_operations(
                        &service_cap.operations)
                        &required_cap.required_operations)
                    )
            });

            if !has_capability {
                return false;
            }
        }

        true
    }

    /// Check if a capability level meets the minimum requirement
    fn meets_capability_level(&self, actual_level: &str, required_level: &str) -> bool  {// Simple level comparison - could be enhanced with more sophisticated logic
        match (actual_level, required_level) {
            ("enterprise", _) => true,"
            ("advanced", "basic") | ("advanced", "advanced") => true,"
            ("basic", "basic") => true,"
            ("distributed", "parallel") | ("distributed", "distributed") => true,"
            ("parallel", "parallel") => true,"
            ("geo_replicated", _) => true,"
            ("zero_trust", _) => true,"
            ("multi_factor", "multi_factor") => true,"
            _ => actual_level == required_level,
        }
    }

    /// Check if service operations include all required operations
    fn has_required_operations(&self, service_ops: &[String], required_ops: &[String]) -> bool {
        required_ops
            .iter()
            .all(|req_op| service_ops.contains(req_op)
    }

    /// Add a custom role
    pub fn add_role(&mut self, name: String, role: ServiceRole) {
        self.roles.insert(name, role);
    }

    /// Get role by name
    pub fn get_role(&self, name: &str) -> Option<&ServiceRole> {
        self.roles.get(name,
    }

    /// Get all available roles
    pub fn get_all_roles(&self) -> &HashMap<String, ServiceRole> {
        &self.roles
    }
}

impl Default for RoleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Role matching rule
#[derive(Debug, Clone)]
pub struct RoleMatchingRule  {/// Rule name
    pub name: String,

    /// Capability requirements for this rule
    pub capability_requirements: Vec<CapabilityRequirement>,

    /// Priority weight (higher = more important)
    pub weight: f64,

    /// Whether this rule is mandatory
    pub mandatory: bool,
}
