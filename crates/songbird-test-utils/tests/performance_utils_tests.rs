//! Tests for performance measurement utilities

use songbird_test_utils::performance::*;
use std::time::Duration;

#[test]
fn test_performance_measurement_new() {
    let pm = PerformanceMeasurement::new("test_operation");
    assert_eq!(pm.operation_name, "test_operation");
    assert_eq!(pm.measurements.len(), 0);
    assert!(pm.min_duration.is_none());
    assert!(pm.max_duration.is_none());
}

#[test]
fn test_performance_measurement_record_single() {
    let mut pm = PerformanceMeasurement::new("test");
    pm.record(Duration::from_millis(100));

    assert_eq!(pm.measurements.len(), 1);
    assert_eq!(pm.min_duration, Some(Duration::from_millis(100)));
    assert_eq!(pm.max_duration, Some(Duration::from_millis(100)));
}

#[test]
fn test_performance_measurement_record_multiple() {
    let mut pm = PerformanceMeasurement::new("test");
    pm.record(Duration::from_millis(100));
    pm.record(Duration::from_millis(50));
    pm.record(Duration::from_millis(150));

    assert_eq!(pm.measurements.len(), 3);
    assert_eq!(pm.min_duration, Some(Duration::from_millis(50)));
    assert_eq!(pm.max_duration, Some(Duration::from_millis(150)));
}

#[test]
fn test_average_duration_empty() {
    let pm = PerformanceMeasurement::new("test");
    assert!(pm.average_duration().is_none());
}

#[test]
fn test_average_duration_single() {
    let mut pm = PerformanceMeasurement::new("test");
    pm.record(Duration::from_millis(100));

    let avg = pm.average_duration().unwrap();
    assert_eq!(avg, Duration::from_millis(100));
}

#[test]
fn test_average_duration_multiple() {
    let mut pm = PerformanceMeasurement::new("test");
    pm.record(Duration::from_millis(100));
    pm.record(Duration::from_millis(200));
    pm.record(Duration::from_millis(300));

    let avg = pm.average_duration().unwrap();
    // Average should be 200ms
    assert_eq!(avg, Duration::from_millis(200));
}

#[test]
fn test_percentile_duration_empty() {
    let pm = PerformanceMeasurement::new("test");
    assert!(pm.percentile_duration(95.0).is_none());
}

#[test]
fn test_percentile_duration_single() {
    let mut pm = PerformanceMeasurement::new("test");
    pm.record(Duration::from_millis(100));

    let p95 = pm.percentile_duration(95.0).unwrap();
    assert_eq!(p95, Duration::from_millis(100));
}

#[test]
fn test_percentile_duration_multiple() {
    let mut pm = PerformanceMeasurement::new("test");
    // Add 100 measurements from 0 to 99ms
    for i in 0..100 {
        pm.record(Duration::from_millis(i));
    }

    // 50th percentile should be around 50ms
    let p50 = pm.percentile_duration(50.0).unwrap();
    assert!(p50.as_millis() >= 40 && p50.as_millis() <= 60);

    // 95th percentile should be around 95ms
    let p95 = pm.percentile_duration(95.0).unwrap();
    assert!(p95.as_millis() >= 85 && p95.as_millis() <= 99);
}

#[test]
fn test_percentile_duration_p99() {
    let mut pm = PerformanceMeasurement::new("test");
    for i in 0..100 {
        pm.record(Duration::from_millis(i));
    }

    let p99 = pm.percentile_duration(99.0).unwrap();
    assert!(p99.as_millis() >= 95);
}

#[test]
fn test_print_summary_empty() {
    let pm = PerformanceMeasurement::new("test");
    // Should not panic
    pm.print_summary();
}

#[test]
fn test_print_summary_with_data() {
    let mut pm = PerformanceMeasurement::new("test");
    pm.record(Duration::from_millis(100));
    pm.record(Duration::from_millis(200));
    pm.record(Duration::from_millis(300));

    // Should not panic
    pm.print_summary();
}

#[tokio::test]
async fn test_benchmark_async_success() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = benchmark_async("test_op", 5, || {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        }
    })
    .await;

    assert!(result.is_ok());
    let measurement = result.unwrap();
    assert_eq!(measurement.measurements.len(), 5);
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn test_benchmark_async_with_actual_work() {
    let result = benchmark_async("sleep_test", 3, || async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    })
    .await;

    assert!(result.is_ok());
    let measurement = result.unwrap();
    assert_eq!(measurement.measurements.len(), 3);

    // Each iteration should take at least 50ms
    for duration in &measurement.measurements {
        assert!(duration.as_millis() >= 45); // Allow small variance
    }
}

#[test]
fn test_benchmark_sync_success() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = benchmark_sync("test_op", 5, || {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(5));
        Ok(())
    });

    assert!(result.is_ok());
    let measurement = result.unwrap();
    assert_eq!(measurement.measurements.len(), 5);
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

#[test]
fn test_benchmark_sync_measures_time() {
    let result = benchmark_sync("sleep_test", 3, || {
        std::thread::sleep(Duration::from_millis(20));
        Ok(())
    });

    assert!(result.is_ok());
    let measurement = result.unwrap();
    assert_eq!(measurement.measurements.len(), 3);

    // Each iteration should take at least 20ms
    for duration in &measurement.measurements {
        assert!(duration.as_millis() >= 15); // Allow small variance
    }
}

#[test]
fn test_load_tester_new() {
    let load_tester = LoadTester::new(10, Duration::from_secs(30));

    assert_eq!(load_tester.concurrent_users, 10);
    assert_eq!(load_tester.test_duration, Duration::from_secs(30));
    assert_eq!(load_tester.ramp_up_duration, Duration::from_secs(10));
}

#[test]
fn test_load_tester_with_ramp_up() {
    let load_tester =
        LoadTester::new(10, Duration::from_secs(30)).with_ramp_up(Duration::from_secs(5));

    assert_eq!(load_tester.concurrent_users, 10);
    assert_eq!(load_tester.test_duration, Duration::from_secs(30));
    assert_eq!(load_tester.ramp_up_duration, Duration::from_secs(5));
}

#[tokio::test]
async fn test_load_tester_run_basic() {
    let load_tester =
        LoadTester::new(2, Duration::from_millis(200)).with_ramp_up(Duration::from_millis(50));

    let result = load_tester
        .run_load_test("basic_test", || async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        })
        .await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert!(!results.samples.is_empty());
    assert_eq!(results.test_name, "basic_test");
}

#[test]
fn test_load_tester_builder_pattern() {
    let load_tester =
        LoadTester::new(5, Duration::from_secs(10)).with_ramp_up(Duration::from_secs(2));

    assert_eq!(load_tester.concurrent_users, 5);
    assert_eq!(load_tester.test_duration.as_secs(), 10);
    assert_eq!(load_tester.ramp_up_duration.as_secs(), 2);
}
