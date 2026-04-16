// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg(test)]

use crate::error::{IpcError, IpcResult};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::types::HttpResponse;

/// Rotating scripted responses (replaces legacy `MockHttpClient` in unit tests).
pub struct RotatingMockClient {
    responses: Vec<HttpResponse>,
    call_count: AtomicUsize,
}

impl RotatingMockClient {
    pub fn new(responses: Vec<HttpResponse>) -> Self {
        Self {
            responses,
            call_count: AtomicUsize::new(0),
        }
    }

    pub async fn request(
        &self,
        _method: &str,
        _url: &str,
        _headers: &HashMap<String, String>,
        _body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(self.responses[count % self.responses.len()].clone())
    }
}

pub type CapturedRequest = (String, String, HashMap<String, String>, Option<Vec<u8>>);

/// Queues per-call results and records every request (JSON-RPC → HTTP path tests).
pub struct QueuedMockClient {
    pub(super) outcomes: Mutex<VecDeque<IpcResult<HttpResponse>>>,
    pub(super) captures: Mutex<Vec<CapturedRequest>>,
}

impl QueuedMockClient {
    pub fn new(outcomes: Vec<IpcResult<HttpResponse>>) -> Self {
        Self {
            outcomes: Mutex::new(outcomes.into()),
            captures: Mutex::new(Vec::new()),
        }
    }

    pub fn take_captures(&self) -> Vec<CapturedRequest> {
        std::mem::take(&mut *self.captures.lock().expect("poisoned captures mutex"))
    }

    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse> {
        self.captures.lock().expect("poisoned captures mutex").push((
            method.to_string(),
            url.to_string(),
            headers.clone(),
            body.map(<[u8]>::to_vec),
        ));
        self.outcomes
            .lock()
            .expect("poisoned outcomes mutex")
            .pop_front()
            .unwrap_or_else(|| Err(IpcError::Internal("no queued mock response".into())))
    }
}
