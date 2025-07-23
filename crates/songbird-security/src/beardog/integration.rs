//! BearDog Security Integration Implementation

use super::types::*;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// BearDog security integration manager
pub struct BeardogIntegration {
    config: BeardogConfig,
    client: Client,
    event_buffer: Arc<RwLock<Vec<SecurityEvent>>>,
    active_policies: Arc<RwLock<Vec<SecurityPolicy>>>,
}

impl BeardogIntegration {
    /// Create new BearDog integration
    pub fn new(config: BeardogConfig) -> Result<Self, std::io::Error> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                tracing::error!("Failed to create BearDog HTTP client: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, 
                    format!("BearDog integration unavailable: HTTP client creation failed: {}", e))
            })?;

        Ok(Self {
            config,
            client,
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            active_policies: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Initialize BearDog integration
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing BearDog security integration");

        // Test connection
        self.test_connection().await?;

        // Load security policies
        self.load_security_policies().await?;

        // Start background tasks
        self.start_background_tasks().await;

        info!("BearDog integration initialized successfully");
        Ok(())
    }

    /// Test connection to BearDog API
    async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/health", self.config.endpoint);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            debug!("BearDog connection test successful");
            Ok(())
        } else {
            Err(format!("BearDog connection failed: {}", response.status()).into())
        }
    }

    /// Load security policies from BearDog
    async fn load_security_policies(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/policies", self.config.endpoint);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            let policies: Vec<SecurityPolicy> = response.json().await?;
            let mut active_policies = self.active_policies.write().await;
            *active_policies = policies;
            info!("Loaded {} security policies", active_policies.len());
            Ok(())
        } else {
            Err(format!("Failed to load policies: {}", response.status()).into())
        }
    }

    /// Submit security event to BearDog
    pub async fn submit_security_event(&self, event: SecurityEvent) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Submitting security event: {}", event.event_id);

        // Add to local buffer
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push(event.clone());
        }

        // Submit to BearDog API
        let url = format!("{}/api/v1/events", self.config.endpoint);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&event)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Security event submitted successfully");
            Ok(())
        } else {
            error!("Failed to submit security event: {}", response.status());
            Err(format!("Event submission failed: {}", response.status()).into())
        }
    }

    /// Assess threat level for given context
    pub async fn assess_threat(&self, context: &AuthenticationContext) -> Result<ThreatAssessment, Box<dyn std::error::Error>> {
        debug!("Assessing threat for user: {}", context.user_id);

        let assessment_request = serde_json::json!({
            "user_id": context.user_id,
            "source_ip": context.source_ip,
            "authentication_method": context.authentication_method,
            "risk_score": context.risk_score,
            "mfa_verified": context.mfa_verified
        });

        let url = format!("{}/api/v1/threat-assessment", self.config.endpoint);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&assessment_request)
            .send()
            .await?;

        if response.status().is_success() {
            let assessment: ThreatAssessment = response.json().await?;
            debug!("Threat assessment completed: risk_score={}", assessment.risk_score);
            Ok(assessment)
        } else {
            error!("Threat assessment failed: {}", response.status());
            Err(format!("Threat assessment failed: {}", response.status()).into())
        }
    }

    /// Check if authentication context passes security policies
    pub async fn validate_authentication(&self, context: &AuthenticationContext) -> Result<bool, Box<dyn std::error::Error>> {
        debug!("Validating authentication for user: {}", context.user_id);

        let policies = self.active_policies.read().await;
        
        for policy in policies.iter() {
            if !self.check_policy_compliance(context, policy).await {
                warn!("Authentication failed policy check: {}", policy.name);
                return Ok(false);
            }
        }

        // Perform threat assessment if high risk
        if context.risk_score > self.config.alert_threshold {
            let assessment = self.assess_threat(context).await?;
            if assessment.requires_immediate_action() {
                warn!("Authentication blocked due to high threat assessment");
                return Ok(false);
            }
        }

        debug!("Authentication validation passed");
        Ok(true)
    }

    /// Check compliance with a specific security policy
    async fn check_policy_compliance(&self, context: &AuthenticationContext, policy: &SecurityPolicy) -> bool {
        if !matches!(policy.enforcement_mode, EnforcementMode::Strict | EnforcementMode::Adaptive) {
            return true; // Monitor mode or disabled
        }

        for rule in &policy.rules {
            if rule.enabled && !self.evaluate_rule_condition(context, &rule.condition) {
                return false;
            }
        }

        true
    }

    /// Evaluate a rule condition against authentication context
    fn evaluate_rule_condition(&self, context: &AuthenticationContext, condition: &RuleCondition) -> bool {
        match condition {
            RuleCondition::RiskThreshold(threshold) => context.risk_score <= *threshold,
            RuleCondition::TimeWindow(window) => self.is_within_time_window(window),
            RuleCondition::MultiCondition(conditions) => {
                conditions.iter().all(|c| self.evaluate_rule_condition(context, c))
            },
            _ => true, // Simplified evaluation
        }
    }

    /// Check if current time is within time window
    fn is_within_time_window(&self, window: &TimeWindow) -> bool {
        let now = chrono::Utc::now();
        let hour = now.hour() as u8;
        let weekday = now.weekday().number_from_monday() as u8;

        hour >= window.start_hour && 
        hour <= window.end_hour &&
        window.days_of_week.contains(&weekday)
    }

    /// Start background monitoring tasks
    async fn start_background_tasks(&self) {
        self.start_event_batch_processor().await;
        self.start_policy_sync_task().await;
        self.start_health_check_task().await;
    }

    /// Start event batch processing
    async fn start_event_batch_processor(&self) {
        let buffer = self.event_buffer.clone();
        let client = self.client.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let events_to_process = {
                    let mut buffer_guard = buffer.write().await;
                    if buffer_guard.is_empty() {
                        continue;
                    }
                    let events = buffer_guard.clone();
                    buffer_guard.clear();
                    events
                };

                if let Err(e) = Self::batch_submit_events(&client, &config, events_to_process).await {
                    error!("Failed to batch submit events: {}", e);
                }
            }
        });
    }

    /// Batch submit events to BearDog
    async fn batch_submit_events(
        client: &Client,
        config: &BeardogConfig,
        events: Vec<SecurityEvent>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if events.is_empty() {
            return Ok(());
        }

        let url = format!("{}/api/v1/events/batch", config.endpoint);
        
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&events)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Batch submitted {} events successfully", events.len());
            Ok(())
        } else {
            Err(format!("Batch event submission failed: {}", response.status()).into())
        }
    }

    /// Start policy synchronization task
    async fn start_policy_sync_task(&self) {
        let policies = self.active_policies.clone();
        let client = self.client.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::sync_policies(&client, &config, &policies).await {
                    warn!("Failed to sync security policies: {}", e);
                }
            }
        });
    }

    /// Synchronize security policies
    async fn sync_policies(
        client: &Client,
        config: &BeardogConfig,
        policies: &Arc<RwLock<Vec<SecurityPolicy>>>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v1/policies", config.endpoint);
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            let new_policies: Vec<SecurityPolicy> = response.json().await?;
            let mut current_policies = policies.write().await;
            *current_policies = new_policies;
            debug!("Security policies synchronized");
            Ok(())
        } else {
            Err(format!("Failed to sync policies: {}", response.status()).into())
        }
    }

    /// Start health check task
    async fn start_health_check_task(&self) {
        let client = self.client.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(120)); // 2 minutes
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::perform_health_check(&client, &config).await {
                    warn!("BearDog health check failed: {}", e);
                }
            }
        });
    }

    /// Perform health check
    async fn perform_health_check(
        client: &Client,
        config: &BeardogConfig
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/health", config.endpoint);
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            debug!("BearDog health check passed");
            Ok(())
        } else {
            Err(format!("Health check failed: {}", response.status()).into())
        }
    }

    /// Get integration statistics
    pub async fn get_statistics(&self) -> IntegrationStatistics {
        let buffer = self.event_buffer.read().await;
        let policies = self.active_policies.read().await;

        IntegrationStatistics {
            pending_events: buffer.len(),
            active_policies: policies.len(),
            connection_status: "Connected".to_string(), // Simplified
            last_sync: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntegrationStatistics {
    pub pending_events: usize,
    pub active_policies: usize,
    pub connection_status: String,
    pub last_sync: DateTime<Utc>,
} 