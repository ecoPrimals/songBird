// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Response Types
//!
//! **CANONICAL**: Standardized response types for the Songbird ecosystem

use crate::errors::SongbirdError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// **CANONICAL**: Standard response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdResult<T> {
    /// Success status
    pub success: bool,
    /// Response data (if successful)
    pub data: Option<T>,
    /// Error information (if failed)
    pub error: Option<ResponseError>,
    /// Request metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Error information in responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<HashMap<String, String>>,
}

impl<T> SongbirdResult<T> {
    /// Create a successful response
    #[must_use]
    pub const fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(_request_id: impl Into<String>, _error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ResponseError {
                code: "ERROR".to_string(),
                message: "An error occurred".to_string(),
                details: None,
            }),
            metadata: None,
        }
    }

    /// Create response from `SongbirdError`
    #[must_use]
    pub fn from_error(error: &crate::errors::SongbirdError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ResponseError {
                code: "SONGBIRD_ERROR".to_string(),
                message: error.to_string(),
                details: None,
            }),
            metadata: None,
        }
    }

    /// Add metadata to the response
    pub fn with_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(ref mut metadata) = self.metadata {
            metadata.insert(key.into(), value.into());
        }
        self
    }

    /// Check if the response is successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the response is an error
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.success
    }

    /// Get the data if successful, or return an error
    /// Get data from response
    ///
    /// # Errors
    /// Returns error if response contains an error instead of data
    pub fn get_data(&self) -> Result<&T, String> {
        if self.success {
            self.data.as_ref().map_or_else(
                || Err("Response marked as successful but contains no data".to_string()),
                Ok,
            )
        } else {
            let error_message = self
                .error
                .as_ref()
                .map_or_else(|| "Unknown error".to_string(), |e| e.message.clone());
            Err(error_message)
        }
    }

    /// Convert response to Result<T, String>
    /// Convert response into Result
    ///
    /// # Errors
    /// Returns error if response contains an error instead of data
    pub fn into_result(self) -> Result<T, String> {
        if self.success {
            self.data.map_or_else(
                || Err("Response marked as successful but contains no data".to_string()),
                |data| Ok(data),
            )
        } else {
            let error_message = self
                .error
                .as_ref()
                .map_or_else(|| "Unknown error".to_string(), |e| e.message.clone());
            Err(error_message)
        }
    }
}

impl<T> AIFirstResponse<T> {
    /// Create a new AI-First response
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            data,
            context: None,
            confidence: None,
            suggested_actions: Vec::new(),
        }
    }

    /// Add human context
    pub fn with_context(&mut self, context: impl Into<String>) -> &mut Self {
        self.context = Some(context.into());
        self
    }

    /// Add confidence score
    pub const fn with_confidence(&mut self, confidence: f64) -> &mut Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }

    /// Add suggested action
    pub fn with_action(&mut self, action: impl Into<String>) -> &mut Self {
        self.suggested_actions.push(action.into());
        self
    }
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    #[must_use]
    pub const fn new(items: Vec<T>, page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = total.div_ceil(per_page);
        let has_more = page + 1 < total_pages;

        Self {
            items,
            page,
            per_page,
            total,
            total_pages,
            has_more,
        }
    }
}

/// **CANONICAL**: AI-First API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstResponse<T> {
    /// Response data
    pub data: T,
    /// Human-readable context
    pub context: Option<String>,
    /// AI confidence score (0.0-1.0)
    pub confidence: Option<f64>,
    /// Suggested next actions
    pub suggested_actions: Vec<String>,
}

/// **CANONICAL**: Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Items in current page
    pub items: Vec<T>,
    /// Current page number (0-based)
    pub page: usize,
    /// Items per page
    pub per_page: usize,
    /// Total number of items
    pub total: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Whether there are more pages
    pub has_more: bool,
}

/// Convenience type aliases
pub type StringResponse = SongbirdResult<String>;
pub type BoolResponse = SongbirdResult<bool>;
pub type JsonResponse = SongbirdResult<serde_json::Value>;

/// Utility functions for creating common responses
impl SongbirdResult<String> {
    /// Create a simple success response with message
    #[must_use]
    pub fn ok(message: impl Into<String>) -> Self {
        Self::success(message.into())
    }
}

impl SongbirdResult<bool> {
    /// Create a boolean success response
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self::success(value)
    }
}

impl<T> From<Result<T, SongbirdError>> for SongbirdResult<T> {
    fn from(result: Result<T, SongbirdError>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(error) => Self::from_error(&error),
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;

    #[test]
    fn test_successful_response() {
        let response = SongbirdResult::success("Hello, World!");
        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.data, Some("Hello, World!"));
    }

    #[test]
    fn test_error_response() {
        let response: SongbirdResult<String> =
            SongbirdResult::error("NOT_FOUND", "Resource not found");
        assert!(!response.is_success());
        assert!(response.is_error());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_response_conversion() {
        let success_response = SongbirdResult::success(42);
        let result = success_response.into_result();
        assert_eq!(result, Ok(42));

        let error_response: SongbirdResult<i32> =
            SongbirdResult::error("ERROR", "Something went wrong");
        let result = error_response.into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_ai_first_response() {
        let mut response = AIFirstResponse::new("Hello");
        response.with_context("Greeting response");
        response.with_confidence(0.95);
        response.with_action("Say hello back");

        assert_eq!(response.data, "Hello");
        assert_eq!(response.context, Some("Greeting response".to_string()));
        assert_eq!(response.confidence, Some(0.95));
        assert_eq!(response.suggested_actions.len(), 1);
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3, 4, 5];
        let response = PaginatedResponse::new(items, 0, 10, 5);

        assert_eq!(response.items.len(), 5);
        assert_eq!(response.page, 0);
        assert_eq!(response.total, 5);
        assert_eq!(response.total_pages, 1);
        assert!(!response.has_more);
    }
}
