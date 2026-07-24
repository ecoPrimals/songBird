// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Consent request helpers

/// Consent request builder
pub struct ConsentRequestBuilder {
    #[expect(dead_code, reason = "captured for future consent payload serialization")]
    operation: String,
    estimated_cost: Option<f64>,
    justification: Option<String>,
}

impl ConsentRequestBuilder {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            estimated_cost: None,
            justification: None,
        }
    }

    #[must_use]
    pub const fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost = Some(cost);
        self
    }

    pub fn with_justification(mut self, justification: impl Into<String>) -> Self {
        self.justification = Some(justification.into());
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::ConsentRequestBuilder;

    #[test]
    fn builder_new_accepts_str() {
        let _ = ConsentRequestBuilder::new("deploy");
    }

    #[test]
    fn builder_new_accepts_string() {
        let _ = ConsentRequestBuilder::new(String::from("op"));
    }

    #[test]
    fn builder_chains_cost() {
        let _ = ConsentRequestBuilder::new("x").with_cost(0.0);
    }

    #[test]
    fn builder_chains_justification() {
        let _ = ConsentRequestBuilder::new("y").with_justification("because");
    }

    #[test]
    fn builder_full_chain() {
        let _ = ConsentRequestBuilder::new("scale").with_cost(12.5).with_justification("peak load");
    }

    #[test]
    fn builder_const_with_cost() {
        let _ = ConsentRequestBuilder::new("z").with_cost(1.0).with_cost(2.0);
    }
}
