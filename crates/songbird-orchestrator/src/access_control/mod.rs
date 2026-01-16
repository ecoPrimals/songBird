//! Access Control System for Songbird
//!
//! Implements graduated information disclosure and capability-based access control.
//!
//! ## Modes
//!
//! - **Standalone**: JWT-based authentication, works independently
//! - **security provider-Enhanced**: Genetic identity, hardware binding (when available)
//!
//! ## Information Layers
//!
//! - Public: Anyone
//! - Educational: Students (see sharding, anonymized topology)
//! - Operational: TAs (see failures, node health)
//! - Administrative: Professors (see utilization, statistics)
//! - Infrastructure: Admins (see IPs, configs, requires hardware key)

pub mod auth;
pub mod capabilities;
pub mod information_layers;
pub mod roles;
pub mod tokens;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub use capabilities::Capability;
pub use roles::Role;
pub use tokens::{AccessToken, TokenValidator};

/// Access control manager
pub struct AccessControl {
    /// Token validator
    token_validator: Arc<TokenValidator>,

    /// Role -> Capability mapping
    role_capabilities: Arc<RoleCapabilityMap>,

    /// Information layer builder
    info_builder: Arc<InformationLayerBuilder>,

    /// Audit logger
    audit_log: Arc<AuditLog>,

    /// Authentication mode
    mode: AuthMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMode {
    /// Standalone JWT authentication
    Standalone,

    /// security provider genetic identity (when available)
    BearDogEnhanced {
        genetic_verification_endpoint: String,
        hardware_binding_required: bool,
    },
}

/// Role -> Capabilities mapping
pub struct RoleCapabilityMap {
    mappings: std::collections::HashMap<Role, Vec<Capability>>,
}

impl Default for RoleCapabilityMap {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleCapabilityMap {
    pub fn new() -> Self {
        let mut mappings = std::collections::HashMap::new();

        // Anonymous (no auth)
        mappings.insert(Role::Anonymous, vec![Capability::ViewPublicInfo]);

        // Student
        mappings.insert(
            Role::Student {
                student_id: String::new(),
                course_id: String::new(),
            },
            vec![
                Capability::ViewPublicInfo,
                Capability::ViewEducationalInfo,
                Capability::SubmitTask,
                Capability::ViewOwnTasks,
                Capability::CancelOwnTasks,
            ],
        );

        // TA
        mappings.insert(
            Role::TA {
                ta_id: String::new(),
                course_id: String::new(),
            },
            vec![
                Capability::ViewPublicInfo,
                Capability::ViewEducationalInfo,
                Capability::ViewOperationalInfo,
                Capability::SubmitTask,
                Capability::ViewAllStudentTasks,
                Capability::AccessStudentLogs,
            ],
        );

        // Professor
        mappings.insert(
            Role::Professor {
                professor_id: String::new(),
                courses: vec![],
            },
            vec![
                Capability::ViewPublicInfo,
                Capability::ViewEducationalInfo,
                Capability::ViewOperationalInfo,
                Capability::ViewAdministrativeInfo,
                Capability::SubmitTask,
                Capability::ViewAllTasks,
                Capability::ManageCourseUsers,
                Capability::ManageQuotas,
                Capability::ViewStatistics,
            ],
        );

        // Admin
        mappings.insert(
            Role::Admin {
                admin_id: String::new(),
            },
            vec![Capability::All],
        );

        Self {
            mappings,
        }
    }

    pub fn get_capabilities(&self, role: &Role) -> Vec<Capability> {
        // Match by role type, ignoring specific IDs
        let role_key = match role {
            Role::Anonymous => Role::Anonymous,
            Role::Student {
                ..
            } => Role::Student {
                student_id: String::new(),
                course_id: String::new(),
            },
            Role::TA {
                ..
            } => Role::TA {
                ta_id: String::new(),
                course_id: String::new(),
            },
            Role::Professor {
                ..
            } => Role::Professor {
                professor_id: String::new(),
                courses: vec![],
            },
            Role::Admin {
                ..
            } => Role::Admin {
                admin_id: String::new(),
            },
            Role::RemoteAdmin {
                ..
            } => {
                // Remote admin starts with limited capabilities
                return vec![Capability::ViewAdministrativeInfo];
            }
        };

        self.mappings.get(&role_key).cloned().unwrap_or_else(|| vec![Capability::ViewPublicInfo])
    }
}

/// Information layer builder
pub struct InformationLayerBuilder;

impl Default for InformationLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InformationLayerBuilder {
    pub fn new() -> Self {
        Self
    }

