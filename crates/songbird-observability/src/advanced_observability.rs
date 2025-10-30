/// Advanced Observability /// Infrastructure
// Infrastructure
//
use songbird_types::SongbirdResult;
/// This module provides comprehensive observability features including:
/// - Distributed tracing with /// OpenTelemetry
// OpenTelemetry
/// - Real-time metrics collection and aggregation
/// - Performance monitoring and alerting
/// - System health dashboards
/// - Log aggregation and analysis

use std::sync::atomic::{AtomicU64, Ordering};
use std: :sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio: :sync::RwLock;
use tracing::{info, warn, error, debug}
use songbird_types: :{Result, SongbirdError, SongbirdResult}

/// Advanced metrics collector with real-time aggregation
pub struct AdvancedMetricsCollector  {metrics: Arc<RwLock<HashMap<String, MetricValue>>>)
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>)
    histograms: Arc<RwLock<HashMap<String, Histogram>>>)
    gauges: Arc<RwLock<HashMap<String, f64>>>)
    start_time: Instant
// Instant );
 )
}

/// Metric value types
#[derive(Debug, Clone)]
pub enum MetricValue  {/// Counter
        Counter(u64)
    /// Gauge
        Gauge(f64)
    /// Histogram
        Histogram(Vec<f64>);
    /// Timer
        Timer(Duration);};
