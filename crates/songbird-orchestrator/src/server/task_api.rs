// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task Lifecycle REST API
//!
//! Provides HTTP endpoints for task management

use crate::task_lifecycle::{TaskFilter, TaskId, TaskLifecycleManager, TaskSpec, TowerId, UserId};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Task API state
#[derive(Clone)]
pub struct TaskApiState {
    pub manager: Arc<TaskLifecycleManager>,
}

/// Create task request
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub owner: String,
    pub spec: TaskSpec,
}

/// Create task response
#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    pub task_id: String,
}

/// Task query parameters
#[derive(Debug, Deserialize)]
pub struct TaskQueryParams {
    pub owner: Option<String>,
    pub tower: Option<String>,
}

/// Progress update request
#[derive(Debug, Deserialize)]
pub struct UpdateProgressRequest {
    pub progress: f32,
}

/// Start task request
#[derive(Debug, Deserialize)]
pub struct StartTaskRequest {
    pub tower: String,
}

/// Checkpoint request
#[derive(Debug, Deserialize)]
pub struct CreateCheckpointRequest {
    pub state: String, // Base64 encoded
}

/// Checkpoint response
#[derive(Debug, Serialize)]
pub struct CreateCheckpointResponse {
    pub checkpoint_id: String,
}

/// Cancel task request
#[derive(Debug, Deserialize)]
pub struct CancelTaskRequest {
    pub reason: Option<String>,
}

/// Create task lifecycle router
pub fn task_lifecycle_router(manager: Arc<TaskLifecycleManager>) -> Router {
    let state = TaskApiState {
        manager,
    };

    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id/start", post(start_task))
        .route("/tasks/:id/pause", post(pause_task))
        .route("/tasks/:id/resume", post(resume_task))
        .route("/tasks/:id/complete", post(complete_task))
        .route("/tasks/:id/cancel", post(cancel_task))
        .route("/tasks/:id/progress", post(update_progress))
        .route("/tasks/:id/checkpoint", post(create_checkpoint))
        .with_state(state)
}

/// Create a new task
async fn create_task(
    State(state): State<TaskApiState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, AppError> {
    let owner = UserId::from(req.owner);
    let task_id = state.manager.create_task(owner, req.spec).await?;

    info!("Task created via API: {}", task_id);

    Ok(Json(CreateTaskResponse {
        task_id: task_id.to_string(),
    }))
}

/// Get a task by ID
async fn get_task(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;

    let task = state.manager.get_task(task_id).await?.ok_or(AppError::TaskNotFound)?;

    Ok(Json(serde_json::to_value(task)?))
}

/// List tasks
async fn list_tasks(
    State(state): State<TaskApiState>,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let filter = TaskFilter {
        owner: params.owner.map(UserId::from),
        tower: params.tower.map(TowerId::from),
        ..Default::default()
    };

    let tasks = state.manager.list_tasks(&filter).await?;

    let tasks_json: Result<Vec<_>, _> =
        tasks.into_iter().map(|t| serde_json::to_value(t)).collect();

    Ok(Json(tasks_json?))
}

/// Start a task
async fn start_task(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
    Json(req): Json<StartTaskRequest>,
) -> Result<StatusCode, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;
    let tower = TowerId::from(req.tower);

    state.manager.start_task(task_id, tower).await?;

    Ok(StatusCode::OK)
}

/// Pause a task
async fn pause_task(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;

    state.manager.pause_task(task_id).await?;

    Ok(StatusCode::OK)
}

/// Resume a task
async fn resume_task(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
    Json(req): Json<StartTaskRequest>,
) -> Result<StatusCode, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;
    let tower = TowerId::from(req.tower);

    state.manager.resume_task(task_id, tower).await?;

    Ok(StatusCode::OK)
}

/// Complete a task
async fn complete_task(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;

    state.manager.complete_task(task_id).await?;

    Ok(StatusCode::OK)
}

/// Cancel a task
async fn cancel_task(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
    Json(req): Json<CancelTaskRequest>,
) -> Result<StatusCode, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;
    let reason = req.reason.map(Arc::from);

    state.manager.cancel_task(task_id, reason).await?;

    Ok(StatusCode::OK)
}

/// Update task progress
async fn update_progress(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<StatusCode, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;

    state.manager.update_progress(task_id, req.progress).await?;

    Ok(StatusCode::OK)
}

/// Create a checkpoint
async fn create_checkpoint(
    State(state): State<TaskApiState>,
    Path(id): Path<String>,
    Json(req): Json<CreateCheckpointRequest>,
) -> Result<Json<CreateCheckpointResponse>, AppError> {
    let task_id: TaskId = id.parse().map_err(|_| AppError::InvalidTaskId)?;

    // Decode base64 state
    let state_bytes = base64::decode(&req.state).map_err(|_| AppError::InvalidCheckpointData)?;

    let checkpoint_id = state.manager.create_checkpoint(task_id, state_bytes).await?;

    Ok(Json(CreateCheckpointResponse {
        checkpoint_id: checkpoint_id.to_string(),
    }))
}

/// API error type
#[derive(Debug)]
pub enum AppError {
    InvalidTaskId,
    TaskNotFound,
    InvalidCheckpointData,
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidTaskId => (StatusCode::BAD_REQUEST, "Invalid task ID"),
            Self::TaskNotFound => (StatusCode::NOT_FOUND, "Task not found"),
            Self::InvalidCheckpointData => (StatusCode::BAD_REQUEST, "Invalid checkpoint data"),
            Self::Internal(err) => {
                error!("Internal error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, message).into_response()
    }
}

// Need base64 for checkpoint data
mod base64 {
    use base64::{Engine as _, engine::general_purpose};

    pub fn decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(input)
    }
}
