//! Metrics-Aware Load Balancer
//!
//! Load balancing based on real metrics from ToadStool and other primals,
//! not hardcoded algorithms. This is Songbird's core expertise.

use crate::load_balancer::LoadBalancer;
use crate::metrics::{ComputeMetrics, MetricsCapabilityAdapter};
use songbird_errors::{SongbirdError, SongbirdResult};
use songbird_universal_primals::PrimalProvider;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

/// Load balancer that makes decisions based on real metrics from primals
pub struct MetricsAwareLoadBalancer {
    /// Metrics capability adapter for collecting real-time data
    metrics_adapter: Arc<dyn MetricsCapabilityAdapter>,

    /// Base load balancer for routing logic
    load_balancer: Arc<dyn LoadBalancer>,

    /// Cache of recent metrics to avoid constant polling
    metrics_cache: Arc<tokio::sync::RwLock<HashMap<String, CachedMetrics>>>,

    /// How long to cache metrics before refreshing
    cache_duration: Duration,
}

/// Cached metrics with timestamp
#[derive(Debug, Clone)]
struct CachedMetrics {
    compute_metrics: ComputeMetrics,
    cached_at: std::time::Instant,
}

/// Request types for capability-based routing
#[derive(Debug, Clone)]
pub struct ComputeRequest {
    pub request_id: String,
    pub workload_type: String,
    pub cpu_requirement: f64,    // Required CPU percentage
    pub memory_requirement: u64, // Required memory in bytes
    pub priority: RequestPriority,
    pub timeout_seconds: u32,
}

/// Response from compute operations
#[derive(Debug, Clone)]
pub struct ComputeResponse {
    pub request_id: String,
    pub success: bool,
    pub result_data: Option<serde_json::Value>,
    pub processing_time_ms: u64,
    pub handled_by_primal: String,
}

/// Priority levels for routing decisions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Scoring criteria for primal selection
#[derive(Debug, Clone)]
pub struct PrimalScore {
    pub primal_id: String,
    pub total_score: f64,
    pub cpu_score: f64,          // Higher is better (more available CPU)
    pub memory_score: f64,       // Higher is better (more available memory)
    pub load_score: f64,         // Higher is better (less current load)
    pub performance_score: f64,  // ToadStool's performance metrics
    pub availability_score: f64, // Primal health and responsiveness
}

impl MetricsAwareLoadBalancer {
    /// Create new metrics-aware load balancer
    pub fn new(
        metrics_adapter: Arc<dyn MetricsCapabilityAdapter>,
        load_balancer: Arc<dyn LoadBalancer>,
    ) -> Self {
        Self {
            metrics_adapter,
            load_balancer,
            metrics_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            cache_duration: Duration::from_secs(30), // Cache metrics for 30 seconds
        }
    }

    /// Route compute request based on real ToadStool metrics
    pub async fn route_compute_request(
        &self,
        request: ComputeRequest,
    ) -> SongbirdResult<ComputeResponse> {
        info!(
            "🎯 Routing compute request {} based on real metrics",
            request.request_id
        );

        // Get real compute metrics from ToadStool
        let compute_metrics = self.get_cached_compute_metrics().await?;

        debug!(
            "📊 Current ToadStool metrics - CPU: {:.1}%, Memory: {} MB, Containers: {}, Queued: {}",
            compute_metrics.cpu_usage_percent,
            compute_metrics.memory_usage_bytes / 1024 / 1024,
            compute_metrics.active_containers,
            compute_metrics.queued_jobs
        );

        // Find available compute primals
        let available_primals = self.find_compute_primals().await?;

        if available_primals.is_empty() {
            return Err(SongbirdError::service(
                "load_balancer",
                "No compute primals available for routing",
            ));
        }

        // Select best primal based on real metrics and request requirements
        let target_primal = self
            .select_best_compute_primal(&available_primals, &compute_metrics, &request)
            .await?;

        info!(
            "✅ Selected primal '{}' for request {}",
            target_primal.primal_id(),
            request.request_id
        );

        // Route request to selected primal
        self.execute_compute_request(target_primal, request).await
    }

    /// Get cached compute metrics or fetch fresh ones
    async fn get_cached_compute_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        let cache = self.metrics_cache.read().await;

        // Check if we have recent cached metrics
        if let Some(cached) = cache.get("compute") {
            if cached.cached_at.elapsed() < self.cache_duration {
                debug!("📊 Using cached compute metrics");
                return Ok(cached.compute_metrics.clone());
            }
        }

        drop(cache); // Release read lock

        // Fetch fresh metrics from ToadStool
        debug!("📊 Fetching fresh compute metrics from ToadStool");
        let fresh_metrics = self
            .metrics_adapter
            .collect_compute_metrics()
            .await
            .map_err(|e| songbird_errors::SongbirdError::network(e.to_string()))?;

