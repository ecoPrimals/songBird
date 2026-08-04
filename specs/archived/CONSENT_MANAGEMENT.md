# Consent Management Specification

**Status**: 🔴 Not Implemented  
**Priority**: Critical (Week 5)  
**Owner**: Songbird Core

---

## Overview

Songbird must request and respect human consent for resource-intensive operations, ensuring humans maintain control over their resources.

---

## Requirements

### Functional Requirements

1. **Consent Requests**
   - Request consent before expensive operations
   - Provide cost estimates
   - Provide time estimates
   - Explain resource requirements
   - Human-readable explanations

2. **Consent Tracking**
   - Track granted consents
   - Track denied consents
   - Consent expiration
   - Consent conditions
   - Consent revocation

3. **Consent Enforcement**
   - Block operations without consent
   - Enforce consent conditions
   - Alert when approaching limits
   - Auto-revoke expired consents

4. **User Preferences**
   - Auto-approve small operations
   - Require approval for expensive ops
   - Per-resource-type thresholds
   - Notification preferences

### Non-Functional Requirements

- Consent check latency < 10ms
- Clear, understandable explanations
- No silent resource usage
- Respect human dignity

---

## API Design

```rust
/// Consent request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequest {
    /// Unique request ID
    pub id: String,
    
    /// User who must consent
    pub user_id: UserId,
    
    /// Task requiring consent
    pub task_spec: TaskSpec,
    
    /// Estimated cost
    pub estimated_cost: f32,
    
    /// Estimated duration
    pub estimated_duration: Duration,
    
    /// Resources needed
    pub resources_needed: Vec<Resource>,
    
    /// Human-readable explanation
    pub explanation: String,
    
    /// Expires if not responded
    pub expires_at: DateTime<Utc>,
}

/// Resource to be used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub amount: f32,
    pub unit: String,
    pub cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Cpu,
    Gpu,
    Memory,
    Network,
    Storage,
}

/// Consent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentResponse {
    /// Request ID
    pub request_id: String,
    
    /// Granted or denied
    pub granted: bool,
    
    /// Optional conditions
    pub conditions: Vec<Condition>,
    
    /// Valid until
    pub valid_until: DateTime<Utc>,
    
    /// Reason (if denied)
    pub reason: Option<String>,
}

/// Consent condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Maximum cost allowed
    MaxCost(f32),
    
    /// Complete by specific time
    CompleteBy(DateTime<Utc>),
    
    /// Use specific tower
    UseTower(TowerId),
    
    /// Don't use specific tower
    AvoidTower(TowerId),
    
    /// Require periodic progress updates
    RequireUpdates { interval: Duration },
}

/// User consent preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentPreferences {
    pub user_id: UserId,
    
    /// Auto-approve tasks below this cost
    pub auto_approve_under_cost: f32,
    
    /// Require approval above this cost
    pub require_approval_above_cost: f32,
    
    /// Auto-deny tasks above this cost
    pub auto_deny_above_cost: f32,
    
    /// Notification method
    pub notification_method: NotificationMethod,
    
    /// Consent timeout (auto-deny if no response)
    pub consent_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationMethod {
    WebUI,
    Email,
    SMS,
    Webhook { url: String },
}

/// Consent manager
pub trait ConsentManager: Send + Sync {
    /// Request consent
    async fn request_consent(&self, request: ConsentRequest) -> Result<String>;
    
    /// Check consent status
    async fn check_consent(&self, request_id: &str) -> Result<Option<ConsentResponse>>;
    
    /// Respond to consent request
    async fn respond_to_consent(&self, response: ConsentResponse) -> Result<()>;
    
    /// Revoke consent
    async fn revoke_consent(&self, task_id: TaskId) -> Result<()>;
    
    /// Get user preferences
    async fn get_preferences(&self, user_id: UserId) -> Result<ConsentPreferences>;
    
    /// Update user preferences
    async fn update_preferences(&self, prefs: ConsentPreferences) -> Result<()>;
    
    /// Check if operation needs consent
    async fn needs_consent(&self, user_id: UserId, spec: &TaskSpec) -> Result<bool>;
}
```

---

## Consent Flow

