/// Gaming CLI Commands - Clean Modular Architecture
///
/// This module demonstrates excellent code organization for the SongBird project:
/// - Focused modules (each under 1000 lines,
/// - Clear separation of concerns
/// - Well-documented public API
/// - Easy to maintain and extend

pub mod commands;
pub mod handlers;
pub mod scan;
pub mod host;
pub mod join;

// Re-export the main types
pub use commands::*;
pub use handlers::*;