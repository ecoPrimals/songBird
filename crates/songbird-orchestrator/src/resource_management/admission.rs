//! Admission Control
//!
//! Decides whether new tasks can be admitted based on:
//! - Current system load
//! - User quotas
//! - Resource availability

use super::{QuotaManager, ResourceAmount, ResourceType};
use crate::task_lifecycle::TaskLifecycle;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Admission decision
#[derive(Debug, Clone)]
pub enum AdmissionDecision {
    Admitted,
    Rejected {
        reason: Arc<str>,
    },
    Delayed {
        estimated_wait_seconds: u64,
    },
}

/// System load tracking
#[derive(Debug, Clone)]
pub struct SystemLoad {
    cpu_usage: f64,    // 0.0 - 1.0
    memory_usage: f64, // 0.0 - 1.0
    active_tasks: usize,
}

/// Admission controller
pub struct AdmissionController {
    quota_manager: Arc<QuotaManager>,
    system_load: Arc<RwLock<SystemLoad>>,
    max_active_tasks: usize,
    cpu_threshold: f64,
    memory_threshold: f64,
}

impl AdmissionController {
    pub fn new(quota_manager: Arc<QuotaManager>) -> Self {
        Self {
            quota_manager,
            system_load: Arc::new(RwLock::new(SystemLoad {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                active_tasks: 0,
            })),
            max_active_tasks: 1000,
            cpu_threshold: 0.9,    // 90%
            memory_threshold: 0.9, // 90%
        }
    }

    /// Evaluate whether a task should be admitted
    pub async fn evaluate(&self, task: &TaskLifecycle) -> Result<AdmissionDecision> {
        // Check 1: System capacity
        let load = self.system_load.read().await;

        if load.active_tasks >= self.max_active_tasks {
            return Ok(AdmissionDecision::Rejected {
                reason: "System at maximum task capacity".into(),
            });
        }

        if load.cpu_usage >= self.cpu_threshold {
            return Ok(AdmissionDecision::Delayed {
                estimated_wait_seconds: 60, // Conservative estimate
            });
        }

        if load.memory_usage >= self.memory_threshold {
            return Ok(AdmissionDecision::Delayed {
                estimated_wait_seconds: 120,
            });
        }

        drop(load);

        // Check 2: User quota
        let requested_resources = self.extract_resources(task);
        let can_allocate =
            self.quota_manager.can_allocate(&task.owner, &requested_resources).await?;

        if !can_allocate {
            return Ok(AdmissionDecision::Rejected {
                reason: "User quota exceeded".into(),
            });
        }

        // Check 3: Resource availability (simplified)
        if !self.has_available_resources(&requested_resources).await {
            return Ok(AdmissionDecision::Delayed {
                estimated_wait_seconds: 30,
            });
        }

        Ok(AdmissionDecision::Admitted)
    }

    /// Admit a task (update tracking)
    pub async fn admit(&self, task: &TaskLifecycle) -> Result<()> {
        let mut load = self.system_load.write().await;
        load.active_tasks += 1;

        // Allocate from quota
        let requested = self.extract_resources(task);
        self.quota_manager.allocate(&task.owner, &requested).await?;

        Ok(())
    }

    /// Release a task (update tracking)
    pub async fn release(&self, task: &TaskLifecycle) -> Result<()> {
        let mut load = self.system_load.write().await;
        load.active_tasks = load.active_tasks.saturating_sub(1);

        // Release from quota
        let requested = self.extract_resources(task);
        self.quota_manager.release(&task.owner, &requested).await?;

        Ok(())
    }

    /// Update system load metrics
    pub async fn update_system_load(&self, cpu_usage: f64, memory_usage: f64) {
        let mut load = self.system_load.write().await;
        load.cpu_usage = cpu_usage.clamp(0.0, 1.0);
        load.memory_usage = memory_usage.clamp(0.0, 1.0);
    }

    /// Get current system load
    pub async fn get_system_load(&self) -> SystemLoad {
        self.system_load.read().await.clone()
    }