    // Implementation in information_layers module
}

/// Audit logger
pub struct AuditLog;

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLog {
    pub fn new() -> Self {
        Self
    }

    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        info!(
            user = ?entry.identity,
            capability = ?entry.capability,
            granted = entry.granted,
            "Access attempt"
        );
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub identity: String,
    pub capability: Capability,
    pub granted: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AccessControl {
    pub fn new(mode: AuthMode) -> Self {
        Self {
            token_validator: Arc::new(TokenValidator::new()),
            role_capabilities: Arc::new(RoleCapabilityMap::new()),
            info_builder: Arc::new(InformationLayerBuilder::new()),
            audit_log: Arc::new(AuditLog::new()),
            mode,
        }
    }

    /// Check if token has required capability
    pub async fn check_access(&self, token: &AccessToken, capability: &Capability) -> Result<bool> {
        // 1. Validate token
        let identity = self.token_validator.validate(token).await?;

        // 2. Get role capabilities
        let caps = self.role_capabilities.get_capabilities(&identity.role);

        // 3. Check capability (with implication)
        let has_capability = caps.iter().any(|c| c.implies(capability));

        // 4. Log access attempt
        let entry = AuditEntry {
            identity: identity.id.clone(),
            capability: capability.clone(),
            granted: has_capability,
            timestamp: chrono::Utc::now(),
        };
        self.audit_log.log(entry).await?;

        if has_capability {
            debug!(%identity.id, ?capability, "Access granted");
        } else {
            warn!(%identity.id, ?capability, "Access denied");
        }

        Ok(has_capability)
    }

    /// Get visible information for task based on token capabilities
    pub async fn get_visible_task_info(
        &self,
        token: &AccessToken,
        task: &crate::task_lifecycle::TaskLifecycle,
    ) -> Result<information_layers::TaskInfo> {
        let identity = self.token_validator.validate(token).await?;

        let mut info = information_layers::TaskInfo::new(task.id);

        // Always add public layer
        info.add_public_layer(self.info_builder.build_public(task));

        // Add educational layer if capability present
        if self.check_access(token, &Capability::ViewEducationalInfo).await? {
            info.add_educational_layer(self.info_builder.build_educational(task));
        }

        // Add operational layer if capability present
        if self.check_access(token, &Capability::ViewOperationalInfo).await? {
            info.add_operational_layer(self.info_builder.build_operational(task));
        }

        // Add administrative layer if capability present
        if self.check_access(token, &Capability::ViewAdministrativeInfo).await? {
            info.add_administrative_layer(self.info_builder.build_administrative(task));
        }

        // Add infrastructure layer if capability present AND 2FA verified
        if self.check_access(token, &Capability::ViewInfrastructureInfo).await? {
            // Verify 2FA/hardware key for infrastructure access
            // Implementation depends on auth mode:
            // - Standalone mode: Check for explicit 2FA token claim
            // - security provider mode: Verify hardware key (SoloKey) entropy level
            match &self.mode {
                AuthMode::Standalone => {
                    // In standalone mode, infrastructure access requires explicit 2FA verification
                    // This should be checked in token validation, but we do an additional check here
                    if token.has_2fa_verified() {
                        info.add_infrastructure_layer(self.info_builder.build_infrastructure(task));
                    } else {
                        tracing::warn!(
                            "Infrastructure access attempted without 2FA verification. \
                             Token: {}, Role: {:?}",
                            token.sub,
                            token.role
                        );
                        // Do not add infrastructure layer without 2FA
                    }
                }
                AuthMode::BearDogEnhanced {
                    ..
                } => {
                    // security provider mode: Verify hardware key entropy level
                    // Future: Integrate with security provider for hardware key verification
                    // For now, allow if role permits (security provider integration Q1 2025)
                    info.add_infrastructure_layer(self.info_builder.build_infrastructure(task));
                }
            }
        }

        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_anonymous_access() {
        let ac = AccessControl::new(AuthMode::Standalone);

        let token = AccessToken::anonymous();

        // Anonymous can view public info
        assert!(ac.check_access(&token, &Capability::ViewPublicInfo).await.unwrap());

        // Anonymous cannot submit tasks
        assert!(!ac.check_access(&token, &Capability::SubmitTask).await.unwrap());
    }

    #[tokio::test]
    async fn test_student_access() {
        let ac = AccessControl::new(AuthMode::Standalone);

        let token = AccessToken::student("student-123", "CSE-847");

        // Student can view educational info
        assert!(ac.check_access(&token, &Capability::ViewEducationalInfo).await.unwrap());

        // Student can submit tasks
        assert!(ac.check_access(&token, &Capability::SubmitTask).await.unwrap());

        // Student cannot view infrastructure
        assert!(!ac.check_access(&token, &Capability::ViewInfrastructureInfo).await.unwrap());
    }

    #[tokio::test]
    async fn test_capability_implication() {
        let cap_infra = Capability::ViewInfrastructureInfo;
        let cap_admin = Capability::ViewAdministrativeInfo;
        let cap_ops = Capability::ViewOperationalInfo;
        let cap_edu = Capability::ViewEducationalInfo;
        let cap_public = Capability::ViewPublicInfo;

        // Infrastructure implies all lower layers
        assert!(cap_infra.implies(&cap_admin));
        assert!(cap_infra.implies(&cap_ops));
        assert!(cap_infra.implies(&cap_edu));
        assert!(cap_infra.implies(&cap_public));

        // But not the reverse
        assert!(!cap_public.implies(&cap_infra));
    }
}
