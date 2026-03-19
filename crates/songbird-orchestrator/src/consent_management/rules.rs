//! Auto-approval rules

/// Rule for auto-approving consent
#[derive(Debug, Clone)]
pub struct AutoApprovalRule {
    pub name: String,
    pub max_cost: Option<f64>,
    pub operations: Vec<String>,
}

impl AutoApprovalRule {
    #[must_use]
    pub fn matches(&self, operation: &str, cost: Option<f64>) -> bool {
        // Check operation
        if !self.operations.is_empty() && !self.operations.contains(&operation.to_string()) {
            return false;
        }

        // Check cost
        if let Some(max) = self.max_cost {
            if let Some(actual_cost) = cost {
                if actual_cost > max {
                    return false;
                }
            }
        }

        true
    }
}
