// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Resource Quota System
//!
//! Manages per-user resource quotas with:
//! - No unsafe code
//! - Async operations
//! - Fair enforcement

use super::{ResourceAmount, ResourceType, ResourceUnit};
use crate::task_lifecycle::UserId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// User resource quota
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub user_id: UserId,
    pub limits: HashMap<ResourceType, ResourceAmount>,
    pub used: HashMap<ResourceType, ResourceAmount>,
}

impl ResourceQuota {
    /// Create a new quota for a user
    #[must_use]
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            limits: Self::default_limits(),
            used: Self::zero_usage(),
        }
    }

    /// Default resource limits
    fn default_limits() -> HashMap<ResourceType, ResourceAmount> {
        let mut limits = HashMap::new();
        limits.insert(ResourceType::Cpu, ResourceAmount::new(8.0, ResourceUnit::Cores));
        limits.insert(ResourceType::Memory, ResourceAmount::new(16384.0, ResourceUnit::Megabytes));
        limits.insert(ResourceType::Gpu, ResourceAmount::new(2.0, ResourceUnit::Devices));
        limits.insert(ResourceType::Network, ResourceAmount::new(1000.0, ResourceUnit::Mbps));
        limits
            .insert(ResourceType::Storage, ResourceAmount::new(100000.0, ResourceUnit::Megabytes));
        limits
    }

    /// Zero usage for all resource types
    fn zero_usage() -> HashMap<ResourceType, ResourceAmount> {
        let mut used = HashMap::new();
        used.insert(ResourceType::Cpu, ResourceAmount::zero(ResourceUnit::Cores));
        used.insert(ResourceType::Memory, ResourceAmount::zero(ResourceUnit::Megabytes));
        used.insert(ResourceType::Gpu, ResourceAmount::zero(ResourceUnit::Devices));
        used.insert(ResourceType::Network, ResourceAmount::zero(ResourceUnit::Mbps));
        used.insert(ResourceType::Storage, ResourceAmount::zero(ResourceUnit::Megabytes));
        used
    }

    /// Check if requested resources are within quota
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn can_allocate(&self, requested: &HashMap<ResourceType, ResourceAmount>) -> Result<bool> {
        for (resource_type, amount) in requested {
            let limit = self
                .limits
                .get(resource_type)
                .ok_or_else(|| anyhow::anyhow!("No limit set for {resource_type:?}"))?;

            let used = self
                .used
                .get(resource_type)
                .ok_or_else(|| anyhow::anyhow!("No usage tracking for {resource_type:?}"))?;

            let new_usage = used.add(amount)?;

            if !new_usage.le(limit) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Allocate resources (increases usage)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn allocate(&mut self, resources: &HashMap<ResourceType, ResourceAmount>) -> Result<()> {
        if !self.can_allocate(resources)? {
            anyhow::bail!("Resource allocation would exceed quota");
        }

        for (resource_type, amount) in resources {
            let used = self
                .used
                .get_mut(resource_type)
                .ok_or_else(|| anyhow::anyhow!("No usage tracking for {resource_type:?}"))?;
            *used = used.add(amount)?;
        }

        Ok(())
    }

    /// Release resources (decreases usage)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn release(&mut self, resources: &HashMap<ResourceType, ResourceAmount>) -> Result<()> {
        for (resource_type, amount) in resources {
            if let Some(used) = self.used.get_mut(resource_type) {
                *used = used.sub(amount)?;
            }
        }

        Ok(())
    }

    /// Get available resources
    #[must_use]
    pub fn available(&self) -> HashMap<ResourceType, ResourceAmount> {
        let mut available = HashMap::new();

        for (resource_type, limit) in &self.limits {
            if let Some(used) = self.used.get(resource_type)
                && let Ok(avail) = limit.sub(used)
            {
                available.insert(*resource_type, avail);
            }
        }

        available
    }
}

/// Quota manager
pub struct QuotaManager {
    quotas: Arc<RwLock<HashMap<UserId, ResourceQuota>>>,
}

