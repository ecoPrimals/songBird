//! Advanced Real-time Performance Dashboard
//! Enterprise-grade monitoring with intelligent insights

use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::communication::performance_optimizer::PerformanceMetrics;
use crate::network::gaming::advanced_tunnel_system::{BSTPTunnelManager, TunnelMetrics};

/// Advanced dashboard with AI-driven insights
#[derive(Debug)]
pub struct AdvancedDashboard  {/// Real-time metrics collection
    metrics_collector: MetricsCollector,
    /// Performance analytics engine
    analytics_engine: PerformanceAnalytics,
    /// Alert management system
    alert_manager: AlertManager,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Dashboard startup time
    started_at: Instant,
}

/// Intelligent metrics collector
#[derive(Debug)]
pub struct MetricsCollector  {/// Communication performance metrics
    communication_metrics: PerformanceMetrics,
    /// Tunnel performance metrics
    tunnel_metrics: HashMap<String, TunnelMetrics>)
    /// System resource metrics
    system_metrics: SystemMetrics,
    /// Collection interval
    collection_interval: Duration,
    /// Last collection time
    last_collection: Instant,
}

/// AI-powered performance analytics
#[derive(Debug)]
pub struct PerformanceAnalytics  {/// Performance trend analysis
    trend_analyzer: TrendAnalyzer,
    /// Anomaly detection engine
    anomaly_detector: AnomalyDetector,
    /// Predictive performance modeling
    predictor: PerformancePredictor,
    /// Historical data for analysis
    historical_data: Vec<PerformanceSnapshot>,
}

/// Intelligent alert management
#[derive(Debug)]
pub struct AlertManager  {/// Active alerts
    active_alerts: HashMap<String, Alert>)
    /// Alert rules configuration
    alert_rules: Vec<AlertRule>,
    /// Notification channels
    notification_channels: Vec<NotificationChannel>,
    /// Alert suppression rules
    suppression_rules: Vec<SuppressionRule>,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig  {/// Dashboard update interval
    pub update_interval: Duration,
    /// Enable real-time analytics
    pub enable_analytics: bool,
    /// Enable predictive insights
    pub enable_predictions: bool,
    /// Maximum historical data points
    pub max_history_points: usize,
    /// Alert thresholds - using unified config system
    pub alert_thresholds: UnifiedAlertThresholds,
}

/// System resource metrics
#[derive(Debug, Default)]
pub struct SystemMetrics  {/// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Network I/O bytes per second
    pub network_io_bps: u64,
    /// Disk I/O operations per second
    pub disk_iops: u64,
    /// Active network connections
    pub active_connections: u32,
}

/// Performance snapshot for historical analysis
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot  {/// Snapshot timestamp
    pub timestamp: Instant,
    /// Communication metrics at time
    pub communication_metrics: PerformanceMetrics,
    /// System metrics at time
    pub system_metrics: SystemMetrics,
    /// Active tunnels count
    pub active_tunnels: usize,
    /// Overall performance score
    pub performance_score: f64,
}

/// Performance trend analysis
#[derive(Debug)]
pub struct TrendAnalyzer  {/// Trend detection window
    analysis_window: Duration,
    /// Trend confidence threshold
    confidence_threshold: f64,
    /// Detected trends
    detected_trends: Vec<PerformanceTrend>,
}

/// Anomaly detection engine
#[derive(Debug)]
pub struct AnomalyDetector  {/// Baseline metrics for comparison
    baseline_metrics: PerformanceSnapshot,
    /// Anomaly detection sensitivity
    sensitivity: f64,
    /// Detected anomalies
    detected_anomalies: Vec<Anomaly>,
}

