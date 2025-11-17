//! Comprehensive tests for ServiceQuery and discovery types
//!
//! Tests ServiceQuery builder, HealthStatus, SortBy, and related types

use songbird_discovery::traits::discovery::{HealthStatus, ServiceQuery, SortBy};
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// ServiceQuery Tests
// ============================================================================

#[test]
fn test_service_query_new() -> SongbirdResult<()> {
    let query = ServiceQuery::new();
    assert!(query.name.is_none());
    assert!(query.service_id.is_none());
    assert!(query.service_type.is_none());
    assert!(query.version.is_none());
    assert!(query.tags.is_empty());
    assert!(query.metadata.is_empty());
    assert!(query.health_status.is_none());
    assert!(query.limit.is_none());
    assert!(query.sort_by.is_none());
    Ok(())
}

#[test]
fn test_service_query_default() -> SongbirdResult<()> {
    let query = ServiceQuery::default();
    assert!(query.service_id.is_none());
    assert!(query.tags.is_empty());
    Ok(())
}

#[test]
fn test_service_query_with_service_id() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_service_id("test-service-123");
    assert_eq!(
        query.service_id.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "test-service-123"
    );
    Ok(())
}

#[test]
fn test_service_query_with_service_type() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_service_type("http-api");
    assert_eq!(
        query.service_type.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "http-api"
    );
    Ok(())
}

#[test]
fn test_service_query_with_version() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_version("1.0.0");
    assert_eq!(
        query.version.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "1.0.0"
    );
    Ok(())
}

#[test]
fn test_service_query_with_single_tag() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_tag("production");
    assert_eq!(query.tags.len(), 1);
    assert_eq!(query.tags[0], "production");
    Ok(())
}

#[test]
fn test_service_query_with_multiple_tags() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_tag("production").with_tag("critical").with_tag("api");

    assert_eq!(query.tags.len(), 3);
    assert!(query.tags.contains(&"production".to_string()));
    assert!(query.tags.contains(&"critical".to_string()));
    assert!(query.tags.contains(&"api".to_string()));
    Ok(())
}

#[test]
fn test_service_query_with_metadata() -> SongbirdResult<()> {
    let query = ServiceQuery::new()
        .with_metadata("region", serde_json::json!("us-west"))
        .with_metadata("tier", serde_json::json!("premium"));

    assert_eq!(query.metadata.len(), 2);
    assert_eq!(
        query
            .metadata
            .get("region")
            .or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        &serde_json::json!("us-west")
    );
    assert_eq!(
        query
            .metadata
            .get("tier")
            .or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        &serde_json::json!("premium")
    );
    Ok(())
}

#[test]
fn test_service_query_with_health_status() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_health_status(HealthStatus::Healthy);
    assert!(matches!(query.health_status, Some(HealthStatus::Healthy)));
    Ok(())
}

#[test]
fn test_service_query_with_limit() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_limit(50);
    assert_eq!(
        query.limit.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        50
    );
    Ok(())
}

#[test]
fn test_service_query_sort_by() -> SongbirdResult<()> {
    let query = ServiceQuery::new().sort_by(SortBy::Name);
    assert!(matches!(query.sort_by, Some(SortBy::Name)));
    Ok(())
}

#[test]
fn test_service_query_builder_chain() -> SongbirdResult<()> {
    let query = ServiceQuery::new()
        .with_service_type("api")
        .with_version(">=1.0.0")
        .with_tag("production")
        .with_tag("critical")
        .with_health_status(HealthStatus::Healthy)
        .with_limit(100)
        .sort_by(SortBy::Health);

    assert_eq!(
        query.service_type.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "api"
    );
    assert_eq!(
        query.version.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        ">=1.0.0"
    );
    assert_eq!(query.tags.len(), 2);
    assert!(matches!(query.health_status, Some(HealthStatus::Healthy)));
    assert_eq!(
        query.limit.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        100
    );
    assert!(matches!(query.sort_by, Some(SortBy::Health)));
    Ok(())
}

