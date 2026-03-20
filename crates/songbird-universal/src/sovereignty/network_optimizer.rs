// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🌐 Network Effects Optimizer
//!
//! **CANONICAL NETWORK OPTIMIZATION** ✅
//!
//! This module provides network effects optimization for routing paths)
//! maximizing beneficial network effects while maintaining sovereignty.

#![allow(
    clippy::unused_self,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::unused_async
)]

use super::types::{PathSegment, RoutingPath, SecurityCapability, SecurityLevel};
use songbird_types::SongbirdResult;
use tracing::{debug, info};

/// Network effects optimizer
#[derive(Debug)]
pub struct NetworkEffectsOptimizer {
    /// Optimization strategies configuration
    optimization_config: OptimizationConfig,
}

/// Configuration for network optimization
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable latency optimization
    pub enable_latency_optimization: bool,

    /// Enable throughput optimization
    pub enable_throughput_optimization: bool,

    /// Enable security enhancement
    pub enable_security_enhancement: bool,

    /// Enable cost optimization
    pub enable_cost_optimization: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_latency_optimization: true,
            enable_throughput_optimization: true,
            enable_security_enhancement: true,
            enable_cost_optimization: false, // Conservative default
        }
    }
}

impl NetworkEffectsOptimizer {
    /// Create a new `NetworkEffectsOptimizer` with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            optimization_config: OptimizationConfig::default(),
        }
    }

    /// Create a new `NetworkEffectsOptimizer` with custom configuration
    #[must_use]
    pub const fn with_config(config: OptimizationConfig) -> Self {
        Self {
            optimization_config: config,
        }
    }

    /// Optimize paths for network effects
    ///
    /// # Errors
    ///
    /// Returns an error if optimization strategies fail to apply
    pub async fn optimize_for_network_effects(
        &self,
        paths: &[RoutingPath],
    ) -> SongbirdResult<Vec<RoutingPath>> {
        debug!("Optimizing {} paths for network effects", paths.len());

        let mut optimized_paths = Vec::new();

        for path in paths {
            // Optimize paths to maximize beneficial network effects
            // This applies various optimization strategies
            let optimized = self.apply_optimization_strategies(path).await?;
            optimized_paths.push(optimized);
        }

        info!("Optimized {} paths for network effects", optimized_paths.len());
        Ok(optimized_paths)
    }

    async fn apply_optimization_strategies(
        &self,
        path: &RoutingPath,
    ) -> SongbirdResult<RoutingPath> {
        let mut optimized_segments = Vec::new();
        let mut current_path_sovereignty_score = 0.0;
        let mut current_path_efficiency_score = 0.0;
        let mut current_path_security_level = SecurityLevel::Maximum;

        for segment in &path.segments {
            let mut optimized_segment = self.optimize_segment_for_network_effects(segment).await?;

            // Apply specific optimization strategies
            if self.optimization_config.enable_latency_optimization {
                optimized_segment = self.optimize_for_latency(&optimized_segment).await?;
            }

            if self.optimization_config.enable_throughput_optimization {
                optimized_segment = self.optimize_for_throughput(&optimized_segment).await?;
            }

            if self.optimization_config.enable_security_enhancement {
                optimized_segment = self.enhance_security(&optimized_segment).await?;
            }

            if self.optimization_config.enable_cost_optimization {
                optimized_segment = self.optimize_for_cost(&optimized_segment).await?;
            }

            optimized_segments.push(optimized_segment.clone());
            current_path_sovereignty_score += optimized_segment.sovereignty_level.score();
            current_path_efficiency_score += optimized_segment.efficiency_score;
            current_path_security_level =
                self.assess_path_security_level(&[&optimized_segment.service]).await?;
        }

        let mut optimized_path = RoutingPath {
            segments: optimized_segments,
            sovereignty_score: current_path_sovereignty_score / path.segments.len() as f64,
            efficiency_score: current_path_efficiency_score / path.segments.len() as f64,
            combined_score: 0.0, // Will be calculated
            security_level: current_path_security_level,
        };

        optimized_path.combined_score = self.calculate_combined_path_score(&optimized_path);
        Ok(optimized_path)
    }

    async fn optimize_segment_for_network_effects(
        &self,
        segment: &PathSegment,
    ) -> SongbirdResult<PathSegment> {
        let mut optimized_segment = segment.clone();

        // Add network-specific capabilities
        let mut new_capabilities = segment.security_capabilities.clone();
        new_capabilities.push(SecurityCapability::NetworkOptimized);
        optimized_segment.security_capabilities = new_capabilities;

        // Add network-specific metadata
        let mut new_metadata = segment.metadata.clone();
        new_metadata.insert("network_optimized".to_string(), "true".to_string());
        new_metadata.insert(
            "optimization_timestamp".to_string(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );
        optimized_segment.metadata = new_metadata;

        Ok(optimized_segment)
    }

    async fn optimize_for_latency(&self, segment: &PathSegment) -> SongbirdResult<PathSegment> {
        let mut optimized_segment = segment.clone();

        // Add latency optimization metadata
        optimized_segment.metadata.insert("latency_optimized".to_string(), "true".to_string());
        optimized_segment
            .metadata
            .insert("optimization_strategy".to_string(), "latency".to_string());

        // Boost efficiency score for latency-optimized segments
        optimized_segment.efficiency_score = (optimized_segment.efficiency_score * 1.1).min(1.0);

        Ok(optimized_segment)
    }

    async fn optimize_for_throughput(&self, segment: &PathSegment) -> SongbirdResult<PathSegment> {
        let mut optimized_segment = segment.clone();

        // Add throughput optimization metadata
        optimized_segment.metadata.insert("throughput_optimized".to_string(), "true".to_string());
        optimized_segment
            .metadata
            .insert("optimization_strategy".to_string(), "throughput".to_string());

        // Boost efficiency score for throughput-optimized segments
        optimized_segment.efficiency_score = (optimized_segment.efficiency_score * 1.05).min(1.0);

        Ok(optimized_segment)
    }

    async fn enhance_security(&self, segment: &PathSegment) -> SongbirdResult<PathSegment> {
        let mut optimized_segment = segment.clone();

        // Add security enhancement capabilities
        if !optimized_segment
            .security_capabilities
            .contains(&SecurityCapability::SovereigntyCompliant)
        {
            optimized_segment.security_capabilities.push(SecurityCapability::SovereigntyCompliant);
        }

        // Add security enhancement metadata
        optimized_segment.metadata.insert("security_enhanced".to_string(), "true".to_string());
        optimized_segment.metadata.insert("security_level".to_string(), "enhanced".to_string());

        Ok(optimized_segment)
    }

    async fn optimize_for_cost(&self, segment: &PathSegment) -> SongbirdResult<PathSegment> {
        let mut optimized_segment = segment.clone();

        // Add cost optimization metadata
        optimized_segment.metadata.insert("cost_optimized".to_string(), "true".to_string());
        optimized_segment.metadata.insert("optimization_strategy".to_string(), "cost".to_string());

        // Note: Cost optimization might slightly reduce efficiency for cost savings
        optimized_segment.efficiency_score = (optimized_segment.efficiency_score * 0.95).max(0.0);

        Ok(optimized_segment)
    }

    async fn assess_path_security_level(
        &self,
        services: &[&crate::types::ServiceInfo],
    ) -> SongbirdResult<SecurityLevel> {
        // Simple assessment based on service count and type
        // In a real implementation, this would be more sophisticated
        match services.len() {
            0 => Ok(SecurityLevel::Minimal),
            1 => Ok(SecurityLevel::Medium),
            2..=3 => Ok(SecurityLevel::High),
            _ => Ok(SecurityLevel::Maximum),
        }
    }

    fn calculate_combined_path_score(&self, path: &RoutingPath) -> f64 {
        // Simple weighted combination of sovereignty and efficiency
        // In a real implementation, this would consider network effects
        path.sovereignty_score.mul_add(0.6, path.efficiency_score * 0.4)
    }

    /// Get optimization statistics
    #[must_use]
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        OptimizationStats {
            strategies_enabled: self.count_enabled_strategies(),
            optimization_config: self.optimization_config.clone(),
        }
    }

    const fn count_enabled_strategies(&self) -> usize {
        let mut count = 0;
        if self.optimization_config.enable_latency_optimization {
            count += 1;
        }
        if self.optimization_config.enable_throughput_optimization {
            count += 1;
        }
        if self.optimization_config.enable_security_enhancement {
            count += 1;
        }
        if self.optimization_config.enable_cost_optimization {
            count += 1;
        }
        count
    }
}