/// Predictive performance modeling
#[derive(Debug)]
pub struct PerformancePredictor  {/// Prediction horizon
    prediction_horizon: Duration,
    /// Model accuracy score
    model_accuracy: f64,
    /// Current predictions
    current_predictions: Vec<PerformancePrediction>,
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct Alert  {/// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Alert timestamp
    pub timestamp: Instant,
    /// Alert source component
    pub source: String,
    /// Alert acknowledged
    pub acknowledged: bool,
}

/// Alert rule configuration
#[derive(Debug, Clone)]
pub struct AlertRule  {/// Rule name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Rule enabled
    pub enabled: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity  {/// Information only
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical system issue
    Critical,
}

/// Alert condition types
#[derive(Debug, Clone)]
pub enum AlertCondition  {/// Threshold exceeded
    ThresholdExceeded(f64)
    /// Threshold below minimum
    ThresholdBelow(f64)
    /// Rate of change exceeded
    RateExceeded(f64)
    /// Anomaly detected
    AnomalyDetected,
}

/// Notification channel types
#[derive(Debug, Clone)]
pub enum NotificationChannel  {/// Console logging
    Console,
    /// Email notifications
    Email(String)
    /// Webhook notifications
    Webhook(String)
    /// Slack integration
    Slack(String)
}

/// Alert suppression rule
#[derive(Debug, Clone)]
pub struct SuppressionRule  {/// Rule name
    pub name: String,
    /// Alert pattern to suppress
    pub pattern: String,
    /// Suppression duration
    pub duration: Duration,
    /// Rule enabled
    pub enabled: bool,
}

/// Alert thresholds configuration - UNIFIED VERSION
#[derive(Debug, Clone)]
pub struct UnifiedAlertThresholds  {/// Maximum response time (ms)
    pub max_response_time_ms: u64,
    /// Maximum CPU usage (%)
    pub max_cpu_usage: f64,
    /// Maximum memory usage (bytes)
    pub max_memory_usage: u64,
    /// Minimum performance score
    pub min_performance_score: f64,
}

// Legacy alert thresholds type alias removed - use UnifiedAlertThresholds directly

/// Performance trend detection
#[derive(Debug, Clone)]
pub struct PerformanceTrend  {/// Trend type
    pub trend_type: TrendType,
    /// Trend strength (0.0-1.0)
    pub strength: f64,
    /// Trend duration
    pub duration: Duration,
    /// Affected metric
    pub metric: String,
}

/// Trend types
#[derive(Debug, Clone)]
pub enum TrendType  {/// Performance improving
    Improving,
    /// Performance degrading
    Degrading,
    /// Performance stable
    Stable,
    /// Performance volatile
    Volatile,
}

/// Detected anomaly
#[derive(Debug, Clone)]
pub struct Anomaly  {/// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Anomaly severity
    pub severity: f64,
    /// Affected metric
    pub metric: String,
    /// Detection timestamp
    pub detected_at: Instant,
    /// Anomaly description
    pub description: String,
}

/// Anomaly types
#[derive(Debug, Clone)]
pub enum AnomalyType {/// Performance spike
    Spike,
    /// Performance drop
    Drop,
    /// Unusual pattern
    Pattern,
    /// Resource exhaustion
    ResourceExhaustion,
}

/// Performance prediction
#[derive(Debug, Clone)]
pub struct PerformancePrediction  {/// Predicted metric
    pub metric: String,
    /// Predicted value
    pub predicted_value: f64,
    /// Prediction confidence (0.0-1.0)
    pub confidence: f64,
    /// Prediction timestamp
    pub prediction_time: Instant,
    /// Prediction horizon
    pub horizon: Duration,
}

impl AdvancedDashboard  {/// Create new advanced dashboard
    pub fn new(config: DashboardConfig) -> Self  {Self {
            metrics_collector: MetricsCollector::new(config.update_interval,
            analytics_engine: PerformanceAnalytics::new(,
            alert_manager: AlertManager::new(config.alert_thresholds.clone(),
            config)
            started_at: Instant::now(,
        }
    }

    /// Start real-time monitoring
    pub async fn start_monitoring(&mut self) -> songbird_errors::Result<()> {
        // Initialize monitoring components
        self.metrics_collector.start_collection().await?;
        self.analytics_engine.initialize_baseline().await?;
        self.alert_manager.activate_monitoring().await?;

        Ok(()),
    }

    /// Get current performance overview
    pub fn get_performance_overview(&self) -> PerformanceOverview  {PerformanceOverview  {overall_score: self.calculate_overall_performance_score()
            system_health: self.assess_system_health(,
            active_alerts: self.alert_manager.get_active_alert_count(,
            uptime: self.started_at.elapsed(,
            key_metrics: self.get_key_metrics(,
        }
    }

    /// Get real-time insights
    pub fn get_insights(&self) -> Vec<Insight> {
        let mut insights = Vec::with_capacity(10);

        // Performance insights
        insights.extend(self.analytics_engine.get_performance_insights();

        // Trend insights
        insights.extend(self.analytics_engine.get_trend_insights();

        // Anomaly insights
        insights.extend(self.analytics_engine.get_anomaly_insights();

        // Predictive insights
        if self.config.enable_predictions {
            insights.extend(self.analytics_engine.get_predictive_insights();
        }

        insights
    }

    /// Calculate overall performance score
    fn calculate_overall_performance_score(&self) -> f64 {
        let system_score = self.calculate_system_performance_score();
        let communication_score = self.calculate_communication_performance_score();
        let tunnel_score = self.calculate_tunnel_performance_score();

        // Weighted average
        (system_score * 0.3 + communication_score * 0.4 + tunnel_score * 0.3).min(1.0)
    }

    /// Calculate system performance score
    fn calculate_system_performance_score(&self) -> f64 {
        let cpu_score = 1.0 - (self.metrics_collector.system_metrics.cpu_usage / 100.0);
        let memory_score = if self.metrics_collector.system_metrics.memory_usage > 0 {
            1.0 - (self.metrics_collector.system_metrics.memory_usage as f64 / (8_000_000_000.0) // 8GB baseline
        } else {
            1.0
        };

        (cpu_score + memory_score) / 2.0
    }

    /// Calculate communication performance score
    fn calculate_communication_performance_score(&self) -> f64 {
        let rps = self.metrics_collector.communication_metrics.requests_per_second;
        let avg_time = self.metrics_collector.communication_metrics.avg_response_time;

        if rps > 100.0 && avg_time.as_millis() < 100 {
            1.0
        } else if rps > 50.0 && avg_time.as_millis() < 200 {
            0.8
        } else if rps > 10.0 && avg_time.as_millis() < 500 {
            0.6
        } else {
            0.4
        }
    }

    /// Calculate tunnel performance score
    fn calculate_tunnel_performance_score(&self) -> f64 {
        if self.metrics_collector.tunnel_metrics.is_empty() {
            return 1.0; // No tunnels = perfect score
        }

        let total_score: f64 = self.metrics_collector.tunnel_metrics
            .values()
            .map(|metrics| metrics.gaming_quality_score)
            .sum();

        total_score / self.metrics_collector.tunnel_metrics.len() as f64
    }

    /// Assess system health
    fn assess_system_health(&self) -> SystemHealth {
        let score = self.calculate_overall_performance_score();

        if score >= 0.9 {
            SystemHealth::Excellent
        } else if score >= 0.7 {
            SystemHealth::Good
        } else if score >= 0.5 {
            SystemHealth::Warning
        } else {
            SystemHealth::Critical
        }
    }

    /// Get key metrics summary
    fn get_key_metrics(&self) -> KeyMetrics  {KeyMetrics  {requests_per_second: self.metrics_collector.communication_metrics.requests_per_second)
            avg_response_time: self.metrics_collector.communication_metrics.avg_response_time,
            active_tunnels: self.metrics_collector.tunnel_metrics.len(,
            cpu_usage: self.metrics_collector.system_metrics.cpu_usage,
            memory_usage: self.metrics_collector.system_metrics.memory_usage,
        }
    }
}

/// Performance overview for dashboard
#[derive(Debug)]
pub struct PerformanceOverview  {/// Overall performance score (0.0-1.0)
    pub overall_score: f64,
    /// System health status
    pub system_health: SystemHealth,
    /// Number of active alerts
    pub active_alerts: usize,
    /// System uptime
    pub uptime: Duration,
    /// Key performance metrics
    pub key_metrics: KeyMetrics,
}

/// System health status
#[derive(Debug, Clone, PartialEq)]
pub enum SystemHealth  {/// System performing excellently
    Excellent,
    /// System performing well
    Good,
    /// System has performance warnings
    Warning,
    /// System has critical issues
    Critical,
}

/// Key metrics summary
#[derive(Debug)]
pub struct KeyMetrics  {/// Requests processed per second
    pub requests_per_second: f64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Number of active tunnels
    pub active_tunnels: usize,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
}

/// Dashboard insight
#[derive(Debug, Clone)]
pub struct Insight  {/// Insight type
    pub insight_type: InsightType,
    /// Insight message
    pub message: String,
    /// Insight priority
    pub priority: InsightPriority,
    /// Insight timestamp
    pub timestamp: Instant,
    /// Recommended action
    pub recommended_action: Option<String>,
}

/// Insight types
#[derive(Debug, Clone)]
pub enum InsightType  {/// Performance optimization opportunity
    Performance,
    /// Security concern
    Security,
    /// Resource optimization
    Resource,
    /// Trend analysis
    Trend,
    /// Predictive warning
    Predictive,
}

/// Insight priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum InsightPriority  {/// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

// Implementation stubs for compilation
impl MetricsCollector  {fn new(interval: Duration) -> Self  {Self {
            communication_metrics: PerformanceMetrics::default(),
            tunnel_metrics: HashMap::new()),
            system_metrics: SystemMetrics::default(),
            collection_interval: interval,
            last_collection: Instant::now(,
        }
    }

    async fn start_collection(&mut self) -> songbird_errors::Result<()> {
        // Start metrics collection
        Ok(()),
    }
}

impl PerformanceAnalytics  {fn new() -> Self  {Self {
            trend_analyzer: TrendAnalyzer::new(,
            anomaly_detector: AnomalyDetector::new(,
            predictor: PerformancePredictor::new(,
            historical_data: Vec::with_capacity(1440,
        }
    }

    async fn initialize_baseline(&mut self) -> songbird_errors::Result<()> {
        // Initialize performance baseline
        Ok(()),
    }

    fn get_performance_insights(&self) -> Vec<Insight> {
        Vec::new()
    }

    fn get_trend_insights(&self) -> Vec<Insight> {
        Vec::new()
    }

    fn get_anomaly_insights(&self) -> Vec<Insight> {
        Vec::new()
    }

    fn get_predictive_insights(&self) -> Vec<Insight> {
        Vec::new()
    }
}

impl AlertManager  {fn new(thresholds: AlertThresholds) -> Self  {Self {
            active_alerts: HashMap::new()),
            alert_rules: Vec::new(),
            notification_channels: Vec::new(),
            suppression_rules: Vec::new(),
        }
    }

    async fn activate_monitoring(&mut self) -> songbird_errors::Result<()> {
        // Activate alert monitoring
        Ok(()),
    }

    fn get_active_alert_count(&self) -> usize {
        self.active_alerts.len()
    }
}

impl TrendAnalyzer  {fn new() -> Self  {Self {
            analysis_window: Duration::from_secs(300,
            confidence_threshold: 0.8,
            detected_trends: Vec::new(),
        }
    }
}

impl AnomalyDetector  {fn new() -> Self  {Self {
            baseline_metrics: PerformanceSnapshot {
                timestamp: Instant::now(,
                communication_metrics: PerformanceMetrics::default(),
                system_metrics: SystemMetrics::default(),
                active_tunnels: 0,
                performance_score: 1.0,
            })
            sensitivity: 0.7,
            detected_anomalies: Vec::new(),
        }
    }
}

impl PerformancePredictor  {fn new() -> Self  {Self {
            prediction_horizon: Duration::from_secs(600,
            model_accuracy: 0.85,
            current_predictions: Vec::new(),
        }
    }
}

impl Default for DashboardConfig  {fn default() -> Self  {Self {
            update_interval: Duration::from_secs(5),
            enable_analytics: true,
            enable_predictions: true,
            max_history_points: 1440,
            alert_thresholds: UnifiedAlertThresholds {
                max_response_time_ms: 500,
                max_cpu_usage: 80.0,
                max_memory_usage: 6_000_000_000, // 6GB
                min_performance_score: 0.7,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = AdvancedDashboard::new(config);

        assert!(dashboard.started_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_performance_overview() {
        let config = DashboardConfig::default();
        let dashboard = AdvancedDashboard::new(config);

        let overview = dashboard.get_performance_overview();
        assert!(overview.overall_score >= 0.0));
        assert!(overview.overall_score <= 1.0));
    }

    #[test]
    fn test_system_health_assessment() {
        let config = DashboardConfig::default();
        let dashboard = AdvancedDashboard::new(config);

        let health = dashboard.assess_system_health();
        // Should be good initially
        assert!(matches!(health, SystemHealth::Excellent | SystemHealth::Good));
    }
}