#[test]
fn test_service_query_complex_metadata() -> SongbirdResult<()> {
    let query = ServiceQuery::new()
        .with_metadata("env", serde_json::json!("production"))
        .with_metadata("region", serde_json::json!("us-east-1"))
        .with_metadata("az", serde_json::json!("us-east-1a"))
        .with_metadata("instance_type", serde_json::json!("t3.large"));

    assert_eq!(query.metadata.len(), 4);
    assert_eq!(
        query
            .metadata
            .get("env")
            .or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        &serde_json::json!("production")
    );
    assert_eq!(
        query
            .metadata
            .get("az")
            .or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        &serde_json::json!("us-east-1a")
    );
    Ok(())
}

#[test]
fn test_service_query_clone() {
    let query = ServiceQuery::new().with_service_id("test-123").with_tag("production");

    let cloned = query.clone();
    assert_eq!(query.service_id, cloned.service_id);
    assert_eq!(query.tags, cloned.tags);
}

// ============================================================================
// HealthStatus Tests
// ============================================================================

#[test]
fn test_health_status_healthy() {
    let status = HealthStatus::Healthy;
    assert!(matches!(status, HealthStatus::Healthy));
}

#[test]
fn test_health_status_degraded() {
    let status = HealthStatus::Degraded;
    assert!(matches!(status, HealthStatus::Degraded));
}

#[test]
fn test_health_status_unhealthy() {
    let status = HealthStatus::Unhealthy;
    assert!(matches!(status, HealthStatus::Unhealthy));
}

#[test]
fn test_health_status_unknown() {
    let status = HealthStatus::Unknown;
    assert!(matches!(status, HealthStatus::Unknown));
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Degraded, HealthStatus::Unknown);
}

#[test]
fn test_health_status_clone() {
    let status = HealthStatus::Healthy;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[test]
fn test_health_status_in_query() {
    let query_healthy = ServiceQuery::new().with_health_status(HealthStatus::Healthy);
    let query_degraded = ServiceQuery::new().with_health_status(HealthStatus::Degraded);

    assert!(matches!(query_healthy.health_status, Some(HealthStatus::Healthy)));
    assert!(matches!(query_degraded.health_status, Some(HealthStatus::Degraded)));
}

// ============================================================================
// SortBy Tests
// ============================================================================

#[test]
fn test_sort_by_name() {
    let sort = SortBy::Name;
    assert!(matches!(sort, SortBy::Name));
}

#[test]
fn test_sort_by_created_at() {
    let sort = SortBy::CreatedAt;
    assert!(matches!(sort, SortBy::CreatedAt));
}

#[test]
fn test_sort_by_last_seen() {
    let sort = SortBy::LastSeen;
    assert!(matches!(sort, SortBy::LastSeen));
}

#[test]
fn test_sort_by_health() {
    let sort = SortBy::Health;
    assert!(matches!(sort, SortBy::Health));
}

#[test]
fn test_sort_by_equality() {
    assert_eq!(SortBy::Name, SortBy::Name);
    assert_eq!(SortBy::Health, SortBy::Health);
    assert_ne!(SortBy::Name, SortBy::Health);
}

#[test]
fn test_sort_by_clone() {
    let sort = SortBy::Name;
    let cloned = sort.clone();
    assert_eq!(sort, cloned);
}

#[test]
fn test_sort_by_in_query() {
    let query_name = ServiceQuery::new().sort_by(SortBy::Name);
    let query_health = ServiceQuery::new().sort_by(SortBy::Health);

    assert!(matches!(query_name.sort_by, Some(SortBy::Name)));
    assert!(matches!(query_health.sort_by, Some(SortBy::Health)));
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_service_query_serialization() -> SongbirdResult<()> {
    let query =
        ServiceQuery::new().with_service_id("test-123").with_tag("production").with_limit(10);

    let json = serde_json::to_string(&query);
    assert!(json.is_ok());
    Ok(())
}

#[test]
fn test_service_query_deserialization() -> SongbirdResult<()> {
    let json = r#"{
        "name": null,
        "service_id": "test-123",
        "service_type": null,
        "version": null,
        "tags": ["production"],
        "metadata": {},
        "health_status": null,
        "limit": 10,
        "sort_by": null
    }"#;

    let result: Result<ServiceQuery, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    let query = result.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(
        query.service_id.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "test-123"
    );
    assert_eq!(query.tags.len(), 1);
    assert_eq!(
        query.limit.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        10
    );
    Ok(())
}

#[test]
fn test_health_status_serialization() -> SongbirdResult<()> {
    let status = HealthStatus::Healthy;
    let json = serde_json::to_string(&status);
    assert!(json.is_ok());
    assert!(json
        .ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?
        .contains("Healthy"));
    Ok(())
}

#[test]
fn test_sort_by_serialization() -> SongbirdResult<()> {
    let sort = SortBy::Name;
    let json = serde_json::to_string(&sort);
    assert!(json.is_ok());
    assert!(json
        .ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?
        .contains("Name"));
    Ok(())
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_empty_query() -> SongbirdResult<()> {
    let query = ServiceQuery::new();
    // Empty query is valid - should match all services
    assert!(query.service_id.is_none());
    assert!(query.tags.is_empty());
    assert!(query.metadata.is_empty());
    Ok(())
}

#[test]
fn test_query_with_wildcard_version() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_version("*");
    assert_eq!(
        query.version.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "*"
    );
    Ok(())
}

#[test]
fn test_query_with_complex_version_constraint() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_version(">=1.0.0,<2.0.0");
    let version = query.version.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert!(version.contains(">="));
    assert!(version.contains('<'));
    Ok(())
}

