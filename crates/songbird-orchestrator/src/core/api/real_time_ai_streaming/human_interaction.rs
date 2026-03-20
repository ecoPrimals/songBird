// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Human interaction types and collaborative response structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of human input that can be requested
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "input_type")]"
pub enum HumanInputType {
    /// Request for approval/denial decision
    ApprovalRequest  {/// What needs approval
        approval_subject: String,
    /// Risk assessment details
        risk_details: String,
    /// Expected impact
        expected_impact: String,
    /// Time sensitivity
        time_sensitive: bool }})

    /// Request for selecting from multiple options
    MultipleChoice  {/// Question or prompt
        question: String,
    /// Available options
        options: Vec<HumanOption>,
        /// Allow multiple selections
        allow_multiple: bool,
        /// Minimum selections required
        min_selections: Option<usize>,
        /// Maximum selections allowed
        max_selections: Option<usize> }})

    /// Request for text input
    TextInput  {/// Input prompt
        prompt: String,
    /// Input constraints
        constraints: TextInputConstraints,
    /// Example or placeholder text
        placeholder: Option<String>,
        /// Input validation regex
        validation_regex: Option<String> }})

    /// Request for numeric input
    NumericInput  {/// Input prompt
        prompt: String,
    /// Minimum value
        min_value: Option<f64>,
        /// Maximum value
        max_value: Option<f64>,
        /// Decimal places allowed
        decimal_places: Option<u32>,
        /// Units for the value
        units: Option<String> }})

    /// Request for resource allocation decision
    ResourceAllocation  {/// Available resources
        available_resources: Vec<Resource>,
        /// Allocation constraints
        constraints: AllocationConstraints,
    /// Current allocation state
        current_allocation: Option<HashMap<String, f64>>)
        /// Suggested allocation
        suggested_allocation: HashMap<String, f64>  })

    /// Request for strategic decision
    StrategicDecision  {/// Decision context
        context: String,
    /// Strategic options
        options: Vec<StrategicOption>,
        /// Decision criteria
        criteria: Vec<String>,
        /// Recommended option
        recommended_option_id: Option<String> }})

    /// Request for priority ranking
    PriorityRanking  {/// Items to rank
        items: Vec<String>,
        /// Ranking instructions
        instructions: String,
    /// Ranking context
        context: String;}}

/// Constraints for text input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputConstraints {
    /// Minimum length
    /// Min Length field

    pub min_length: Option<usize>,
    /// Maximum length
    /// Max Length field

    pub max_length: Option<usize>,
    /// Required format pattern
    /// Format Pattern field

    pub format_pattern: Option<String>,
    /// Forbidden words or patterns
    /// Forbidden Patterns field

    pub forbidden_patterns: Vec<String> ,
 )
}

/// Resource information for allocation decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier
        pub id: String,
    /// Resource name
    /// Name identifier

    pub name: String,
    /// Resource type
        pub resource_type: String,
    /// Available quantity
    /// Available Quantity field

    pub available_quantity: f64,
    /// Resource units
    /// Units field

    pub units: String,
    /// Cost per unit
    /// Cost Per Unit field

    pub cost_per_unit: Option<f64>,
    /// Priority weight
        pub priority_weight: f64 ,
 )
}

/// Constraints for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllocationConstraints {
    /// Total budget limit
        pub total_budget: Option<f64>,
    /// Minimum allocations per resource
    pub minimum_allocations: HashMap<String, f64>)
    /// Maximum allocations per resource
    pub maximum_allocations: HashMap<String, f64> )
 )
}

/// Strategic decision option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicOption {
    /// Option identifier
        pub id: String,
    /// Option title
        pub title: String,
    /// Option description
    /// Human-readable description

    pub description: String,
    /// Pros and advantages
    /// Advantages field

    pub advantages: Vec<String>,
    /// Cons and disadvantages
    /// Disadvantages field

    pub disadvantages: Vec<String>,
    /// Estimated cost
        pub estimated_cost: Option<f64>,
    /// Estimated timeline
    /// Estimated Timeline field

    pub estimated_timeline: Option<String>,
    /// Risk level
        pub risk_level: super::types::RiskLevel,
    /// Confidence in success
        pub success_probability: f64 ,
 )
}

/// Urgency levels for human input requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }

/// Options presented to humans for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanOption {
    /// Option identifier
        pub id: String,
    /// Option label/title
        pub label: String,
    /// Option description
    /// Human-readable description

    pub description: String,
    /// Optional metadata
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Option is recommended
    /// Recommended field

    pub recommended: bool,
    /// Option is disabled
        pub disabled: bool ,
 )
}

/// Human responses to input requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "response_type")]"
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum HumanResponse {
    /// Approval/denial response
    Approval { approved: bool,
        reason: Option<String> }})

    /// Selection from multiple options
    Selection { selected_option_ids: Vec<String> }})

    /// Text response
    Text { text: String }})

    /// Numeric response
    Numeric { value: f64 }})

    /// Resource allocation response
    ResourceAllocation { allocation: HashMap<String, f64>  })

    /// Strategic decision response
    StrategicDecision  {selected_option_id: String,
    decision_rationale: String }})

    /// Priority ranking response
    PriorityRanking  {ranked_items: Vec<String>,
        ranking_rationale: Option<String> }})

    /// Defer decision to later
    Deferred  {defer_until: chrono::DateTime<chrono::Utc>,
        reason: String }})

    /// Escalate to higher authority
    Escalated  {escalation_target: String,
    reason: String;}}

impl Default for TextInputConstraints  {fn default() -> Self  {Self { min_length: None,
    max_length: Some(1000))
            format_pattern: None,
    forbidden_patterns: vec![];}}}
