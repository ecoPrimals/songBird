//! Songbird Unwrap Migrator Library
//!
//! Provides systematic migration of unwrap(), expect(), and panic! patterns
//! to use Songbird's graceful error handling.

pub mod systematic_migrator;

pub use systematic_migrator::{
    SystematicUnwrapMigrator,
    MigratorError,
    MigratorResult,
    CodebaseStats,
    MigrationResult,
};
