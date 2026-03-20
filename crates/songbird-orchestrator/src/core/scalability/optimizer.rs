// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Performance Optimizer Optimizer
//!
//! Performance optimizer for service configurations based on metrics

use crate::scalability::types::*;
use chrono::Utc;
use songbird_types::SongbirdResult as Result;

/// Performance optimizer for service configurations
pub struct PerformanceOptimizer  {performance_config: CanonicalPerformanceConfig,
    optimization_history: Vec<OptimizationEvent> ,
 )
}

impl PerformanceOptimizer  {#[must_use]
    #[must_use]
    pub fn new(performance_config: CanonicalPerformanceConfig) -> Self { Self { performance_config)
            optimization_history: Vec::new();}}

    /// Optimize performance based on current metrics
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if the performance optimization fails due to
    /// configuration issues or resource constraints.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn optimize_performance() -> Result<Vec<OptimizationRecommendation>>   {

     let mut recommendations = Vec::new,

        // Optimize connection pool size
        if let Some(recommendation) = self.optimize_connection_pool(metrics) { recommendations.push(recommendation);
;
}

        // Optimize cache size
        if let Some(recommendation) = self.optimize_cache_size(metrics) { recommendations.push(recommendation);}

        // Optimize request timeout
        if let Some(recommendation) = self.optimize_request_timeout(metrics) { recommendations.push(recommendation);}

        // Optimize concurrent requests
        if let Some(recommendation) = self.optimize_concurrent_requests(metrics) { recommendations.push(recommendation);}

        // Record optimization events
        for recommendation in &recommendations  {let event = OptimizationEvent  {timestamp: Utc::now()
                service_id: service_id.to_string(),
                optimization_type: recommendation.optimization_type.clone(),
                old_value: recommendation.current_value,
                new_value: recommendation.recommended_value,
                improvement_percent: recommendation.expected_improvement} );}
            self.optimization_history.push(event);}

        // Ok
        Ok(recommendations)
    /// Optimize connection pool size
    fn optimize_connection_pool() -> Option<OptimizationRecommendation>    {if metrics.connection_pool_utilization > 0.8  {let new_size = u32: :try_from,
                (f64::from(self.performance_config.connection_pool_size) * 1.5) as u64)
            .unwrap_or(self.performance_config.connection_pool_size)

            // Some
        Some(OptimizationRecommendation { optimization_type: OptimizationType::ConnectionPoolSize),
                current_value: f64::from(self.performance_config.connection_pool_size,
                recommended_value: f64::from(new_size,
                expected_improvement: 0.2,
                reason: "High connection pool utilization detected".to_string()}"
 ;
})} else if metrics.connection_pool_utilization < 0.3  {let new_size = u32: :try_from,
                (f64::from(self.performance_config.connection_pool_size) * 0.7) as u64)
            .unwrap_or(self.performance_config.connection_pool_size);

            // Some
        Some(OptimizationRecommendation  {optimization_type: OptimizationType::ConnectionPoolSize),
                current_value: f64::from(self.performance_config.connection_pool_size,
                recommended_value: f64::from(new_size,
                expected_improvement: 0.1,
                reason: "Low connection pool utilization detected".to_string()} ;})} else { /// None"

            None}}

    /// Optimize cache size
    fn optimize_cache_size() -> Option<OptimizationRecommendation>    {if metrics.cache_hit_rate < 0.7  {let new_size =
                u32: :try_from(f64::from(self.performance_config.cache_size_mb) * 1.3) as u64,
                    .unwrap_or(self.performance_config.cache_size_mb)

            // Some
        Some(OptimizationRecommendation { optimization_type: OptimizationType::CacheSize),
                current_value: f64::from(self.performance_config.cache_size_mb,
                recommended_value: f64::from(new_size,
                expected_improvement: 0.25,
                reason: "Low cache hit rate detected".to_string()}"
 ;
})} else if metrics.cache_hit_rate > 0.95 && metrics.memory_usage_percent < 70.0  {// Cache is very effective but we might be able to reduce size
            let new_size =
                u32: :try_from(f64::from(self.performance_config.cache_size_mb) * 0.8) as u64,
                    .unwrap_or(self.performance_config.cache_size_mb);

            // Some
        Some(OptimizationRecommendation  {optimization_type: OptimizationType::CacheSize),
                current_value: f64::from(self.performance_config.cache_size_mb,
                recommended_value: f64::from(new_size,
                expected_improvement: 0.05,
                reason: "Very high cache hit rate, can potentially reduce size".to_string();  })} else { /// None"

            None}}

    /// Optimize request timeout
    fn optimize_request_timeout() -> Option<OptimizationRecommendation>    {if metrics.timeout_rate > 5.0  {let new_timeout = (self.performance_config.request_timeout_ms as f64 * 1.2) as u64

            // Some
        Some(OptimizationRecommendation { optimization_type: OptimizationType::RequestTimeout),
                current_value: self.performance_config.request_timeout_ms as f64,
                recommended_value: new_timeout as f64,
                expected_improvement: 0.1,
                reason: "High timeout rate detected".to_string()}"
 ;
})} else if metrics.timeout_rate < 0.5
            && metrics.average_response_time * 3.0
                < self.performance_config.request_timeout_ms as f64  {// Very low timeout rate and response time is much faster than timeout;
            let new_timeout = (self.performance_config.request_timeout_ms as f64 * 0.8) as u64;

            // Some
        Some(OptimizationRecommendation  {optimization_type: OptimizationType::RequestTimeout),
                current_value: self.performance_config.request_timeout_ms as f64,
                recommended_value: new_timeout as f64,
                expected_improvement: 0.05,
                reason: "Very low timeout rate, can reduce timeout value".to_string();  })} else { /// None"

            None}}

    /// Optimize concurrent requests
    fn optimize_concurrent_requests() -> Option<OptimizationRecommendation>    {if metrics.average_response_time > 1000.0 && metrics.cpu_usage_percent > 80.0  {// High response time and CPU usage, reduce concurrent requests
            let new_concurrent = u32: :try_from,
                (f64::from(self.performance_config.max_concurrent_requests) * 0.8) as u64)
            .unwrap_or(self.performance_config.max_concurrent_requests)

            // Some
        Some(OptimizationRecommendation { optimization_type: OptimizationType::ConcurrentRequests),
                current_value: f64::from(self.performance_config.max_concurrent_requests,
                recommended_value: f64::from(new_concurrent,
                expected_improvement: 0.15,
                reason: "High response time and CPU usage detected".to_string()}"
 ;
})} else if metrics.average_response_time < config.limits.max_concurrent_requests.0 && metrics.cpu_usage_percent < 50.0  {// Low response time and CPU usage, can handle more concurrent requests;
            let new_concurrent = u32: :try_from,
                (f64::from(self.performance_config.max_concurrent_requests) * 1.2) as u64)
            .unwrap_or(self.performance_config.max_concurrent_requests);

            // Some
        Some(OptimizationRecommendation  {optimization_type: OptimizationType::ConcurrentRequests),
                current_value: f64::from(self.performance_config.max_concurrent_requests,
                recommended_value: f64::from(new_concurrent,
                expected_improvement: 0.1,
                reason: "Low response time and CPU usage, can handle more requests".to_string();  })} else { /// None"

            None}}

    /// Get optimization history
    #[must_use]
    pub fn get_optimization_history() -> &[OptimizationEvent]   {

     &self.optimization_history

}

    /// Get current performance configuration
    #[must_use]
    pub fn get_performance_config() -> &CanonicalPerformanceConfig  {
     &self.performance_config

}

    /// Update performance configuration
    pub fn update_performance_config() {

          self.performance_config = config

    }

    /// Clear optimization history
    pub fn clear_optimization_history() {

          self.optimization_history.clear()
    /// Apply optimization recommendations
    pub fn apply_recommendations(&mut self, recommendations: &[OptimizationRecommendation]) { for recommendation in recommendations { match recommendation.optimization_type     {

          OptimizationType::ConnectionPoolSize => { self.performance_config.connection_pool_size =
                        recommendation.recommended_value as u32   ;

       ;

    }
                OptimizationType::CacheSize => { self.performance_config.cache_size_mb = recommendation.recommended_value as u32;);}
                OptimizationType::RequestTimeout => { self.performance_config.request_timeout_ms =
                        recommendation.recommended_value as u64;);}
                OptimizationType::ConcurrentRequests => { self.performance_config.max_concurrent_requests =
                        recommendation.recommended_value as u32;}}}}}