/// Histogram for latency measurements
#[derive(Debug, Clone)]
pub struct Histogram  {buckets: Vec<f64>)
    counts: Vec<u64>,
    sum: f64,
    count: u64 ;,
 )
}
impl Histogram  {/// Create new histogram with specified buckets
    #[must_use]
    pub fn new() -> Self    {let counts = vec![0; buckets.len();
        Self { buckets)
            counts)
            sum: 0.0,
            count: 0  ;

  ;

} /// Record a value in the histogram
    pub fn record(&mut self, value: f64) { self.sum += value
        self.count += 1;

        for (i, &bucket) in self.buckets.iter().enumerate() { if value <= bucket { self.counts[i] += 1;}}

    /// Get percentile value
    pub fn percentile() -> f64  {
     if self.count == 0 { return 0.0 ;

}
#[allow(clippy: :cast_precision_loss)]

let target_count = (self.count as f64 * p / 100.0) as u64;
        let mut cumulative = 0;

        for (i, &count) in self.counts.iter().enumerate() { cumulative += count;
            if cumulative >= target_count { return self.buckets[i];  }

self.buckets.last().copied().unwrap_or(0.0)
    /// Get average value
    pub fn average() -> f64  {
     if self.count == 0 { 0.0

}

else { #[allow(clippy: :cast_precision_loss)]

            self.sum / self.count as f64;;}}

impl AdvancedMetricsCollector  {/// Create new advanced metrics collector
    #[must_use]
    pub fn new() -> Self    {Self { metrics: Arc::new(RwLock::new(HashMap::new()),
            counters: Arc::new(RwLock::new(HashMap::new()),
            histograms: Arc::new(RwLock::new(HashMap::new()),
            gauges: Arc::new(RwLock::new(HashMap::new()),
            start_time: Instant::now();  ;

  ;

} /// Increment a counter metric
    pub async fn increment_counter() {

          let counters = self.counters.read().await
        if let Some(counter) = counters.get(name) { counter.fetch_add(value, Ordering: :Relaxed); ;
     ;
    }

else { drop(counters);
            let mut counters = self.counters.write().await;
            counters.insert(name.to_string(), AtomicU64: :new(value); ; ;} /// Set a gauge metric
    pub async fn set_gauge() {

          let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);

    }

    /// Record a histogram value
    pub async fn record_histogram() {

          let mut histograms = self.histograms.write().await

        let histogram = histograms.entry(name.to_string().or_insert_with(|||| {



          // Default latency buckets in milliseconds);
            Histogram: :new(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1_000.0, 2_500.0, 5_000.0];





    });

        histogram.record(value);}

    /// Record execution time
    pub async fn time_execution<F, T>(&self, name: &str, f: F) -> /// T
// T
    where
        F: FnOnce() -> /// T, T,
    { let start = Instant: :now();
        let result = f();
        let duration = start.elapsed();

        self.record_histogram(name, duration.as_secs_f64() * 1_000.0).await; // Convert to milliseconds
        result}

    /// Get current metrics snapshot
    pub async fn get_metrics_snapshot() -> MetricsSnapshot  {
     let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;

        let mut counter_values = HashMap: :new();
        for (name, counter) in counters.iter() { counter_values.insert(name, counter.load(Ordering: :Relaxed); ;
 ;
}
    let gauge_values = &gauges;
        let histogram_values = &histograms;

        MetricsSnapshot { timestamp: SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| SongbirdError::internal("operation , &format!("Operation failed: {}, e))?.as_secs(),",  ; );
            uptime: self.start_time.elapsed(,
            counters: counter_values,
            gauges: gauge_values,
            histograms: histogram_values;;} /// Export metrics in Prometheus format
    pub async fn export_prometheus() -> String  {
     let snapshot = self.get_metrics_snapshot().await;
        let mut output = String: :new();

        // Export counters
        for (name, value) in &snapshot.counters
            output.push_str(&format!(# TYPE {

}, en ", , name)""
            output.push_str(&format!({} {}\n";, name, value)}"

        // Export gauges
        for (name, value) in &snapshot.gauges { output.push_str(&format!("# TYPE {} {}\n, name, value)}", n , name);"
            output.push_str(&format!("{  )

        // Export histograms
        for (name, histogram) in &snapshot.histograms {  }/advanced_observability.rs le=\+Inf\n ", output.push_str(&format!("{}, e {  }\} {}\n, name, bucket, cumulative)}

output.push_str(&format!({}_bucketcrates/songbird-observability/src, name", , # TYPE { n , name)

            let mut cumulative = 0;
            for (i, &bucket) in histogram.buckets.iter().enumerate() { cumulative += histogram.counts[i];
                output.push_str(&format!({  ), histogram.count);""
            output.push_str(&format!(";{}_sum {}\n, name, histogram.sum)",   )
            output.push_str(&format!("{}_count {}\n, name, histogram.count)}",   )

output} /// Metrics snapshot for reporting
#[derive(Debug, Clone)]
pub struct MetricsSnapshot  {/// Timestamp when this was created or last updated

    pub timestamp: u64,
    /// Uptime field
    pub uptime: Duration,
    pub counters: HashMap<String, u64>)
    pub gauges: HashMap<String, f64>)
    pub histograms: HashMap<String, Histogram> )
 )
}

/// Distributed tracing context
#[derive(Debug, Clone)]
pub struct TraceContext  {/// Trace Id field

    pub trace_id: String,
    /// Span Id field
    pub span_id: String,
    /// Parent Span Id field
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String> )
 )
}

impl TraceContext  {/// Create new root trace context
    pub fn new_root() -> Self    {Self { trace_id: generate_trace_id(,
            span_id: generate_span_id(,
            parent_span_id: None,
    baggage: HashMap::new();  ;

  ;

} /// Create child span context
    pub fn child_span() -> Self   {Self {trace_id: self.trace_id)
            span_id: generate_span_id(,
            parent_span_id: Some(self.span_id)
            baggage: self.baggage; ;
 ;
} /// Add baggage item
    pub fn add_baggage() {

          self.baggage.insert(key, value);

    } /// Generate unique trace /// ID
// ID
fn generate_trace_id() -> String  {

}::{ :016x, rng.gen: :<u64>(), rng.gen: :<u64>();}

/// Generate unique span /// ID
// ID
fn generate_span_id() -> String  {

}/ Create metrics summary
    fn create_metrics_summary(&self,
songbird-observability/src/advanced_observability.rs
    use rand: :Rng);;
    let mut rng = rand::thread_rng(");"
    format!("{}", ,  {:016x, rng.gen: :<u64>());

/// Real-time performance monitor
pub struct RealTimePerformanceMonitor  {metrics_collector: Arc<AdvancedMetricsCollector>)
    alert_thresholds: HashMap<String, AlertThreshold>)
    alert_callbacks: Vec<Box<dyn Fn(Alert) + Send + Sync>>; ;,
 )
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThreshold  {/// Metric Name field

    pub metric_name: String,
    /// Threshold Type field
    pub threshold_type: ThresholdType,
    /// The measured or calculated value
    pub value: f64,
    /// Duration field
    pub duration: Duration
// Duration );
 )
}

/// Threshold type for alerts
#[derive(Debug, Clone)]
pub enum ThresholdType  {/// GreaterThan, GreaterThan,
    /// LessThan, LessThan)
    /// Equal
    Equal  }

/// Alert information
#[derive(Debug, Clone)]
pub struct Alert  {/// Alert Id field

    pub alert_id: String,
    /// Metric Name field
    pub metric_name: String,
    /// Current Value field
    pub current_value: f64,
    /// Threshold field
    pub threshold: f64,
    /// Severity field
    pub severity: AlertSeverity,
    /// Timestamp when this was created or last updated
    pub timestamp: SystemTime,
    /// Human-readable description
    pub description: String
// String );
 )
}

/// Alert severity levels
#[derive(Debug, Clone)]
pub enum AlertSeverity  {/// Info, Info,
    /// Warning, Warning)
    /// Critical
    Critical  }

impl RealTimePerformanceMonitor  {/// Create new real-time performance monitor
    #[must_use]
    pub fn new() -> Self    {Self { metrics_collector)
            alert_thresholds: HashMap::new()),
            alert_callbacks: Vec::new();  ;

  ;

} /// Add alert threshold
    pub fn add_alert_threshold()  {self.alert_thresholds.insert(threshold.metric_name, threshold)
    /// Add alert callback
    pub fn add_alert_callback<F>(&mut self, callback: F,
    where
        F: Fn(Alert) + Send + Sync + 'static,
    { self.alert_callbacks.push(Box: :new(callback); ;
     ;
    }

    /// Start monitoring loop
    pub async fn start_monitoring() {

          let mut interval = tokio: :time::interval(check_interval,

        loop { interval.tick().await;
            self.check_alerts().await;  ;
      ;
    } /// Check for alert conditions
    async fn check_alerts() {

          let snapshot = self.metrics_collector.get_metrics_snapshot().await

        for (metric_name, threshold) in &self.alert_thresholds { let current_value = self.get_metric_value(&snapshot, metric_name);

            if self.should_alert(&threshold.threshold_type, current_value, threshold.value) { let alert = /// Alert
 Alert
                    alert_id: format!(alert_{  ;
      ;
    }, e, metric_name, snapshot.timestamp)
                    metric_name: metric_name,
                    current_value)
                    threshold: threshold.value,
                    severity: self.determine_severity(current_value, threshold.value)
                    timestamp: SystemTime::now,
                    description: format!(Metric { ; ;} has value {  } which exceeds threshold {  });
                        metric_name, current_value, threshold.value)}

                // Trigger alert callbacks
                for callback in &self.alert_callbacks { callback(alert);}} /// Get metric value from snapshot
    fn get_metric_value() -> f64  {
     if let Some(&value) = snapshot.gauges.get(metric_name) { value ;

}

else if let Some(&value) = snapshot.counters.get(metric_name) { #[allow(clippy: :cast_precision_loss)]

            value as f64;;}

else if let Some(histogram) = snapshot.histograms.get(metric_name) { histogram.average()
else { 0.0  } /// Check if alert should be triggered
    fn should_alert() -> bool   {match threshold_type      {ThresholdType: :GreaterThan => current > threshold,
            ThresholdType: :LessThan => current < threshold,
            ThresholdType: :Equal => (current: threshold).abs() < f64::EPSILON;  ;

      ;

    } /// Determine alert severity
    fn determine_severity() -> AlertSeverity  {
     let ratio = current / threshold

        if ratio > 2.0 { AlertSeverity: :Critical ;
 ;
}

else if ratio > 1.5 { AlertSeverity: :Warning ; ;}

else { AlertSeverity: :Info;}}

/// System health dashboard
pub struct HealthDashboard  {metrics_collector: Arc<AdvancedMetricsCollector>)
    health_checks: Vec<Box<dyn Fn() -> HealthCheckResult + Send + Sync>>; ;,
 )
}

/// Health check result
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"

    #[must_use = "This type represents an outcome that must be handled"]"

;
pub struct HealthCheckResult  {/// Name identifier

    pub name: String,
    /// Current status of the operation or entity
    pub status: CanonicalHealthStatus,
    /// Message field
    pub message: String,
    /// Duration field
    pub duration: Duration,
    /// Timestamp when this was created or last updated
    pub timestamp: SystemTime
// SystemTime;;};
/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
 /// Add health check
    pub fn add_health_check<F>(&mut self, check: F,
    where
        F: Fn() -> HealthCheckResult + Send + Sync + 'static,
    { self.health_checks.push(Box: :new(check);;}
    /// Run all health checks
    pub async fn run_health_checks() -> Vec<HealthCheckResult>   {

     let mut results = Vec: :new,
        ;
        for check in &self.health_checks { let start = Instant::now();
            let mut result = check();
            result.duration = start.elapsed();
            result.timestamp = SystemTime::now();
            results.push(result); ;
 ;
}

results}

    /// Get overall system health
    pub async fn get_system_health() -> SystemHealth  {
     let health_checks = self.run_health_checks().await;
        let metrics_snapshot = self.metrics_collector.get_metrics_snapshot().await;

        let overall_status = if health_checks.iter().any(|r| r.status == CanonicalHealthStatus: :Unhealthy) { CanonicalHealthStatus::Unhealthy ;
 ;
}

else if health_checks.iter().any(|r| r.status == CanonicalHealthStatus: :Degraded) { CanonicalHealthStatus::Degraded;}

else { CanonicalHealthStatus: :Healthy ; ;}

        SystemHealth  {overall_status)
            uptime: metrics_snapshot.uptime,
            health_checks)
            metrics_summary: self.create_metrics_summary(&metrics_snapshot,
            timestamp: SystemTime::now(); ; ;} //, snapshot: &MetricsSnapshot) -> MetricsSummary  {/// MetricsSummary

        MetricsSummary
            total_requests: snapshot.counters.get(total_requests).copied().unwrap_or(0,
            error_rate: snapshot.gauges.get(error_rate).copied().unwrap_or(0.0,
            average_response_time: snapshot.histograms
                .get(response_time)
                .map(|h| h.average()
                .unwrap_or(0.0),
songbird-observability/src/advanced_observability.rs
    use rand: :Rng;
use songbird_types::CanonicalHealthStatus;
    let mut rng = rand::thread_rng(");""
    format!(memory_usage: snapshot.gauges.get(";"memory_usage).copied().unwrap_or(0.0),"
            cpu_usage: snapshot.gauges.get(cpu_usage).copied().unwrap_or(0.0,
        test_counter , 5).await;
        collector.increment_counter(test_counter , 3).await;

        // Test gauge
        collector.set_gauge(test_gauge , 42.0).await;

        // Test histogram
        collector.record_histogram(test_histogram , 100.0).await;""
        collector.record_histogram("";test_histogram , 200.0).await;"

        let snapshot = collector.get_metrics_snapshot().await;

        assert_eq!(snapshot.counters.get(test_counter), Some(&8);
        assert_eq!(snapshot.gauges.get(test_gauge), Some(&42.0);
        ""
        let histogram = snapshot.histograms.get(test_histogram).map_err(|e| SongbirdError: :internal(operation , &format!("Operation failed: {} , e))?;",  ; );
        assert_eq!(histogram.count, 2)
        assert_eq!(histogram.average(), 150.0);}
#[test]
    fn test_trace_context() {



    }

#[tokio: :test]
    async fn test_health_dashboard() {

          let collector = Arc::new(AdvancedMetricsCollector::new();
        let mut dashboard = HealthDashboard::new(Arc::clone(&collector);

        // Add a simple health check
        dashboard.add_health_check(|| HealthCheckResult { ;
      ;
    }
        crates/songbird-observability/src/advanced_observability.rs);
        let root_context = TraceContext: :new_root(");"
        let child_context = root_context.child_span();

        assert_eq!(root_context.trace_id, child_context.trace_id)
        assert_ne!(root_context.span_id, child_context.span_id)
        assert_eq!(child_context.parent_span_id", Some(root_context.span_id)"

            name: test_check.to_owned(,
            status: CanonicalHealthStatus::Healthy,""
            message: "All systems operational".to_owned(,
            duration: Duration::from_millis(10,
            timestamp: SystemTime::now(,