#[test]
fn test_query_with_zero_limit() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_limit(0);
    assert_eq!(
        query.limit.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        0
    );
    Ok(())
}

#[test]
fn test_query_with_large_limit() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_limit(10000);
    assert_eq!(
        query.limit.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        10000
    );
    Ok(())
}

#[test]
fn test_query_with_empty_string_values() -> SongbirdResult<()> {
    let query = ServiceQuery::new().with_service_id("").with_service_type("");

    assert_eq!(
        query.service_id.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        ""
    );
    assert_eq!(
        query.service_type.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        ""
    );
    Ok(())
}

#[test]
fn test_query_with_special_characters() -> SongbirdResult<()> {
    let query =
        ServiceQuery::new().with_service_id("test-service-123_v2.0").with_tag("env:production");

    assert_eq!(
        query.service_id.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "test-service-123_v2.0"
    );
    assert_eq!(query.tags[0], "env:production");
    Ok(())
}

#[test]
fn test_metadata_with_complex_json() -> SongbirdResult<()> {
    let complex_value = serde_json::json!({
        "nested": {
            "field": "value",
            "count": 42
        },
        "array": [1, 2, 3]
    });

    let query = ServiceQuery::new().with_metadata("complex", complex_value.clone());
    assert_eq!(
        query
            .metadata
            .get("complex")
            .or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        &complex_value
    );
    Ok(())
}

#[test]
fn test_multiple_queries_independence() -> SongbirdResult<()> {
    let query1 = ServiceQuery::new().with_tag("tag1");
    let query2 = ServiceQuery::new().with_tag("tag2");

    assert_eq!(query1.tags.len(), 1);
    assert_eq!(query2.tags.len(), 1);
    assert_ne!(query1.tags[0], query2.tags[0]);
    Ok(())
}

// ============================================================================
// Real-world Usage Patterns
// ============================================================================

#[test]
fn test_production_service_query() -> SongbirdResult<()> {
    let query = ServiceQuery::new()
        .with_service_type("http-api")
        .with_tag("production")
        .with_tag("critical")
        .with_health_status(HealthStatus::Healthy)
        .with_metadata("region", serde_json::json!("us-east-1"))
        .with_limit(50)
        .sort_by(SortBy::Health);

    assert_eq!(
        query.service_type.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?,
        "http-api"
    );
    assert_eq!(query.tags.len(), 2);
    assert!(matches!(query.health_status, Some(HealthStatus::Healthy)));
    Ok(())
}

#[test]
fn test_development_service_query() {
    let query = ServiceQuery::new()
        .with_tag("development")
        .with_tag("debug")
        .with_metadata("env", serde_json::json!("dev"))
        .sort_by(SortBy::Name);

    assert_eq!(query.tags.len(), 2);
    assert!(matches!(query.sort_by, Some(SortBy::Name)));
}

#[test]
fn test_staging_service_query() {
    let query = ServiceQuery::new()
        .with_service_type("api")
        .with_version(">=2.0.0")
        .with_tag("staging")
        .with_health_status(HealthStatus::Healthy)
        .with_limit(20);

    assert!(query.version.is_some());
    assert_eq!(query.tags[0], "staging");
}