        // Update cache
        let mut cache = self.metrics_cache.write().await;
        cache.insert(
            "compute".to_string(),
            CachedMetrics {
                compute_metrics: fresh_metrics.clone(),
                cached_at: std::time::Instant::now(),
            },
        );

        Ok(fresh_metrics)
    }

    /// Find available compute primals using capability-based discovery
    async fn find_compute_primals(&self) -> SongbirdResult<Vec<Arc<dyn PrimalProvider>>> {
        // Use our universal primal registry to find compute capabilities
        // This is capability-based, not hardcoded to specific primal names

        // For now, simulate this - in full implementation this would use
        // the universal primal registry from songbird-universal-primals

        debug!("🔍 Discovering compute primals via capability-based discovery");

        // This would be: registry.find_by_capability(PrimalCapability::ContainerRuntime {...}).await
        // For now, return empty vec to indicate no primals found
        Ok(Vec::new())
    }

    /// Select best compute primal based on real metrics and request requirements
    async fn select_best_compute_primal(
        &self,
        available_primals: &[Arc<dyn PrimalProvider>],
        compute_metrics: &ComputeMetrics,
        request: &ComputeRequest,
    ) -> SongbirdResult<Arc<dyn PrimalProvider>> {
        if available_primals.is_empty() {
            return Err(SongbirdError::service(
                "load_balancer",
                "No compute primals available for selection",
            ));
        }

        // Score each primal based on real metrics and request requirements
        let mut primal_scores = Vec::new();

        for primal in available_primals {
            let score = self
                .score_primal_for_request(primal, compute_metrics, request)
                .await?;
            primal_scores.push((primal.clone(), score));
        }

        // Sort by total score (highest first)
        primal_scores.sort_by(|a, b| {
            b.1.total_score
                .partial_cmp(&a.1.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let (best_primal, best_score) = primal_scores
            .into_iter()
            .next()
            .ok_or_else(|| SongbirdError::service("load_balancer", "No suitable primal found"))?;

        info!(
            "🏆 Best primal '{}' selected with score {:.2}",
            best_primal.primal_id(),
            best_score.total_score
        );

        debug!("📊 Scoring breakdown - CPU: {:.2}, Memory: {:.2}, Load: {:.2}, Performance: {:.2}, Availability: {:.2}",
            best_score.cpu_score,
            best_score.memory_score,
            best_score.load_score,
            best_score.performance_score,
            best_score.availability_score
        );

        Ok(best_primal)
    }

    /// Score a primal based on real metrics and request requirements
    async fn score_primal_for_request(
        &self,
        primal: &Arc<dyn PrimalProvider>,
        compute_metrics: &ComputeMetrics,
        request: &ComputeRequest,
    ) -> SongbirdResult<PrimalScore> {
        // CPU availability score (0.0-1.0, higher is better)
        let cpu_available_percent = 100.0 - compute_metrics.cpu_usage_percent;
        let cpu_score = if cpu_available_percent >= request.cpu_requirement {
            (cpu_available_percent / 100.0).min(1.0)
        } else {
            0.0 // Can't meet CPU requirement
        };

        // Memory availability score (0.0-1.0, higher is better)
        let memory_score = if compute_metrics.memory_available_bytes >= request.memory_requirement {
            (compute_metrics.memory_available_bytes as f64
                / (compute_metrics.memory_usage_bytes + compute_metrics.memory_available_bytes)
                    as f64)
                .min(1.0)
        } else {
            0.0 // Can't meet memory requirement
        };

        // Load score based on queued jobs (0.0-1.0, lower queue is better)
        let load_score = match compute_metrics.queued_jobs {
            0 => 1.0,       // No queue, perfect
            1..=5 => 0.8,   // Light queue
            6..=20 => 0.5,  // Medium queue
            21..=50 => 0.2, // Heavy queue
            _ => 0.1,       // Very heavy queue
        };

        // Performance score from ToadStool's optimizations
        let performance_score = compute_metrics.performance_score.clamp(0.0, 1.0);

        // Availability score based on primal health
        let health = primal.health().await;
        let availability_score = match health {
            Ok(health_status) => match health_status.status {
                songbird_universal_primals::traits::health::HealthStatus::Healthy => 1.0,
                songbird_universal_primals::traits::health::HealthStatus::Degraded => 0.5,
                songbird_universal_primals::traits::health::HealthStatus::Unhealthy => 0.0,
                songbird_universal_primals::traits::health::HealthStatus::Down => 0.0,
                songbird_universal_primals::traits::health::HealthStatus::Unknown => 0.2,
            },
            Err(_) => 0.0, // Error getting health means unavailable
        };

        // Weight scores based on request priority
        let (cpu_weight, memory_weight, load_weight, performance_weight, availability_weight) =
            match request.priority {
                RequestPriority::Critical => (0.4, 0.3, 0.1, 0.1, 0.1), // Prioritize resources
                RequestPriority::High => (0.3, 0.3, 0.2, 0.1, 0.1), // Balanced with load consideration
                RequestPriority::Normal => (0.2, 0.2, 0.3, 0.2, 0.1), // Consider all factors
                RequestPriority::Low => (0.1, 0.1, 0.4, 0.3, 0.1),  // Prefer less loaded primals
            };

        // Calculate weighted total score
        let total_score = cpu_score * cpu_weight
            + memory_score * memory_weight
            + load_score * load_weight
            + performance_score * performance_weight
            + availability_score * availability_weight;

        Ok(PrimalScore {
            primal_id: primal.primal_id().to_string(),
            total_score,
            cpu_score,
            memory_score,
            load_score,
            performance_score,
            availability_score,
        })
    }

    /// Execute compute request on selected primal
    async fn execute_compute_request(
        &self,
        target_primal: Arc<dyn PrimalProvider>,
        request: ComputeRequest,
    ) -> SongbirdResult<ComputeResponse> {
        let start_time = std::time::Instant::now();

        info!(
            "🚀 Executing compute request {} on primal '{}'",
            request.request_id,
            target_primal.primal_id()
        );

        // Convert our ComputeRequest to PrimalRequest format
        let mut payload = HashMap::new();
        payload.insert(
            "workload_type".to_string(),
            serde_json::json!(request.workload_type),
        );
        payload.insert(
            "cpu_requirement".to_string(),
            serde_json::json!(request.cpu_requirement),
        );
        payload.insert(
            "memory_requirement".to_string(),
            serde_json::json!(request.memory_requirement),
        );
        payload.insert(
            "priority".to_string(),
            serde_json::json!(format!("{:?}", request.priority).to_lowercase()),
        );
        payload.insert(
            "timeout_seconds".to_string(),
            serde_json::json!(request.timeout_seconds),
        );

        let canonical_request = songbird_types::CanonicalRequest {
            request_id: Uuid::new_v4().to_string(),
            operation: "compute".to_string(),
            payload: serde_json::json!(payload),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "context".to_string(),
                    format!("songbird-request-{}", request.request_id),
                );
                meta.insert("priority".to_string(), "128".to_string());
                meta.insert("security_level".to_string(), "standard".to_string());
                meta
            },
            timestamp: chrono::Utc::now(),
        };

        // Execute request on target primal
        let primal_response = target_primal
            .handle_request(canonical_request)
            .await
            .map_err(|e| {
                SongbirdError::service("primal_request", format!("Primal request failed: {e}"))
            })?;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "✅ Compute request {} completed in {}ms",
            request.request_id, processing_time_ms
        );

        // Convert PrimalResponse back to ComputeResponse
        Ok(ComputeResponse {
            request_id: request.request_id,
            success: primal_response.status == "success",
            result_data: Some(serde_json::to_value(primal_response.data).unwrap_or_default()),
            processing_time_ms,
            handled_by_primal: target_primal.primal_id().to_string(),
        })
    }

    /// Get comprehensive metrics summary for monitoring
    pub async fn get_metrics_summary(&self) -> SongbirdResult<MetricsSummary> {
        let compute_metrics = self.get_cached_compute_metrics().await?;

        Ok(MetricsSummary {
            cpu_usage_percent: compute_metrics.cpu_usage_percent,
            memory_usage_percent: if compute_metrics.memory_available_bytes > 0 {
                (compute_metrics.memory_usage_bytes as f64
                    / (compute_metrics.memory_usage_bytes + compute_metrics.memory_available_bytes)
                        as f64)
                    * 100.0
            } else {
                0.0
            },
            active_containers: compute_metrics.active_containers,
            queued_jobs: compute_metrics.queued_jobs,
            performance_score: compute_metrics.performance_score,
            zero_copy_ops_per_sec: compute_metrics.zero_copy_operations_per_sec,
            timestamp: compute_metrics.timestamp,
        })
    }

    /// Clear metrics cache to force fresh collection
    pub async fn refresh_metrics_cache(&self) {
        let mut cache = self.metrics_cache.write().await;
        cache.clear();
        debug!("🔄 Metrics cache cleared, will fetch fresh data on next request");
    }
}

/// Summary of current system metrics for monitoring dashboards
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSummary {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub active_containers: u32,
    pub queued_jobs: u32,
    pub performance_score: f64,
    pub zero_copy_ops_per_sec: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
