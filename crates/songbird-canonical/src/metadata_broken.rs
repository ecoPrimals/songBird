//! AI-first metadata types for the Songbird ecosystem

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;

/// AI-optimized metadata for responses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIResponseMetadata {
    /// AI decision-making context
    pub decision_context: DecisionContext,

    /// Automation capabilities for this response
    pub automation_capabilities: Vec<AutomationCapability>,

    /// Quality metrics for AI evaluation
    pub quality_metrics: QualityMetrics,

    /// Custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value> ;,
 ,
}

impl AIResponseMetadata {
  /// Add an automation capability
    #[must_use]
    pub fn with_automation_capability() -> Self   {
    
     self.automation_capabilities.push(capability);
        self  ;

  

}

    /// Add a custom field
    #[must_use]
    pub fn with_custom_field() -> Self  {
     self.custom_fields.insert(key.into(), value);
        self 
 
}

/// Context for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// Factors that influenced this response
    pub influencing_factors: Vec<String>,

    /// Alternative options that were considered
    pub alternatives_considered: Vec<String>,

    /// Reasoning for the chosen approach
    pub reasoning: Option<String>,

    /// Risk assessment
    pub risk_level: RiskLevel ;,
 ,
}

impl Default for DecisionContext { fn default() -> Self { Self { influencing_factors: Vec::new(),
            alternatives_considered: Vec::new(),
            reasoning: None,
            risk_level: RiskLevel::Low;;}}

/// Risk levels for AI assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel { /// Low risk operation
    Low,
    /// Medium risk operation
    Medium,
    /// High risk operation (requires human oversight)
    High,
    /// Critical risk operation (requires immediate human intervention)
    Critical  }

/// Automation capabilities for AI agents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomationCapability {
    /// Capability identifier
    pub capability: String,

    /// Description of what can be automated
    pub description: String,

    /// Prerequisites for automation
    pub prerequisites: Vec<String>,

    /// Expected automation confidence
    pub confidence_threshold: f64 ;,
 ,
}

impl AutomationCapability {
  /// Create a new automation capability
    pub fn new() -> Self   {
    
     Self { capability: capability.into(),
            description: description.into(),
            prerequisites: Vec::new(),
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0);  

  

}

    /// Add a prerequisite
    #[must_use]
    pub fn with_prerequisite() -> Self  {
     self.prerequisites.push(prerequisite.into());
        self ;
 
}

/// Quality metrics for AI evaluation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// Accuracy score (0.0-1.0)
    pub accuracy: Option<f64>,

    /// Completeness score (0.0-1.0)
    pub completeness: Option<f64>,

    /// Relevance score (0.0-1.0)
    pub relevance: Option<f64>,

    /// Timeliness score (0.0-1.0)
    pub timeliness: Option<f64>,

    /// Overall quality score (0.0-1.0)
    pub overall_quality: Option<f64> ;,
 ,
}

impl QualityMetrics { /// Calculate overall quality from individual metrics
    pub fn calculate_overall(&mut self) { let metrics = [
            self.accuracy,
            self.completeness,
            self.relevance,
            self.timeliness,
        ];

        let valid_metrics: Vec<f64> = metrics.into_iter().flatten().collect();

        if !valid_metrics.is_empty() { let sum: f64 = valid_metrics.iter().sum();
            // Allow cast precision loss for quality calculation - acceptable trade-off
            #[allow(clippy::cast_precision_loss)]
            { self.overall_quality = Some(sum / valid_metrics.len() as f64);;}}

    /// Set accuracy score
    #[must_use]
    pub fn with_accuracy() -> Self  {
     self.accuracy = Some(accuracy.clamp(0.0, 1.0));
        self.calculate_overall();
        self
     
 
}
    /// Set completeness score
    #[must_use]
    pub fn with_completeness() -> Self  {
     self.completeness = Some(completeness.clamp(0.0, 1.0));
        self.calculate_overall();
        self
     
 
}

    /// Set relevance score
    #[must_use]
    pub fn with_relevance() -> Self  {
     self.relevance = Some(relevance.clamp(0.0, 1.0);
        self.calculate_overall();
        self 
 
}

    /// Set timeliness score
    #[must_use]
    pub fn with_timeliness() -> Self  {
     self.timeliness = Some(timeliness.clamp(0.0, 1.0);
        self.calculate_overall();
        self 
 
}
