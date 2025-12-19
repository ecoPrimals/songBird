//! Consent request helpers

/// Consent request builder
pub struct ConsentRequestBuilder {
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

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost = Some(cost);
        self
    }

    pub fn with_justification(mut self, justification: impl Into<String>) -> Self {
        self.justification = Some(justification.into());
        self
    }
}
