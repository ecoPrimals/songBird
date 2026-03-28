// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! AI-first universal response envelope for orchestrator APIs.
//!
//! Endpoints return [`AIFirstResponse`] so automation and humans share the same structured
//! outcome, timing, and optional follow-up actions.

mod types;

pub use types::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Universal AI-first response format for Songbird orchestrator endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct AIFirstResponse<T> {
    /// Operation success status (machine-readable)
    pub success: bool,
    /// Strongly-typed response data
    pub data: T,
    /// AI-optimized error information
    pub error: Option<AIFirstError>,
    /// Unique request identifier for tracing and correlation
    pub request_id: Uuid,
    /// Processing time in milliseconds for performance monitoring
    pub processing_time_ms: u64,
    /// AI-specific metadata for decision making
    pub ai_metadata: AIResponseMetadata,
    /// Human interaction context (when applicable)
    pub human_context: Option<HumanInteractionContext>,
    /// Confidence score for AI decision making (0.0 - 1.0)
    pub confidence_score: f64,
    /// Suggested next actions for AI agents
    pub suggested_actions: Vec<SuggestedAction>,
}

impl<T> AIFirstResponse<T> {
    /// Create a successful AI-first response.
    #[must_use]
    pub fn success(
        data: T,
        request_id: Uuid,
        processing_time_ms: u64,
        confidence_score: f64,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score,
            suggested_actions: Vec::new(),
        }
    }

    /// Create a failed AI-first response (still carries `data`, often partial or placeholder).
    #[must_use]
    pub fn error(data: T, error: AIFirstError, request_id: Uuid, processing_time_ms: u64) -> Self {
        Self {
            success: false,
            data,
            error: Some(error),
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0,
            suggested_actions: Vec::new(),
        }
    }

    /// Whether the response indicates success.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Whether the response indicates an error.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.success
    }

    /// Consume the response and return the payload.
    #[must_use]
    pub fn into_data(self) -> T {
        self.data
    }

    /// Attach human interaction context (builder).
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_human_context(mut self, context: HumanInteractionContext) -> Self {
        self.human_context = Some(context);
        self
    }

    /// Replace AI metadata (builder).
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_ai_metadata(mut self, metadata: AIResponseMetadata) -> Self {
        self.ai_metadata = metadata;
        self
    }

    /// Attach suggested follow-up actions (builder).
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_suggested_actions(mut self, actions: Vec<SuggestedAction>) -> Self {
        self.suggested_actions = actions;
        self
    }
}

#[cfg(test)]
#[path = "ai_first_response_tests.rs"]
mod tests;
