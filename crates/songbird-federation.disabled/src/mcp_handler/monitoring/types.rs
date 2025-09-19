//! # 🎼 Monitoring Types - Shared Data Structures
//!
//! **🚀 MODULAR ARCHITECTURE**
//!
//! Shared types and data structures for federation monitoring components.
//! This module contains the core types used across monitoring modules.

// use songbird_config::canonical::  // TEMPORARILY DISABLED - no canonical moduleUniversalHealthStatus;

/// System metrics structure for federation monitoring
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total_gb: u64,
    pub storage_available_gb: u64,
    pub storage_total_gb: u64,
    pub uptime_seconds: u64,
    pub load_average: f64,
    pub service_count: u32,
    pub active_connections: u32,
    pub capacity: f64,
    pub gaming_enabled: bool,
    pub primal_services_enabled: bool,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_total_gb: 0,
            storage_available_gb: 0,
            storage_total_gb: 0,
            uptime_seconds: 0,
            load_average: 0.0,
            service_count: 0,
            active_connections: 0,
            capacity: 0.0,
            gaming_enabled: false,
            primal_services_enabled: false,
        }
    }
}

/// Health status for federation monitoring
#[derive(Debug, Clone, PartialEq)]
pub enum Health {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

impl Default for Health {
    fn default() -> Self {
        Self::Unknown
    }
}

/// System health status for federation monitoring
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub health: Health,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub metrics: SystemMetrics,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            health: Health::Healthy,
            issues: Vec::new(),
            warnings: Vec::new(),
            metrics: SystemMetrics::default(),
        }
    }
}

impl HealthStatus {
    /// Create a new healthy status
    pub fn healthy() -> Self {
        Self {
            health: Health::Healthy,
            issues: Vec::new(),
            warnings: Vec::new(),
            metrics: SystemMetrics::default(),
        }
    }

    /// Create a degraded status with warnings
    pub fn degraded(warnings: Vec<String>) -> Self {
        Self {
            health: Health::Degraded,
            issues: Vec::new(),
            warnings,
            metrics: SystemMetrics::default(),
        }
    }

    /// Create an unhealthy status with issues
    pub fn unhealthy(issues: Vec<String>) -> Self {
        Self {
            health: Health::Critical,
            issues,
            warnings: Vec::new(),
            metrics: SystemMetrics::default(),
        }
    }

    /// Add a warning to the health status
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        if self.health == Health::Healthy {
            self.health = Health::Degraded;
        }
    }

    /// Add an issue to the health status
    pub fn add_issue(&mut self, issue: String) {
        self.issues.push(issue);
        self.health = Health::Critical;
    }

    /// Check if the status is healthy
    pub fn is_healthy(&self) -> bool {
        self.health == Health::Healthy
    }
}