```
┌─────────────────┐
│ Task Submitted  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      No      ┌─────────────────┐
│ Needs Consent?  ├──────────────►│ Execute Task    │
└────────┬────────┘               └─────────────────┘
         │ Yes
         ▼
┌─────────────────┐
│ Check Auto-     │
│ Approve Rules   │
└────────┬────────┘
         │
         ├─► Auto-approve → Execute
         │
         ├─► Auto-deny → Reject
         │
         └─► Needs human → Request consent
                          │
                          ▼
                   ┌─────────────────┐
                   │ Notify Human    │
                   └────────┬────────┘
                            │
                            ├─► Granted → Execute
                            │
                            ├─► Denied → Reject
                            │
                            └─► Timeout → Deny (fail-safe)
```

---

## Storage Schema

```sql
CREATE TABLE consent_requests (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    task_spec_json TEXT NOT NULL,
    estimated_cost REAL NOT NULL,
    estimated_duration_seconds INTEGER NOT NULL,
    explanation TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    responded_at INTEGER,
    granted INTEGER,
    conditions_json TEXT
);

CREATE TABLE consent_preferences (
    user_id TEXT PRIMARY KEY,
    auto_approve_under_cost REAL NOT NULL DEFAULT 1.0,
    require_approval_above_cost REAL NOT NULL DEFAULT 10.0,
    auto_deny_above_cost REAL NOT NULL DEFAULT 100.0,
    notification_method TEXT NOT NULL DEFAULT 'WebUI',
    consent_timeout_seconds INTEGER NOT NULL DEFAULT 300
);

CREATE INDEX idx_consent_user ON consent_requests(user_id);
CREATE INDEX idx_consent_expires ON consent_requests(expires_at);
```

---

## Implementation Plan

### Phase 1: Core Consent (Day 1-2)
- [ ] Consent request/response types
- [ ] Consent storage
- [ ] Consent checking logic
- [ ] Consent expiration

### Phase 2: Preferences (Day 2-3)
- [ ] User preferences
- [ ] Auto-approve rules
- [ ] Auto-deny rules
- [ ] Notification dispatch

### Phase 3: Enforcement (Day 3-4)
- [ ] Pre-execution consent check
- [ ] Condition enforcement
- [ ] Consent revocation
- [ ] Limit warnings

### Phase 4: UI Integration (Day 4-5)
- [ ] REST API endpoints
- [ ] WebSocket for real-time requests
- [ ] Simple consent UI
- [ ] Notification system

---

## REST API Endpoints

```rust
// Get pending consent requests
GET /api/consent/pending?user={user_id}

// Respond to consent request
POST /api/consent/{request_id}/respond
{
  "granted": true,
  "conditions": [
    { "MaxCost": 50.0 }
  ]
}

// Get/update preferences
GET /api/consent/preferences
PUT /api/consent/preferences

// Revoke consent
DELETE /api/consent/task/{task_id}
```

---

## Example Consent Request

```json
{
  "id": "consent-req-123",
  "user_id": "alice",
  "task_spec": {
    "type": "ModelTraining",
    "model": "llama-70b",
    "dataset_size_gb": 500
  },
  "estimated_cost": 45.50,
  "estimated_duration": "24 hours",
  "resources_needed": [
    {
      "resource_type": "Gpu",
      "amount": 2,
      "unit": "GPUs",
      "cost": 40.00
    },
    {
      "resource_type": "Network",
      "amount": 500,
      "unit": "GB",
      "cost": 5.50
    }
  ],
  "explanation": "Training a large language model will use 2 GPUs for 24 hours and transfer 500GB of training data. This is a significant operation that will cost approximately $45.50.",
  "expires_at": "2025-12-18T11:00:00Z"
}
```

---

## Success Criteria

- [ ] Expensive operations require consent
- [ ] Humans receive clear explanations
- [ ] Auto-approve/deny rules work
- [ ] Consent can be revoked
- [ ] No surprises ($500 bill prevented!)

---

## Human Dignity Principles

1. **Informed Consent**: Humans understand what they're agreeing to
2. **Clear Communication**: No technical jargon in explanations
3. **Control**: Humans can always say no or revoke
4. **Transparency**: Costs and resources clearly stated
5. **Fail-Safe**: Default to deny, not approve

---

## Testing Requirements

- Consent flow test (request → approve → execute)
- Auto-approve test (small task skips consent)
- Auto-deny test (expensive task auto-denied)
- Revocation test (cancel running task)
- Expiration test (consent expires if not responded)

---

## Dependencies

- Task lifecycle (for pre-execution check)
- Resource management (for cost estimation)
- Observability (for notifications)

