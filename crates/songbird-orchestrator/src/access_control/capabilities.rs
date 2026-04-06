// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability definitions and checking

use serde::{Deserialize, Serialize};

/// Capabilities that can be granted to users
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Information Access (hierarchical)
    ViewPublicInfo,
    ViewEducationalInfo,
    ViewOperationalInfo,
    ViewAdministrativeInfo,
    ViewInfrastructureInfo,

    // Task Management
    SubmitTask,
    ViewOwnTasks,
    ViewAllStudentTasks,
    ViewAllTasks,
    CancelOwnTasks,
    CancelAnyTask,

    // User Management
    ManageCourseUsers,
    ManageAllUsers,
    ManageQuotas,

    // System Management
    ManageNodes,
    ManageConfiguration,
    RestartServices,
    AccessSystemLogs,

    // Data Access
    ViewStatistics,
    ExportData,
    AccessRawLogs,
    AccessSecurityLogs,
    AccessStudentLogs,

    // Admin wildcard
    All,
}

impl Capability {
    /// Check if this capability implies another
    ///
    /// The information hierarchy is:
    /// Infrastructure ⊃ Administrative ⊃ Operational ⊃ Educational ⊃ Public
    #[must_use]
    pub fn implies(&self, other: &Self) -> bool {
        use Capability::{
            AccessRawLogs, AccessSecurityLogs, AccessStudentLogs, AccessSystemLogs, All,
            CancelAnyTask, CancelOwnTasks, ManageAllUsers, ManageCourseUsers,
            ViewAdministrativeInfo, ViewAllStudentTasks, ViewAllTasks, ViewEducationalInfo,
            ViewInfrastructureInfo, ViewOperationalInfo, ViewOwnTasks, ViewPublicInfo,
        };

        // All implies everything
        if matches!(self, All) {
            return true;
        }

        // Exact match always implies
        if self == other {
            return true;
        }

        // Information hierarchy
        match (self, other) {
            // Infrastructure implies all lower layers
            (ViewInfrastructureInfo, ViewAdministrativeInfo) => true,
            (ViewInfrastructureInfo, ViewOperationalInfo) => true,
            (ViewInfrastructureInfo, ViewEducationalInfo) => true,
            (ViewInfrastructureInfo, ViewPublicInfo) => true,

            // Administrative implies operational, educational, public
            (ViewAdministrativeInfo, ViewOperationalInfo) => true,
            (ViewAdministrativeInfo, ViewEducationalInfo) => true,
            (ViewAdministrativeInfo, ViewPublicInfo) => true,

            // Operational implies educational, public
            (ViewOperationalInfo, ViewEducationalInfo) => true,
            (ViewOperationalInfo, ViewPublicInfo) => true,

            // Educational implies public
            (ViewEducationalInfo, ViewPublicInfo) => true,

            // Task management hierarchy
            (ViewAllTasks, ViewAllStudentTasks) => true,
            (ViewAllTasks, ViewOwnTasks) => true,
            (ViewAllStudentTasks, ViewOwnTasks) => true,

            (CancelAnyTask, CancelOwnTasks) => true,

            // User management hierarchy
            (ManageAllUsers, ManageCourseUsers) => true,

            // System access hierarchy
            (AccessSystemLogs, AccessSecurityLogs) => true,
            (AccessSystemLogs, AccessRawLogs) => true,
            (AccessSystemLogs, AccessStudentLogs) => true,

            // No implication
            _ => false,
        }
    }

    /// Check if this is a sensitive capability requiring additional authentication
    #[must_use]
    pub const fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::ViewInfrastructureInfo
                | Self::ManageConfiguration
                | Self::RestartServices
                | Self::ManageNodes
                | Self::AccessSecurityLogs
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_information_hierarchy() {
        let infra = Capability::ViewInfrastructureInfo;
        let admin = Capability::ViewAdministrativeInfo;
        let ops = Capability::ViewOperationalInfo;
        let edu = Capability::ViewEducationalInfo;
        let public = Capability::ViewPublicInfo;

        // Infrastructure implies everything
        assert!(infra.implies(&admin));
        assert!(infra.implies(&ops));
        assert!(infra.implies(&edu));
        assert!(infra.implies(&public));

        // Administrative implies lower layers
        assert!(admin.implies(&ops));
        assert!(admin.implies(&edu));
        assert!(admin.implies(&public));
        assert!(!admin.implies(&infra));

        // Operational implies lower layers
        assert!(ops.implies(&edu));
        assert!(ops.implies(&public));
        assert!(!ops.implies(&admin));
        assert!(!ops.implies(&infra));

        // Educational implies public only
        assert!(edu.implies(&public));
        assert!(!edu.implies(&ops));

        // Public implies nothing else
        assert!(!public.implies(&edu));
    }

    #[test]
    fn test_all_implies_everything() {
        let all = Capability::All;

        assert!(all.implies(&Capability::ViewPublicInfo));
        assert!(all.implies(&Capability::ViewInfrastructureInfo));
        assert!(all.implies(&Capability::SubmitTask));
        assert!(all.implies(&Capability::ManageConfiguration));
    }

    #[test]
    fn test_sensitive_capabilities() {
        assert!(Capability::ViewInfrastructureInfo.is_sensitive());
        assert!(Capability::ManageConfiguration.is_sensitive());
        assert!(Capability::RestartServices.is_sensitive());

        assert!(!Capability::ViewPublicInfo.is_sensitive());
        assert!(!Capability::ViewEducationalInfo.is_sensitive());
        assert!(!Capability::SubmitTask.is_sensitive());
    }
}