impl QuotaManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            quotas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create quota for a user
    pub async fn get_quota(&self, user_id: &UserId) -> ResourceQuota {
        let quotas = self.quotas.read().await;

        if let Some(quota) = quotas.get(user_id) {
            quota.clone()
        } else {
            drop(quotas);
            let mut quotas = self.quotas.write().await;
            let quota = ResourceQuota::new(user_id.clone());
            quotas.insert(user_id.clone(), quota.clone());
            quota
        }
    }

    /// Update quota for a user
    pub async fn update_quota(&self, quota: ResourceQuota) {
        let mut quotas = self.quotas.write().await;
        quotas.insert(quota.user_id.clone(), quota);
    }

    /// Check if user can allocate resources
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn can_allocate(
        &self,
        user_id: &UserId,
        resources: &HashMap<ResourceType, ResourceAmount>,
    ) -> Result<bool> {
        let quota = self.get_quota(user_id).await;
        quota.can_allocate(resources)
    }

    /// Allocate resources for a user
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn allocate(
        &self,
        user_id: &UserId,
        resources: &HashMap<ResourceType, ResourceAmount>,
    ) -> Result<()> {
        let mut quota = self.get_quota(user_id).await;
        quota.allocate(resources)?;
        self.update_quota(quota).await;
        Ok(())
    }

    /// Release resources for a user
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn release(
        &self,
        user_id: &UserId,
        resources: &HashMap<ResourceType, ResourceAmount>,
    ) -> Result<()> {
        let mut quota = self.get_quota(user_id).await;
        quota.release(resources)?;
        self.update_quota(quota).await;
        Ok(())
    }
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_quota_allocation() {
        let mut quota = ResourceQuota::new(UserId::from("test-user"));

        let mut request = HashMap::new();
        request.insert(ResourceType::Cpu, ResourceAmount::new(2.0, ResourceUnit::Cores));
        request.insert(ResourceType::Memory, ResourceAmount::new(4096.0, ResourceUnit::Megabytes));

        // Should be able to allocate
        assert!(quota.can_allocate(&request).unwrap());

        // Allocate
        quota.allocate(&request).unwrap();

        // Usage should be updated
        assert_eq!(quota.used.get(&ResourceType::Cpu).unwrap().value, 2.0);
        assert_eq!(quota.used.get(&ResourceType::Memory).unwrap().value, 4096.0);

        // Release
        quota.release(&request).unwrap();

        // Usage should be back to zero
        assert_eq!(quota.used.get(&ResourceType::Cpu).unwrap().value, 0.0);
    }

    #[test]
    fn test_quota_exceeds_limit() {
        let mut quota = ResourceQuota::new(UserId::from("test-user"));

        let mut request = HashMap::new();
        request.insert(
            ResourceType::Cpu,
            ResourceAmount::new(100.0, ResourceUnit::Cores), // Exceeds default limit of 8
        );

        // Should not be able to allocate
        assert!(!quota.can_allocate(&request).unwrap());

        // Allocate should fail
        assert!(quota.allocate(&request).is_err());
    }

    #[tokio::test]
    async fn test_quota_manager() {
        let manager = QuotaManager::new();
        let user_id = UserId::from("test-user");

        let mut request = HashMap::new();
        request.insert(ResourceType::Cpu, ResourceAmount::new(2.0, ResourceUnit::Cores));

        // Should be able to allocate
        assert!(manager.can_allocate(&user_id, &request).await.unwrap());

        // Allocate
        manager.allocate(&user_id, &request).await.unwrap();

        // Verify usage increased
        let quota = manager.get_quota(&user_id).await;
        assert_eq!(quota.used.get(&ResourceType::Cpu).unwrap().value, 2.0);

        // Release
        manager.release(&user_id, &request).await.unwrap();

        // Verify usage decreased
        let quota = manager.get_quota(&user_id).await;
        assert_eq!(quota.used.get(&ResourceType::Cpu).unwrap().value, 0.0);
    }

    #[test]
    fn empty_request_can_allocate() {
        let quota = ResourceQuota::new(UserId::from("u"));
        assert!(quota.can_allocate(&HashMap::new()).unwrap());
    }

    #[test]
    fn available_reflects_limits_minus_used() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        let mut req = HashMap::new();
        req.insert(ResourceType::Cpu, ResourceAmount::new(2.0, ResourceUnit::Cores));
        quota.allocate(&req).unwrap();
        let avail = quota.available();
        let cpu = avail.get(&ResourceType::Cpu).unwrap();
        assert_eq!(cpu.value, 6.0);
    }

    #[test]
    fn cumulative_cpu_at_limit_rejects_extra() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        let mut fill = HashMap::new();
        fill.insert(ResourceType::Cpu, ResourceAmount::new(8.0, ResourceUnit::Cores));
        quota.allocate(&fill).unwrap();
        let mut more = HashMap::new();
        more.insert(ResourceType::Cpu, ResourceAmount::new(0.1, ResourceUnit::Cores));
        assert!(!quota.can_allocate(&more).unwrap());
    }

    #[tokio::test]
    async fn get_quota_creates_and_reuses() {
        let m = QuotaManager::new();
        let u = UserId::from("same");
        let q1 = m.get_quota(&u).await;
        let q2 = m.get_quota(&u).await;
        assert_eq!(q1.user_id, q2.user_id);
    }

    #[tokio::test]
    async fn quota_manager_default_matches_new() {
        let a = QuotaManager::new();
        let b = QuotaManager::default();
        assert_eq!(a.quotas.read().await.len(), 0);
        assert_eq!(b.quotas.read().await.len(), 0);
    }

    #[test]
    fn can_allocate_errors_when_limit_missing_for_requested_type() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        quota.limits.remove(&ResourceType::Network);
        let mut req = HashMap::new();
        req.insert(ResourceType::Network, ResourceAmount::new(1.0, ResourceUnit::Mbps));
        let err = quota.can_allocate(&req).unwrap_err();
        assert!(err.to_string().contains("No limit set"));
    }

    #[test]
    fn can_allocate_errors_when_usage_missing_for_requested_type() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        quota.used.remove(&ResourceType::Storage);
        let mut req = HashMap::new();
        req.insert(ResourceType::Storage, ResourceAmount::new(1.0, ResourceUnit::Megabytes));
        let err = quota.can_allocate(&req).unwrap_err();
        assert!(err.to_string().contains("No usage tracking"));
    }

    #[test]
    fn allocate_errors_when_usage_tracking_missing() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        quota.used.remove(&ResourceType::Cpu);
        let mut req = HashMap::new();
        req.insert(ResourceType::Cpu, ResourceAmount::new(1.0, ResourceUnit::Cores));
        assert!(quota.allocate(&req).is_err());
    }

    #[test]
    fn release_ignores_unknown_resource_types() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        let mut rel = HashMap::new();
        rel.insert(ResourceType::Cpu, ResourceAmount::new(1.0, ResourceUnit::Cores));
        rel.insert(ResourceType::Network, ResourceAmount::new(999.0, ResourceUnit::Mbps));
        quota.release(&rel).unwrap();
        assert_eq!(quota.used.get(&ResourceType::Cpu).unwrap().value, 0.0);
    }

    #[test]
    fn allocate_errors_on_unit_mismatch_in_request() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        let mut bad = HashMap::new();
        bad.insert(ResourceType::Cpu, ResourceAmount::new(1.0, ResourceUnit::Megabytes));
        assert!(quota.allocate(&bad).is_err());
    }

    #[test]
    fn available_reports_zero_when_used_exceeds_limit() {
        let mut quota = ResourceQuota::new(UserId::from("u"));
        quota.limits.insert(ResourceType::Cpu, ResourceAmount::new(1.0, ResourceUnit::Cores));
        quota.used.insert(ResourceType::Cpu, ResourceAmount::new(2.0, ResourceUnit::Cores));
        let avail = quota.available();
        let cpu = avail.get(&ResourceType::Cpu).unwrap();
        assert_eq!(cpu.value, 0.0);
    }

    #[tokio::test]
    async fn update_quota_persists_custom_limits() {
        let m = QuotaManager::new();
        let u = UserId::from("custom");
        let mut q = m.get_quota(&u).await;
        q.limits.insert(ResourceType::Cpu, ResourceAmount::new(1.0, ResourceUnit::Cores));
        m.update_quota(q).await;
        let q2 = m.get_quota(&u).await;
        assert_eq!(q2.limits.get(&ResourceType::Cpu).unwrap().value, 1.0);
    }

    #[tokio::test]
    async fn allocate_then_can_allocate_false_until_release() {
        let m = QuotaManager::new();
        let u = UserId::from("alice");
        let mut req = HashMap::new();
        req.insert(ResourceType::Cpu, ResourceAmount::new(8.0, ResourceUnit::Cores));
        m.allocate(&u, &req).await.unwrap();
        let mut more = HashMap::new();
        more.insert(ResourceType::Cpu, ResourceAmount::new(0.1, ResourceUnit::Cores));
        assert!(!m.can_allocate(&u, &more).await.unwrap());
        m.release(&u, &req).await.unwrap();
        assert!(m.can_allocate(&u, &more).await.unwrap());
    }
}
