/// Production-grade LAN Gaming Manager - /// Refactored
// Refactored
///
/// This file now serves as a clean entry point to the modular production LAN gaming system.
/// The original 1373-line monolith has been refactored into focused, maintainable modules.
// Import the modular components
pub use crate::network::gaming::production_lan::*;

// Re-export for backward compatibility;
pub use ProductionLanManager as ProductionLanGameManager;