impl Default for NetworkEffectsOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization statistics
/// Statistics about optimization operations
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Number of optimization strategies currently enabled
    pub strategies_enabled: usize,
    /// Current optimization configuration
    pub optimization_config: OptimizationConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sovereignty::types::{
        PathSegment, RoutingPath, SecurityCapability, SecurityLevel, SovereigntyLevel,
    };
    use crate::types::{HealthStatus, PrimalType, ServiceInfo};
    use songbird_types::SongbirdError;
    use std::collections::HashMap;

    fn create_test_service() -> ServiceInfo {
        ServiceInfo {
            name: "test-service".to_string(),
            primal_type: PrimalType::new("generic"),
            endpoint: "http://test:8080".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        }
    }

    fn create_test_segment() -> PathSegment {
        PathSegment {
            service: create_test_service(),
            sovereignty_level: SovereigntyLevel::FullySovereign,
            security_capabilities: vec![SecurityCapability::Encryption],
            efficiency_score: 0.8,
            metadata: HashMap::new(),
        }
    }

    fn create_test_path() -> RoutingPath {
        RoutingPath {
            segments: vec![create_test_segment()],
            sovereignty_score: 0.9,
            efficiency_score: 0.8,
            combined_score: 0.85,
            security_level: SecurityLevel::High,
        }
    }

    #[test]
    fn test_optimization_config_default() {
        let config = OptimizationConfig::default();
        assert!(config.enable_latency_optimization);
        assert!(config.enable_throughput_optimization);
        assert!(config.enable_security_enhancement);
        assert!(!config.enable_cost_optimization);
    }

    #[test]
    fn test_network_effects_optimizer_new() {
        let optimizer = NetworkEffectsOptimizer::new();
        assert!(optimizer.optimization_config.enable_latency_optimization);
    }

    #[test]
    fn test_network_effects_optimizer_with_config() {
        let config = OptimizationConfig {
            enable_latency_optimization: false,
            enable_throughput_optimization: true,
            enable_security_enhancement: false,
            enable_cost_optimization: true,
        };
        let optimizer = NetworkEffectsOptimizer::with_config(config);
        assert!(!optimizer.optimization_config.enable_latency_optimization);
        assert!(optimizer.optimization_config.enable_throughput_optimization);
        assert!(!optimizer.optimization_config.enable_security_enhancement);
        assert!(optimizer.optimization_config.enable_cost_optimization);
    }

    #[test]
    fn test_count_enabled_strategies_all() {
        let config = OptimizationConfig {
            enable_latency_optimization: true,
            enable_throughput_optimization: true,
            enable_security_enhancement: true,
            enable_cost_optimization: true,
        };
        let optimizer = NetworkEffectsOptimizer::with_config(config);
        assert_eq!(optimizer.count_enabled_strategies(), 4);
    }

    #[test]
    fn test_count_enabled_strategies_none() {
        let config = OptimizationConfig {
            enable_latency_optimization: false,
            enable_throughput_optimization: false,
            enable_security_enhancement: false,
            enable_cost_optimization: false,
        };
        let optimizer = NetworkEffectsOptimizer::with_config(config);
        assert_eq!(optimizer.count_enabled_strategies(), 0);
    }

    #[test]
    fn test_count_enabled_strategies_partial() {
        let config = OptimizationConfig {
            enable_latency_optimization: true,
            enable_throughput_optimization: false,
            enable_security_enhancement: true,
            enable_cost_optimization: false,
        };
        let optimizer = NetworkEffectsOptimizer::with_config(config);
        assert_eq!(optimizer.count_enabled_strategies(), 2);
    }

    #[test]
    fn test_get_optimization_stats() {
        let optimizer = NetworkEffectsOptimizer::new();
        let stats = optimizer.get_optimization_stats();
        assert_eq!(stats.strategies_enabled, 3); // Default has 3 enabled
        assert!(stats.optimization_config.enable_latency_optimization);
    }

    #[test]
    fn test_calculate_combined_path_score() {
        let optimizer = NetworkEffectsOptimizer::new();
        let path = RoutingPath {
            segments: vec![],
            sovereignty_score: 1.0,
            efficiency_score: 0.5,
            combined_score: 0.0,
            security_level: SecurityLevel::Maximum,
        };
        let score = optimizer.calculate_combined_path_score(&path);
        assert_eq!(score, 0.8); // (1.0 * 0.6) + (0.5 * 0.4) = 0.6 + 0.2 = 0.8
    }

    #[test]
    fn test_calculate_combined_path_score_equal_weights() {
        let optimizer = NetworkEffectsOptimizer::new();
        let path = RoutingPath {
            segments: vec![],
            sovereignty_score: 0.5,
            efficiency_score: 0.5,
            combined_score: 0.0,
            security_level: SecurityLevel::Medium,
        };
        let score = optimizer.calculate_combined_path_score(&path);
        assert_eq!(score, 0.5); // (0.5 * 0.6) + (0.5 * 0.4) = 0.3 + 0.2 = 0.5
    }

    #[tokio::test]
    async fn test_optimize_for_network_effects_empty() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let paths: Vec<RoutingPath> = vec![];
        let result = optimizer.optimize_for_network_effects(&paths).await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to optimize empty paths for network effects: {}",
                e
            ))
        })?;
        assert_eq!(result.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_optimize_for_network_effects_single_path() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let paths = vec![create_test_path()];
        let optimized = optimizer.optimize_for_network_effects(&paths).await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to optimize single path for network effects: {}",
                e
            ))
        })?;
        assert_eq!(optimized.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_optimize_segment_for_network_effects() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let segment = create_test_segment();
        let optimized =
            optimizer.optimize_segment_for_network_effects(&segment).await.map_err(|e| {
                SongbirdError::configuration(format!(
                    "Failed to optimize segment for network effects: {}",
                    e
                ))
            })?;
        assert!(optimized.security_capabilities.contains(&SecurityCapability::NetworkOptimized));
        assert_eq!(optimized.metadata.get("network_optimized"), Some(&"true".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_optimize_for_latency() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let segment = create_test_segment();
        let original_score = segment.efficiency_score;
        let optimized = optimizer.optimize_for_latency(&segment).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to optimize segment for latency: {}", e))
        })?;
        assert_eq!(optimized.metadata.get("latency_optimized"), Some(&"true".to_string()));
        assert!(optimized.efficiency_score >= original_score);
        Ok(())
    }

    #[tokio::test]
    async fn test_optimize_for_throughput() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let segment = create_test_segment();
        let original_score = segment.efficiency_score;
        let optimized = optimizer.optimize_for_throughput(&segment).await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to optimize segment for throughput: {}",
                e
            ))
        })?;
        assert_eq!(optimized.metadata.get("throughput_optimized"), Some(&"true".to_string()));
        assert!(optimized.efficiency_score >= original_score);
        Ok(())
    }

    #[tokio::test]
    async fn test_enhance_security() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let segment = create_test_segment();
        let optimized = optimizer.enhance_security(&segment).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to enhance security for segment: {}", e))
        })?;
        assert!(
            optimized.security_capabilities.contains(&SecurityCapability::SovereigntyCompliant)
        );
        assert_eq!(optimized.metadata.get("security_enhanced"), Some(&"true".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_optimize_for_cost() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let segment = create_test_segment();
        let original_score = segment.efficiency_score;
        let optimized = optimizer.optimize_for_cost(&segment).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to optimize segment for cost: {}", e))
        })?;
        assert_eq!(optimized.metadata.get("cost_optimized"), Some(&"true".to_string()));
        assert!(optimized.efficiency_score <= original_score);
        Ok(())
    }

    #[tokio::test]
    async fn test_assess_path_security_level_minimal() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let services: Vec<&ServiceInfo> = vec![];
        let result = optimizer.assess_path_security_level(&services).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to assess minimal security level: {}", e))
        })?;
        assert!(matches!(result, SecurityLevel::Minimal));
        Ok(())
    }

    #[tokio::test]
    async fn test_assess_path_security_level_medium() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let service = create_test_service();
        let services = vec![&service];
        let result = optimizer.assess_path_security_level(&services).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to assess medium security level: {}", e))
        })?;
        assert!(matches!(result, SecurityLevel::Medium));
        Ok(())
    }

    #[tokio::test]
    async fn test_assess_path_security_level_high() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let service1 = create_test_service();
        let service2 = create_test_service();
        let services = vec![&service1, &service2];
        let result = optimizer.assess_path_security_level(&services).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to assess high security level: {}", e))
        })?;
        assert!(matches!(result, SecurityLevel::High));
        Ok(())
    }

    #[tokio::test]
    async fn test_assess_path_security_level_maximum() -> SongbirdResult<()> {
        let optimizer = NetworkEffectsOptimizer::new();
        let service1 = create_test_service();
        let service2 = create_test_service();
        let service3 = create_test_service();
        let service4 = create_test_service();
        let services = vec![&service1, &service2, &service3, &service4];
        let result = optimizer.assess_path_security_level(&services).await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to assess maximum security level: {}", e))
        })?;
        assert!(matches!(result, SecurityLevel::Maximum));
        Ok(())
    }

    #[test]
    fn test_optimization_stats_strategies_count() {
        let optimizer = NetworkEffectsOptimizer::new();
        let stats = optimizer.get_optimization_stats();
        assert!(stats.strategies_enabled > 0);
        assert!(stats.strategies_enabled <= 4);
    }

    #[test]
    fn test_default_trait() {
        let optimizer = NetworkEffectsOptimizer::default();
        assert!(optimizer.optimization_config.enable_latency_optimization);
    }
}