    /// Extract resource requirements from task
    fn extract_resources(&self, task: &TaskLifecycle) -> HashMap<ResourceType, ResourceAmount> {
        use super::ResourceUnit;

        let mut resources = HashMap::new();

        if let Some(cpu) = task.spec.resources.cpu_cores {
            resources
                .insert(ResourceType::Cpu, ResourceAmount::new(cpu as f64, ResourceUnit::Cores));
        }

        if let Some(memory) = task.spec.resources.memory_mb {
            resources.insert(
                ResourceType::Memory,
                ResourceAmount::new(memory as f64, ResourceUnit::Megabytes),
            );
        }

        if let Some(gpu) = task.spec.resources.gpu_count {
            resources
                .insert(ResourceType::Gpu, ResourceAmount::new(gpu as f64, ResourceUnit::Devices));
        }

        resources
    }

    /// Check if system has available resources (simplified)
    async fn has_available_resources(
        &self,
        _resources: &HashMap<ResourceType, ResourceAmount>,
    ) -> bool {
        // In a real implementation, would check actual system resources
        // For now, assume resources are available if we're under thresholds
        let load = self.system_load.read().await;
        load.cpu_usage < self.cpu_threshold && load.memory_usage < self.memory_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_lifecycle::types::{Priority, ResourceRequirements, TaskSpec};
    use crate::task_lifecycle::UserId;

    fn create_test_task(owner: &str, cpu: Option<u32>, memory: Option<u64>) -> TaskLifecycle {
        let spec = TaskSpec {
            task_type: "test".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements {
                cpu_cores: cpu,
                memory_mb: memory,
                gpu_count: None,
                network_mbps: Some(10),
                storage_gb: Some(1),
            },
            priority: Priority::Standard,
        };

        TaskLifecycle::new(UserId::from(owner), spec)
    }

    #[tokio::test]
    async fn test_admission_success() {
        let quota_mgr = Arc::new(QuotaManager::new());
        let controller = AdmissionController::new(quota_mgr);

        let task = create_test_task("alice", Some(2), Some(4096));

        let decision = controller.evaluate(&task).await.unwrap();
        assert!(matches!(decision, AdmissionDecision::Admitted));
    }

    #[tokio::test]
    async fn test_admission_quota_exceeded() {
        let quota_mgr = Arc::new(QuotaManager::new());
        let controller = AdmissionController::new(quota_mgr.clone());

        // Allocate most of user's quota
        let mut pre_allocate = HashMap::new();
        pre_allocate
            .insert(ResourceType::Cpu, ResourceAmount::new(7.0, super::super::ResourceUnit::Cores));
        quota_mgr.allocate(&UserId::from("alice"), &pre_allocate).await.unwrap();

        // Try to allocate more
        let task = create_test_task("alice", Some(2), Some(4096));

        let decision = controller.evaluate(&task).await.unwrap();
        assert!(matches!(decision, AdmissionDecision::Rejected { .. }));
    }

    #[tokio::test]
    async fn test_admission_system_overload() {
        let quota_mgr = Arc::new(QuotaManager::new());
        let controller = AdmissionController::new(quota_mgr);

        // Simulate high CPU usage
        controller.update_system_load(0.95, 0.5).await;

        let task = create_test_task("alice", Some(2), Some(4096));

        let decision = controller.evaluate(&task).await.unwrap();
        assert!(matches!(decision, AdmissionDecision::Delayed { .. }));
    }

    #[tokio::test]
    async fn test_admit_and_release() {
        let quota_mgr = Arc::new(QuotaManager::new());
        let controller = AdmissionController::new(quota_mgr);

        let task = create_test_task("alice", Some(2), Some(4096));

        // Initial state
        let load_before = controller.get_system_load().await;
        assert_eq!(load_before.active_tasks, 0);

        // Admit task
        controller.admit(&task).await.unwrap();

        let load_after_admit = controller.get_system_load().await;
        assert_eq!(load_after_admit.active_tasks, 1);

        // Release task
        controller.release(&task).await.unwrap();

        let load_after_release = controller.get_system_load().await;
        assert_eq!(load_after_release.active_tasks, 0);
    }
}
