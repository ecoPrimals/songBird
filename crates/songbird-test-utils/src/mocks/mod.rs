//! Mock Primal Endpoints for Testing
//!
//! Provides HTTP server mocks for ecosystem primals to enable isolated testing
//! of Songbird's orchestration capabilities without requiring actual primal instances.
//!
//! ## Available Mocks
//!
//! - `MockToadStool` - Compute metrics and workload management
//! - `MockBearDog` - Security, authentication, and deployment
//! - `MockNestGate` - Storage and data management
//! - `MockSquirrel` - AI integration and MCP protocol
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_test_utils::mocks::*;
//!
//! #[tokio::test]
//! async fn test_orchestrator_with_mocks() {
//!     let mut toadstool = MockToadStool::new();
//!     let port = toadstool.start().await.map_err(|e| SongbirdError::configuration(format!("Mock setup failed: {}", e)))?;
//!     
//!     // Configure expected metrics
//!     toadstool.set_cpu_usage(45.0);
//!     toadstool.set_memory_usage(2_000_000_000);
//!     
//!     // Test orchestrator routing to mock
//!     // ...
//!     
//!     toadstool.stop().await;
//! }
//! ```

pub mod beardog;
pub mod common;
pub mod nestgate;
pub mod squirrel;
pub mod toadstool;

pub use beardog::MockBearDog;
pub use common::{HealthStatus, MockPrimalServer, MockResponse};
pub use nestgate::MockNestGate;
pub use squirrel::MockSquirrel;
pub use toadstool::MockToadStool;
