//! Human-AI collaboration requirements and patterns

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Human-AI collaboration requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationRequirements {
    /// Level of human involvement required
    pub human_involvement_level: HumanInvolvementLevel,

    /// AI autonomy level
    pub ai_autonomy_level: AIAutonomyLevel,

    /// Collaboration patterns
    pub patterns: Vec<CollaborationPattern>,

    /// Required expertise level
    pub expertise_requirement: ExpertiseRequirement,

    /// Human approval required
    pub human_approval_required: bool,

    /// Real-time collaboration needed
    pub real_time_collaboration: bool,
}

/// Levels of human involvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HumanInvolvementLevel {
    None,
    Minimal,
    Moderate,
    High,
    Critical,
}

/// AI autonomy levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIAutonomyLevel {
    FullySupervised,
    SemiSupervised,
    SemiAutonomous,
    HighlyAutonomous,
    FullyAutonomous,
}

/// Collaboration pattern description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationPattern {
    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// When this pattern applies
    pub conditions: Vec<String>,

    /// Expected outcomes from this pattern
    pub expected_outcomes: Vec<String>,

    /// Success metrics for this pattern
    pub success_metrics: Vec<String>,
}

/// Expertise requirements for human involvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertiseRequirement {
    /// Required expertise level
    pub level: ExpertiseLevel,

    /// Required expertise domains
    pub domains: Vec<String>,

    /// Minimum experience years
    pub min_experience_years: u32,

    /// Required certifications
    pub certifications: Vec<String>,
}

/// Expertise levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    WorldClass,
}
