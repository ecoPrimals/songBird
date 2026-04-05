// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

fn roundtrip_json<T>(v: &T)
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let a = serde_json::to_value(v).expect("serialize");
    let back: T = serde_json::from_value(a.clone()).expect("deserialize");
    let b = serde_json::to_value(&back).expect("serialize again");
    assert_eq!(a, b);
}

fn roundtrip_json_ne<T>(v: &T)
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let a = serde_json::to_value(v).expect("serialize");
    let back: T = serde_json::from_value(a.clone()).expect("deserialize");
    let b = serde_json::to_value(&back).expect("serialize again");
    assert_eq!(a, b);
}

#[test]
fn analytics_config_default() {
    let c = AnalyticsConfig::default();
    assert_eq!(c.max_data_points, 10000);
    assert!((c.anomaly_sensitivity - 2.0).abs() < f64::EPSILON);
    assert_eq!(c.prediction_horizon, Duration::from_secs(3600));
    assert_eq!(c.retraining_interval, Duration::from_secs(86400));
    assert!(c.enable_real_time);
}

#[test]
fn analytics_statistics_default() {
    let s = AnalyticsStatistics::default();
    assert_eq!(s.total_data_points, 0);
    assert_eq!(s.trends_analyzed, 0);
    assert_eq!(s.anomalies_detected, 0);
    assert_eq!(s.predictions_made, 0);
    assert_eq!(s.models_trained, 0);
    assert_eq!(s.avg_processing_time, Duration::ZERO);
}

#[test]
fn serde_roundtrip_data_point() {
    let dp = DataPoint {
        timestamp: Utc::now(),
        metric_name: "cpu".to_string(),
        value: 42.5,
        metadata: HashMap::from([("host".to_string(), "a".to_string())]),
    };
    roundtrip_json(&dp);
}

#[test]
fn serde_roundtrip_trend_analysis() {
    let t = TrendAnalysis {
        direction: TrendDirection::Increasing,
        strength: 0.75,
        duration: Duration::from_secs(120),
        confidence: 0.9,
        analyzed_at: Utc::now(),
    };
    roundtrip_json(&t);
}

#[test]
fn serde_roundtrip_anomaly_result() {
    let a = AnomalyResult {
        anomaly_type: AnomalyType::Spike,
        severity: AnomalySeverity::High,
        score: 3.1,
        expected_value: 10.0,
        actual_value: 20.0,
        detected_at: Utc::now(),
        description: "spike".to_string(),
    };
    roundtrip_json(&a);
}

#[test]
fn serde_roundtrip_prediction_result() {
    let p = PredictionResult {
        metric_name: "m".to_string(),
        predicted_value: 1.0,
        confidence: 0.8,
        horizon: Duration::from_secs(600),
        predicted_at: Utc::now(),
        method: PredictionMethod::LinearRegression,
    };
    roundtrip_json(&p);
}

#[test]
fn serde_roundtrip_analytics_config() {
    let c = AnalyticsConfig::default();
    roundtrip_json(&c);
}

#[test]
fn serde_roundtrip_anomaly_model() {
    let m = AnomalyModel {
        name: "x".to_string(),
        baseline_mean: 5.0,
        baseline_std: 0.5,
        threshold_multiplier: 2.0,
        training_size: 100,
        last_updated: Utc::now(),
    };
    roundtrip_json(&m);
}

#[test]
fn serde_roundtrip_prediction_model() {
    let m = PredictionModel {
        name: "p".to_string(),
        coefficients: vec![1.0, 2.0],
        accuracy: 0.95,
        training_size: 50,
        last_trained: Utc::now(),
    };
    roundtrip_json(&m);
}

#[test]
fn serde_roundtrip_analytics_insight() {
    let i = AnalyticsInsight {
        insight_type: InsightType::Trend,
        title: "t".to_string(),
        description: "d".to_string(),
        severity: AnomalySeverity::Medium,
        recommendation: "r".to_string(),
        generated_at: Utc::now(),
    };
    roundtrip_json(&i);
}

#[test]
fn serde_roundtrip_trend_direction_variants() {
    for d in [
        TrendDirection::Increasing,
        TrendDirection::Decreasing,
        TrendDirection::Stable,
        TrendDirection::Volatile,
    ] {
        roundtrip_json_ne(&d);
    }
}

