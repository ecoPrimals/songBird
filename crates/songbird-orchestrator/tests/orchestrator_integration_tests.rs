// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Integration tests for Songbird Orchestrator
//!
//! These tests validate the complete orchestrator functionality including:
//! - Task lifecycle
//! - Access control
//! - Observability
//! - Resource management

use anyhow::Result;
use songbird_orchestrator::{
    access_control::{AccessControl, AccessToken, AuthMode, Capability},
    task_lifecycle::{Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId},
};
use songbird_process_env::ScopedEnv;
use std::sync::Arc;

#[tokio::test]
async fn test_access_control_student_permissions() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    // Create student token
    let student_token = AccessToken::student("student-123", "CSE-847");

    // Student can view educational info
    assert!(
        ac.check_access(&student_token, &Capability::ViewEducationalInfo).await?,
        "Student should be able to view educational info"
    );

    // Student can submit tasks
    assert!(
        ac.check_access(&student_token, &Capability::SubmitTask).await?,
        "Student should be able to submit tasks"
    );

    // Student CANNOT view infrastructure
    assert!(
        !ac.check_access(&student_token, &Capability::ViewInfrastructureInfo).await?,
        "Student should NOT be able to view infrastructure info"
    );

    Ok(())
}

#[tokio::test]
async fn test_access_control_ta_permissions() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    // Create TA token
    let ta_token = AccessToken::ta("ta-456", "CSE-847");

    // TA can view operational info
    assert!(
        ac.check_access(&ta_token, &Capability::ViewOperationalInfo).await?,
        "TA should be able to view operational info"
    );

    // TA can view all student tasks
    assert!(
        ac.check_access(&ta_token, &Capability::ViewAllStudentTasks).await?,
        "TA should be able to view all student tasks"
    );

    // TA can access student logs
    assert!(
        ac.check_access(&ta_token, &Capability::AccessStudentLogs).await?,
        "TA should be able to access student logs"
    );

    // TA CANNOT view infrastructure
    assert!(
        !ac.check_access(&ta_token, &Capability::ViewInfrastructureInfo).await?,
        "TA should NOT be able to view infrastructure info"
    );

    Ok(())
}

#[tokio::test]
async fn test_access_control_admin_permissions() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    // Create admin token
    let admin_token = AccessToken::admin("admin-789");

    // Admin can view everything (All capability)
    assert!(
        ac.check_access(&admin_token, &Capability::ViewInfrastructureInfo).await?,
        "Admin should be able to view infrastructure info"
    );

    assert!(
        ac.check_access(&admin_token, &Capability::ManageConfiguration).await?,
        "Admin should be able to manage configuration"
    );

    assert!(
        ac.check_access(&admin_token, &Capability::RestartServices).await?,
        "Admin should be able to restart services"
    );

    Ok(())
}

#[tokio::test]
async fn test_capability_implication_hierarchy() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    // Admin with infrastructure access should implicitly have all lower layers
    let admin_token = AccessToken::admin("admin");

    // Infrastructure implies administrative
    assert!(
        ac.check_access(&admin_token, &Capability::ViewAdministrativeInfo).await?,
        "Infrastructure capability should imply administrative"
    );

    // Infrastructure implies operational
    assert!(
        ac.check_access(&admin_token, &Capability::ViewOperationalInfo).await?,
        "Infrastructure capability should imply operational"
    );

    // Infrastructure implies educational
    assert!(
        ac.check_access(&admin_token, &Capability::ViewEducationalInfo).await?,
        "Infrastructure capability should imply educational"
    );

    // Infrastructure implies public
    assert!(
        ac.check_access(&admin_token, &Capability::ViewPublicInfo).await?,
        "Infrastructure capability should imply public"
    );

    Ok(())
}

#[tokio::test]
async fn test_token_expiry() -> Result<()> {
    let mut token = AccessToken::student("student-123", "CSE-847");

    // Fresh token should not be expired
    assert!(!token.is_expired(), "Fresh token should not be expired");

    // Set expiry to past
    token.exp = chrono::Utc::now().timestamp() - 1000;

    // Expired token should be detected
    assert!(token.is_expired(), "Past expiry should be detected");

    Ok(())
}

#[tokio::test]
async fn test_token_encoding_and_decoding() -> Result<()> {
    let secret = b"test-secret-for-jwt";

    let original_token = AccessToken::student("student-123", "CSE-847");

    // Encode token as JWT string
    let jwt_string = original_token.encode(secret)?;

    // Decode token back
    let decoded_token = AccessToken::decode(&jwt_string, secret)?;

    // Verify JWT standard fields match
    assert_eq!(original_token.sub, decoded_token.sub);
    assert_eq!(original_token.iat, decoded_token.iat);
    assert_eq!(original_token.exp, decoded_token.exp);

    Ok(())
}

