//! Consent Management REST API
//!
//! Provides HTTP endpoints for consent management (MVP Week 5)
//!
//! Features:
//! - Request consent for operations
//! - Approve/deny consent requests
//! - Query consent status
//! - List user consents

use crate::consent_management::{ConsentManager, ConsentRecord, ConsentStatus};
use crate::task_lifecycle::{TaskId, UserId};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Consent API state
#[derive(Clone)]
pub struct ConsentApiState {
    pub manager: Arc<ConsentManager>,
}

impl ConsentApiState {
    /// Create new consent API state
    pub fn new(manager: Arc<ConsentManager>) -> Self {
        Self {
            manager,
        }
    }
}

/// Request consent request body
#[derive(Debug, Deserialize)]
pub struct RequestConsentRequest {
    pub user_id: String,
    pub task_id: String,
    pub operation: String,
    pub estimated_cost: Option<f64>,
}

/// Request consent response
#[derive(Debug, Serialize)]
pub struct RequestConsentResponse {
    pub consent_id: String,
    pub status: String,
}

/// Update consent request body
#[derive(Debug, Deserialize)]
pub struct UpdateConsentRequest {
    pub action: ConsentAction,
    pub reason: Option<String>,
}

/// Consent action (approve or deny)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsentAction {
    Approve,
    Deny,
}

/// Consent query parameters
#[derive(Debug, Deserialize)]
pub struct ConsentQueryParams {
    pub status: Option<String>,
}

/// Create consent management router
pub fn consent_routes() -> Router<ConsentApiState> {
    Router::new()
        .route("/consent/request", post(request_consent))
        .route("/consent/:id", get(get_consent))
        .route("/consent/:id", put(update_consent))
        .route("/consent/user/:user_id", get(list_user_consents))
}

/// Request consent for an operation
async fn request_consent(
    State(state): State<ConsentApiState>,
    Json(req): Json<RequestConsentRequest>,
) -> Result<Json<RequestConsentResponse>, ApiError> {
    let user_id = UserId::new(req.user_id);
    let task_id: TaskId = req.task_id.parse().map_err(|_| ApiError::InvalidTaskId)?;

    let consent_id =
        state.manager.request_consent(user_id, task_id, req.operation, req.estimated_cost).await;

    // Get the record to check status (might be auto-approved)
    let record = state
        .manager
        .get_consent(consent_id.as_ref())
        .await
        .ok_or(ApiError::Internal("Failed to retrieve created consent".into()))?;

    info!("Consent requested via API: {} (status: {:?})", consent_id, record.status);

    Ok(Json(RequestConsentResponse {
        consent_id: consent_id.to_string(),
        status: format!("{:?}", record.status),
    }))
}

/// Get consent by ID
async fn get_consent(
    State(state): State<ConsentApiState>,
    Path(id): Path<String>,
) -> Result<Json<ConsentRecord>, ApiError> {
    let record = state.manager.get_consent(&id).await.ok_or(ApiError::ConsentNotFound)?;

    Ok(Json(record))
}

/// Update consent (approve or deny)
async fn update_consent(
    State(state): State<ConsentApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateConsentRequest>,
) -> Result<StatusCode, ApiError> {
    let reason = req.reason.map(Arc::from);

    let success = match req.action {
        ConsentAction::Approve => state.manager.approve(&id, reason).await,
        ConsentAction::Deny => state.manager.deny(&id, reason).await,
    };

    if !success {
        return Err(ApiError::ConsentNotFound);
    }

    info!("Consent {} {:?} via API", id, req.action);

    Ok(StatusCode::OK)
}

/// List consents for a user
async fn list_user_consents(
    State(state): State<ConsentApiState>,
    Path(user_id): Path<String>,
    Query(params): Query<ConsentQueryParams>,
) -> Result<Json<Vec<ConsentRecord>>, ApiError> {
    let user_id = UserId::new(user_id);

    // Get all consents for user
    let mut records = state.manager.list_by_user(&user_id).await;

    // Filter by status if requested
    if let Some(status_str) = params.status {
        let filter_status = parse_consent_status(&status_str)?;
        records.retain(|r| r.status == filter_status);
    }

    Ok(Json(records))
}

/// Parse consent status from string
fn parse_consent_status(s: &str) -> Result<ConsentStatus, ApiError> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(ConsentStatus::Pending),
        "approved" => Ok(ConsentStatus::Approved),
        "denied" => Ok(ConsentStatus::Denied),
        "expired" => Ok(ConsentStatus::Expired),
        _ => Err(ApiError::InvalidStatus),
    }
}

/// API error type
#[derive(Debug)]
pub enum ApiError {
    InvalidTaskId,
    InvalidStatus,
    ConsentNotFound,
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::InvalidTaskId => (StatusCode::BAD_REQUEST, "Invalid task ID"),
            ApiError::InvalidStatus => (StatusCode::BAD_REQUEST, "Invalid consent status"),
            ApiError::ConsentNotFound => (StatusCode::NOT_FOUND, "Consent not found"),
            ApiError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_consent_status() {
        assert!(matches!(parse_consent_status("pending"), Ok(ConsentStatus::Pending)));
        assert!(matches!(parse_consent_status("APPROVED"), Ok(ConsentStatus::Approved)));
        assert!(matches!(parse_consent_status("Denied"), Ok(ConsentStatus::Denied)));
        assert!(matches!(parse_consent_status("expired"), Ok(ConsentStatus::Expired)));
        assert!(parse_consent_status("invalid").is_err());
    }
}