#[test]
fn serde_roundtrip_enums() {
    roundtrip_json_ne(&AnomalyType::Oscillation);
    roundtrip_json_ne(&AnomalySeverity::Critical);
    roundtrip_json_ne(&PredictionMethod::NeuralNetwork);
    roundtrip_json_ne(&InsightType::Resource);
}

#[test]
#[allow(
    clippy::cast_precision_loss,
    reason = "Test fixture: loop index cast to f64 for synthetic series values"
)]
fn linear_regression_straight_line_positive_slope() {
    let base = Utc::now();
    let points: Vec<DataPoint> = (0..5)
        .map(|i| DataPoint {
            timestamp: base + chrono::Duration::seconds(i),
            metric_name: "m".to_string(),
            value: i as f64,
            metadata: HashMap::new(),
        })
        .collect();
    let refs: Vec<&DataPoint> = points.iter().collect();
    let (slope, r2) = ProductionAnalyticsEngine::calculate_linear_regression(&refs);
    assert!(slope > 0.9 && slope < 1.1, "slope={slope}");
    assert!(r2 > 0.99, "r2={r2}");
}

#[test]
fn linear_regression_insufficient_points() {
    let p = DataPoint {
        timestamp: Utc::now(),
        metric_name: "m".to_string(),
        value: 1.0,
        metadata: HashMap::new(),
    };
    let (slope, r2) = ProductionAnalyticsEngine::calculate_linear_regression(&[&p]);
    assert_eq!((slope, r2), (0.0, 0.0));
}

#[tokio::test]
async fn engine_add_point_and_stats() {
    let engine = ProductionAnalyticsEngine::new(AnalyticsConfig {
        enable_real_time: false,
        ..AnalyticsConfig::default()
    });
    let dp = DataPoint {
        timestamp: Utc::now(),
        metric_name: "latency".to_string(),
        value: 10.0,
        metadata: HashMap::new(),
    };
    engine.add_data_point(dp).await.expect("add");
    let s = engine.get_analytics_statistics().await;
    assert_eq!(s.total_data_points, 1);
}

#[tokio::test]
async fn detect_anomaly_creates_model() {
    let engine = ProductionAnalyticsEngine::new(AnalyticsConfig::default());
    let r = engine.detect_anomaly("m", 100.0).await.expect("detect");
    assert_eq!(r.actual_value, 100.0);
    assert!(r.score >= 0.0);
}

#[tokio::test]
async fn trend_analysis_requires_data() {
    let engine = ProductionAnalyticsEngine::new(AnalyticsConfig::default());
    let err = engine.analyze_trends("missing", 5).await.expect_err("err");
    assert!(err.to_string().contains("Unknown"));
}

#[test]
fn generate_trend_recommendation_branches() {
    let hi = TrendAnalysis {
        direction: TrendDirection::Increasing,
        strength: 0.9,
        duration: Duration::from_secs(1),
        confidence: 0.5,
        analyzed_at: Utc::now(),
    };
    assert!(
        ProductionAnalyticsEngine::generate_trend_recommendation(&hi).contains("Strong upward")
    );
    let lo = TrendAnalysis {
        direction: TrendDirection::Increasing,
        strength: 0.5,
        duration: Duration::from_secs(1),
        confidence: 0.5,
        analyzed_at: Utc::now(),
    };
    assert!(
        ProductionAnalyticsEngine::generate_trend_recommendation(&lo).contains("Gradual increase")
    );
    let st = TrendAnalysis {
        direction: TrendDirection::Stable,
        strength: 0.1,
        duration: Duration::from_secs(1),
        confidence: 0.5,
        analyzed_at: Utc::now(),
    };
    assert!(ProductionAnalyticsEngine::generate_trend_recommendation(&st).contains("stable"));
    let vo = TrendAnalysis {
        direction: TrendDirection::Volatile,
        strength: 0.9,
        duration: Duration::from_secs(1),
        confidence: 0.5,
        analyzed_at: Utc::now(),
    };
    assert!(ProductionAnalyticsEngine::generate_trend_recommendation(&vo).contains("volatility"));
}