#[tokio::test]
async fn test_information_layer_building() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    // Create a mock task
    let task_spec = TaskSpec {
        task_type: Arc::from("test-task"),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let mut task = TaskLifecycle::new(UserId::new("user-test"), task_spec);
    task.status = TaskStatus::Completed {
        completed_at: chrono::Utc::now(),
    };

    // Get task info with student token
    let student_token = AccessToken::student("student-123", "CSE-847");
    let student_info = ac.get_visible_task_info(&student_token, &task).await?;

    // Student should see public and educational layers
    assert!(student_info.public.is_some(), "Student should see public info");
    assert!(student_info.educational.is_some(), "Student should see educational info");
    assert!(student_info.operational.is_none(), "Student should NOT see operational info");
    assert!(student_info.infrastructure.is_none(), "Student should NOT see infrastructure info");

    // Get task info with TA token
    let ta_token = AccessToken::ta("ta-456", "CSE-847");
    let ta_info = ac.get_visible_task_info(&ta_token, &task).await?;

    // TA should see public, educational, and operational layers
    assert!(ta_info.public.is_some(), "TA should see public info");
    assert!(ta_info.educational.is_some(), "TA should see educational info");
    assert!(ta_info.operational.is_some(), "TA should see operational info");
    assert!(ta_info.infrastructure.is_none(), "TA should NOT see infrastructure info");

    // Get task info with admin token
    let admin_token = AccessToken::admin("admin");
    let admin_info = ac.get_visible_task_info(&admin_token, &task).await?;

    // Admin should see all layers
    assert!(admin_info.public.is_some(), "Admin should see public info");
    assert!(admin_info.educational.is_some(), "Admin should see educational info");
    assert!(admin_info.operational.is_some(), "Admin should see operational info");
    assert!(admin_info.infrastructure.is_some(), "Admin should see infrastructure info");

    Ok(())
}

#[tokio::test]
async fn test_anonymous_access_restrictions() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    let anonymous_token = AccessToken::anonymous();

    // Anonymous can only view public info
    assert!(
        ac.check_access(&anonymous_token, &Capability::ViewPublicInfo).await?,
        "Anonymous should be able to view public info"
    );

    // Anonymous cannot submit tasks
    assert!(
        !ac.check_access(&anonymous_token, &Capability::SubmitTask).await?,
        "Anonymous should NOT be able to submit tasks"
    );

    // Anonymous cannot view any elevated info
    assert!(
        !ac.check_access(&anonymous_token, &Capability::ViewEducationalInfo).await?,
        "Anonymous should NOT be able to view educational info"
    );

    Ok(())
}

#[tokio::test]
async fn test_professor_permissions() -> Result<()> {
    let _g = songbird_process_env::test_env_lock();
    let _jwt = ScopedEnv::new("SONGBIRD_JWT_SECRET", "integration-test-secret");
    let ac = AccessControl::new(AuthMode::Standalone);

    let professor_token = AccessToken::professor("prof-abc", vec!["CSE-847".into()]);

    // Professor can view administrative info
    assert!(
        ac.check_access(&professor_token, &Capability::ViewAdministrativeInfo).await?,
        "Professor should be able to view administrative info"
    );

    // Professor can manage course users
    assert!(
        ac.check_access(&professor_token, &Capability::ManageCourseUsers).await?,
        "Professor should be able to manage course users"
    );

    // Professor can view statistics
    assert!(
        ac.check_access(&professor_token, &Capability::ViewStatistics).await?,
        "Professor should be able to view statistics"
    );

    // Professor CANNOT view infrastructure
    assert!(
        !ac.check_access(&professor_token, &Capability::ViewInfrastructureInfo).await?,
        "Professor should NOT be able to view infrastructure info"
    );

    Ok(())
}

#[tokio::test]
async fn test_sensitive_capability_detection() {
    // Infrastructure access is sensitive
    assert!(
        Capability::ViewInfrastructureInfo.is_sensitive(),
        "ViewInfrastructureInfo should be sensitive"
    );

    // Configuration management is sensitive
    assert!(
        Capability::ManageConfiguration.is_sensitive(),
        "ManageConfiguration should be sensitive"
    );

    // Educational info is not sensitive
    assert!(
        !Capability::ViewEducationalInfo.is_sensitive(),
        "ViewEducationalInfo should NOT be sensitive"
    );

    // Task submission is not sensitive
    assert!(!Capability::SubmitTask.is_sensitive(), "SubmitTask should NOT be sensitive");
}
