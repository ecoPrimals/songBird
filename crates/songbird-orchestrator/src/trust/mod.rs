//! Trust Escalation System
//!
//! Implements progressive trust escalation from anonymous to hardware-verified.
//!
//! ## Trust Levels
//!
//! ```text
//! Level 0: Anonymous          → Can discover, no data shared
//! Level 1: Capability-Verified → Can coordinate tasks
//! Level 2: Role-Verified       → Can access service registry
//! Level 3: Identity-Verified   → Can see infrastructure details
//! Level 4: Hardware-Verified   → Full admin access (BearDog)
//! ```
//!
//! ## Progressive Escalation
//!
//! ```text
//! Tower A                          Tower B
//!    |                                |
//!    |  Anonymous Discovery           |
//!    |<==============================>|
//!    |  Trust Level: 0 (Anonymous)    |
//!    |                                |
//!    |  Verify Capabilities           |
//!    |<==============================>|
//!    |  Trust Level: 1 (Capability)   |
//!    |                                |
//!    |  Coordinate Tasks              |
//!    |<==============================>|
//!    |                                |
//!    |  Request Admin Access          |
//!    |------------------------------->|
//!    |  Provide Hardware Key          |
//!    |------------------------------->|
//!    |  Verify with BearDog           |
//!    |<==============================>|
//!    |  Trust Level: 4 (Hardware)     |
//!    |  Grant Full Access             |
//!    |<==============================>|
//! ```

pub mod escalation;
pub mod types;

pub use escalation::{TrustEscalationManager, TrustTimeouts};
pub use types::{TrustLevel, TrustRelationship};

