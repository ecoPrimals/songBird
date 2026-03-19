//! Role definitions

use serde::{Deserialize, Serialize};

/// User roles in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// No authentication
    Anonymous,

    /// Authenticated student
    Student {
        student_id: String,
        course_id: String,
    },

    /// Teaching assistant
    TA {
        ta_id: String,
        course_id: String,
    },

    /// Course instructor / Principal Investigator
    Professor {
        professor_id: String,
        courses: Vec<String>,
    },

    /// System administrator (local)
    Admin {
        admin_id: String,
    },

    /// Remote administrator (requires hardware key)
    RemoteAdmin {
        admin_id: String,
        vpn_session: String,
        hardware_key_verified: bool,
    },
}

impl Role {
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        matches!(self, Self::Admin { .. } | Self::RemoteAdmin { .. })
    }

    #[must_use]
    pub const fn is_teaching_staff(&self) -> bool {
        matches!(self, Self::TA { .. } | Self::Professor { .. })
    }

    #[must_use]
    pub const fn requires_hardware_key(&self) -> bool {
        matches!(self, Self::RemoteAdmin { .. })
    }
}

/// Resource quota for role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_concurrent_tasks: usize,
    pub max_gpu_hours_per_week: Option<f64>,
    pub max_storage_gb: Option<usize>,
}

impl ResourceQuota {
    #[must_use]
    pub const fn for_role(role: &Role) -> Self {
        match role {
            Role::Anonymous => Self {
                max_concurrent_tasks: 0,
                max_gpu_hours_per_week: None,
                max_storage_gb: None,
            },
            Role::Student {
                ..
            } => Self {
                max_concurrent_tasks: 3,
                max_gpu_hours_per_week: Some(10.0),
                max_storage_gb: Some(5),
            },
            Role::TA {
                ..
            } => Self {
                max_concurrent_tasks: 10,
                max_gpu_hours_per_week: Some(50.0),
                max_storage_gb: Some(20),
            },
            Role::Professor {
                ..
            } => Self {
                max_concurrent_tasks: 50,
                max_gpu_hours_per_week: Some(500.0),
                max_storage_gb: Some(100),
            },
            Role::Admin {
                ..
            }
            | Role::RemoteAdmin {
                ..
            } => Self {
                max_concurrent_tasks: 1000,
                max_gpu_hours_per_week: None, // Unlimited
                max_storage_gb: None,         // Unlimited
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_classification() {
        let student = Role::Student {
            student_id: "s123".into(),
            course_id: "CSE847".into(),
        };
        assert!(!student.is_admin());
        assert!(!student.is_teaching_staff());

        let ta = Role::TA {
            ta_id: "ta456".into(),
            course_id: "CSE847".into(),
        };
        assert!(!ta.is_admin());
        assert!(ta.is_teaching_staff());

        let admin = Role::Admin {
            admin_id: "admin789".into(),
        };
        assert!(admin.is_admin());
        assert!(!admin.is_teaching_staff());
    }

    #[test]
    fn test_resource_quotas() {
        let student_quota = ResourceQuota::for_role(&Role::Student {
            student_id: "s123".into(),
            course_id: "CSE847".into(),
        });
        assert_eq!(student_quota.max_concurrent_tasks, 3);
        assert_eq!(student_quota.max_gpu_hours_per_week, Some(10.0));

        let admin_quota = ResourceQuota::for_role(&Role::Admin {
            admin_id: "admin".into(),
        });
        assert!(admin_quota.max_gpu_hours_per_week.is_none()); // Unlimited
    }
}
